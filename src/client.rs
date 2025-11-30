use std::process::Command;
use std::time::Duration;
use std::time::Instant;

use crate::Settings;
use crate::traits::Runnable;
use crate::transport::TransportClient;

pub struct Client {
    transport_client: TransportClient,
    settings: Settings,
}

impl Client {
    async fn spawn_server(settings: &Settings) -> anyhow::Result<TransportClient> {
        let exe = std::env::current_exe()?;

        Command::new(exe)
            .arg("--server")
            .spawn()?;

        let start = Instant::now();
        let timeout = Duration::from_millis(settings.client_timeout);

        loop {
            let transport_client = TransportClient::connect(&settings.socket_path).await;

            if transport_client.is_ok() {
                return transport_client
            }

            if start.elapsed() > timeout {
                anyhow::bail!("Server timed out")
            }

            std::thread::sleep(Duration::from_millis(settings.client_retry_ms));
        }
    }
}

impl Runnable for Client {
    async fn connect(settings: Settings) -> anyhow::Result<Self> {
        let transport_client = match TransportClient::connect(&settings.socket_path).await {
            Ok(x) => x,
            Err(_) => Self::spawn_server(&settings).await?,
        };

        Ok(Self { transport_client, settings })
    }

    async fn run(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}
