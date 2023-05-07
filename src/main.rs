use anyhow::{bail, Context};
use async_shutdown::Shutdown;
use clap::Parser;
use endpoint::Endpoint;
use identifier::IdentifyResult;
use std::net::SocketAddr;
use tokio::{
    io::{copy_bidirectional, AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    select, signal, spawn,
};

mod endpoint;
mod identifier;
mod ssh;
mod strings;
mod tls;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Endpoint to SSH server
    #[arg(long)]
    ssh: Endpoint,

    /// Endpoint to the HTTPS/TLS server
    #[arg(long)]
    tls: Endpoint,

    /// Endpoint to the Web server
    #[arg(long)]
    web: Endpoint,

    /// Whether to use the PROXY protocol
    #[arg(long, default_value_t = false)]
    enable_proxy_protocol: bool,

    /// The TLS endpoint to listen on
    #[arg(short, long, default_value = ":::443")]
    tls_listen: Endpoint,

    /// The Web endpoint to listen on
    #[arg(short, long, default_value = ":::80")]
    web_listen: Endpoint,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let Args {
        ssh,
        tls,
        web,
        enable_proxy_protocol,
        tls_listen,
        web_listen,
    } = Args::parse();

    let shutdown = Shutdown::new();

    let tls_listener = TcpListener::bind((tls_listen.addr.as_ref(), tls_listen.port))
        .await
        .with_context(|| format!("Failed to bind TLS listener to {tls_listen}"))?;

    log::info!("Listening on {tls_listen} for TLS traffic");

    let http_listener = TcpListener::bind((web_listen.addr.as_ref(), web_listen.port))
        .await
        .with_context(|| format!("Failed to bind HTTP listener to {web_listen}"))?;

    log::info!("Listening on {web_listen} for HTTP traffic");

    let tls_task = spawn(serve_tls(
        shutdown.clone(),
        tls_listener,
        ssh,
        tls,
        enable_proxy_protocol,
    ));

    let http_task = spawn(serve_web(
        shutdown.clone(),
        http_listener,
        web,
        enable_proxy_protocol,
    ));

    select! {
        res = tls_task => {
            if let Err(e) = res {
                log::error!("TLS task failed: {e}");
            }
        }
        res = http_task => {
            if let Err(e) = res {
                log::error!("HTTP task failed: {e}");
            }
        }

        _ = signal::ctrl_c() => {
            log::info!("Terminate signal received, shutting down");
        }
    }

    shutdown.shutdown();

    Ok(())
}

async fn serve_tls(
    shutdown: Shutdown,
    listener: TcpListener,
    ssh: Endpoint,
    tls: Endpoint,
    enable_proxy_protocol: bool,
) -> anyhow::Result<()> {
    let bound_addr = listener
        .local_addr()
        .context("Getting TLS listener address")?;
    while let Some(client) = shutdown.wrap_cancel(listener.accept()).await {
        let (client, client_addr) = client.context("Receiving a connection")?;
        log::debug!("Received tls/ssh connection from {client_addr}");

        spawn(shutdown.wrap_cancel(serve_tls_client(
            bound_addr,
            client_addr,
            client,
            ssh.clone(),
            tls.clone(),
            enable_proxy_protocol,
        )));
    }

    Ok(())
}

async fn serve_web(
    shutdown: Shutdown,
    listener: TcpListener,
    web: Endpoint,
    enable_proxy_protocol: bool,
) -> anyhow::Result<()> {
    let bound_addr = listener
        .local_addr()
        .context("Getting TLS listener address")?;
    while let Some(client) = shutdown.wrap_cancel(listener.accept()).await {
        let (client, client_addr) = client.context("Receiving a connection")?;
        log::debug!("Received web connection from {client_addr}");

        spawn(shutdown.wrap_cancel(serve_web_client(
            bound_addr,
            client_addr,
            client,
            web.clone(),
            enable_proxy_protocol,
        )));
    }

    Ok(())
}

async fn serve_web_client(
    bound_addr: SocketAddr,
    client_addr: SocketAddr,
    client: TcpStream,
    web: Endpoint,
    enable_proxy_protocol: bool,
) -> anyhow::Result<()> {
    log::debug!("Redirecting {client_addr} to {web}");
    redirect_tcp(
        client,
        &web,
        bound_addr,
        &[],
        if enable_proxy_protocol {
            Some(client_addr)
        } else {
            None
        },
    )
    .await
}

async fn serve_tls_client(
    bound_addr: SocketAddr,
    client_addr: SocketAddr,
    mut client: TcpStream,
    ssh: Endpoint,
    tls: Endpoint,
    enable_proxy_protocol: bool,
) -> anyhow::Result<()> {
    let mut buf = vec![0u8; 4096];
    let mut bytes_read = 0usize;
    let mut identify_ssh = true;
    let mut identify_tls = true;

    while bytes_read < buf.len() {
        let len = client
            .read(&mut buf[bytes_read..])
            .await
            .context("Reading from {client_addr}")?;

        if len == 0 {
            bail!("Invalid read len");
        }

        bytes_read += len;

        let buf = &buf[..bytes_read];

        if identify_ssh {
            match ssh::identify(buf) {
                IdentifyResult::Positive => {
                    log::info!("Redirect {client_addr} to SSH://{ssh}");
                    return redirect_tcp(client, &ssh, bound_addr, buf, None).await;
                }
                IdentifyResult::Negative => identify_ssh = false,
                IdentifyResult::NeedMoreData => {}
            }
        }

        if identify_tls {
            match tls::identify(buf) {
                IdentifyResult::Positive => {
                    log::info!("Redirect {client_addr} to TLS://{tls}");
                    return redirect_tcp(
                        client,
                        &tls,
                        bound_addr,
                        buf,
                        if enable_proxy_protocol {
                            Some(client_addr)
                        } else {
                            None
                        },
                    )
                    .await;
                }
                IdentifyResult::Negative => identify_tls = false,
                IdentifyResult::NeedMoreData => {}
            }
        }

        if !identify_ssh && !identify_tls {
            break;
        }
    }

    log::warn!("Connection from {client_addr} is not recognizable");
    Ok(())
}

async fn redirect_tcp(
    mut client: TcpStream,
    endpoint: &Endpoint,
    bound_addr: SocketAddr,
    buf: &[u8],
    proxy_protocol_client: Option<SocketAddr>,
) -> anyhow::Result<()> {
    let mut upstream = TcpStream::connect((endpoint.addr.as_ref(), endpoint.port)).await?;
    upstream.set_nodelay(true)?;

    match proxy_protocol_client {
        Some(SocketAddr::V4(addr)) => {
            upstream
                .write_all(
                    format!(
                        "PROXY TCP4 {source_addr} {dest_addr} {source_port} {dest_port}\r\n",
                        source_addr = addr.ip(),
                        source_port = addr.port(),
                        dest_addr = bound_addr.ip(),
                        dest_port = bound_addr.port(),
                    )
                    .as_bytes(),
                )
                .await?;
        }

        Some(SocketAddr::V6(addr)) => {
            upstream
                .write_all(
                    format!(
                        "PROXY TCP6 {source_addr} {dest_addr} {source_port} {dest_port}\r\n",
                        source_addr = addr.ip(),
                        source_port = addr.port(),
                        dest_addr = bound_addr.ip(),
                        dest_port = bound_addr.port(),
                    )
                    .as_bytes(),
                )
                .await?;
        }

        None => {}
    }

    upstream.write_all(buf).await?;
    copy_bidirectional(&mut client, &mut upstream).await?;
    Ok(())
}
