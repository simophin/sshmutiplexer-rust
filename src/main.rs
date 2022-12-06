use anyhow::{bail, Context};
use clap::{ArgGroup, Parser};
use std::net::SocketAddr;
use tokio::{
    net::{TcpListener, TcpSocket, TcpStream},
    spawn,
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
        let (client, client_addr) = server.accept().await?;
        log::debug!("Received connection from {client_addr}");
        serve_client(client, ssh.clone(), tls.clone());
    }
}

fn serve_client(
    mut client: TcpStream,
    ssh: Option<endpoint::Endpoint>,
    tls: Option<endpoint::Endpoint>,
) {
    spawn(async move {
        
    });
}
