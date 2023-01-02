use anyhow::{bail, Context};
use clap::{ArgGroup, Parser};
use endpoint::Endpoint;
use identifier::{IdentifyResult, TrafficIdentifier};
use smallvec::SmallVec;
use ssh::SSHIdentifier;
use std::net::SocketAddr;
use tls::TLSIdentifier;
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
#[command(group(ArgGroup::new("endpoint").multiple(true).required(true)))]
struct Args {
    /// Endpoint to SSH server
    #[arg(long, group = "endpoint")]
    ssh: Option<endpoint::Endpoint>,

    /// Endpoint to TLS server
    #[arg(long, group = "endpoint")]
    tls: Option<endpoint::Endpoint>,

    /// The port to listen to
    #[arg(short, long, default_value = "127.0.0.1:443")]
    listen: SocketAddr,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let Args { ssh, tls, listen } = Args::parse();

    let server = TcpListener::bind(&listen)
        .await
        .with_context(|| format!("Listening on {listen}"))?;

    log::info!("Server started on {listen}");

    loop {
        select! {
            _ = signal::ctrl_c() => {
                break;
            }

            r = server.accept() => {
                let (client, client_addr) = r?;
                log::debug!("Received connection from {client_addr}");
                serve_client(client_addr, client, ssh.clone(), tls.clone());
            }
        }
    }

    Ok(())
}

fn serve_client(
    client_addr: SocketAddr,
    mut client: TcpStream,
    ssh: Option<Endpoint>,
    tls: Option<Endpoint>,
) {
    let mut identitiers: SmallVec<
        [(
            Box<dyn TrafficIdentifier + Send + Sync>,
            Endpoint,
            &'static str,
        ); 2],
    > = Default::default();

    if let Some(ssh) = ssh {
        identitiers.push((Box::new(SSHIdentifier), ssh, "ssh"));
    }

    if let Some(tls) = tls {
        identitiers.push((Box::new(TLSIdentifier), tls, "tls"));
    }

    spawn(async move {
        let mut buf = vec![0u8; 4096];
        let mut bytes_read = 0usize;
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
            for (id, endpoint, name) in &identitiers {
                if id.identify(&buf) == IdentifyResult::Positive {
                    log::info!("Redirect {client_addr} to {name}://{endpoint}");
                    return redirect_tcp(client, endpoint, buf).await.with_context(|| {
                        format!(
                            "Error redirecting traffic from {client_addr} to {name}://{endpoint}"
                        )
                    });
                }
            }
        }

        log::warn!("Connection from {client_addr} is not recognizable");
        anyhow::Ok(())
    });
}

async fn redirect_tcp(
    mut client: TcpStream,
    endpoint: &Endpoint,
    buf: &[u8],
) -> anyhow::Result<()> {
    let mut upstream = TcpStream::connect((endpoint.addr.as_ref(), endpoint.port)).await?;
    upstream.set_nodelay(true)?;
    upstream.write_all(buf).await?;
    copy_bidirectional(&mut client, &mut upstream).await?;
    Ok(())
}
