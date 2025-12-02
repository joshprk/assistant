use tokio::net::UnixListener;

use crate::Settings;
use crate::traits::Runnable;
use crate::transport::TransportClient;

pub struct Server { }

impl Runnable for Server {
    async fn run(settings: Settings) -> anyhow::Result<()> {
        if TransportClient::connect(&settings.socket_path).await.is_ok() {
            return Err(anyhow::anyhow!("Another server is already running"))
        }

        if settings.socket_path.exists() {
            std::fs::remove_file(&settings.socket_path)?;
        }

        let listener = UnixListener::bind(&settings.socket_path)?;

        loop {
            let (stream, _addr) = listener.accept().await?;
            let _transport_client = TransportClient::from_stream(stream)?;
        }
    }
}
