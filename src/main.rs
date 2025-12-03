use std::path::PathBuf;

use clap::Parser;

use crate::client::Client;
use crate::server::Server;
use crate::traits::Runnable;

mod chat;
mod client;
mod server;
mod traits;
mod transport;
mod ui;

#[derive(Debug, Parser)]
struct Args {
    #[clap(long)]
    server: bool
}

struct Settings {
    client_timeout: u64,
    client_retry_ms: u64,
    client_ui_poll_ms: u64,
    socket_path: PathBuf,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Temporary
    let settings = Settings {
        client_timeout: 10,
        client_retry_ms: 100,
        client_ui_poll_ms: 100,
        socket_path: "./test.sock".into()
    };

    if args.server {
        Server::run(settings).await
    } else {
        Client::run(settings).await
    }
}
