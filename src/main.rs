use std::net::SocketAddr;
use anyhow::bail;
use clap::{Parser, ArgGroup};

mod endpoint;
mod identifier;
mod ssh;
mod strings;
mod tls;

const fn default_listen_addr() -> SocketAddr {
    SocketAddr::new()
}

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
    #[arg(short, long)]
    listen: SocketAddr,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    todo!()
}
