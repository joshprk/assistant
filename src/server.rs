use std::os::unix::net::UnixListener;

use crate::Settings;
use crate::traits::Runnable;
use crate::transport::TransportClient;
use crate::transport::TransportEvent;

pub struct Server {
    listener: UnixListener,
    settings: Settings,
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.settings.socket_path);
    }
}

impl Runnable for Server {
    async fn connect(settings: Settings) -> anyhow::Result<Self> {
        let listener = UnixListener::bind(&settings.socket_path)?;
        Ok(Self { listener, settings })
    }

    async fn run(&mut self) -> anyhow::Result<()> {
        // TODO
        loop {
            let (stream, _addr) = self.listener.accept()?;
            let _transport_client = TransportClient::from_stream(stream)?;
        }
    }
}
