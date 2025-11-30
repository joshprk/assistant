use std::os::unix::net::UnixListener;

use crate::Settings;
use crate::traits::Runnable;
use crate::transport::SERVER_ADDR;
use crate::transport::TransportClient;
use crate::transport::TransportEvent;

pub struct Server {
    listener: UnixListener,
    settings: Settings,
    new_addr: u8,
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.settings.socket_path);
    }
}

impl Runnable for Server {
    async fn connect(settings: Settings) -> anyhow::Result<Self> {
        let listener = UnixListener::bind(&settings.socket_path)?;
        let new_addr = SERVER_ADDR + 1;
        Ok(Self { listener, settings, new_addr })
    }

    async fn run(&mut self) -> anyhow::Result<()> {
        // TODO
        loop {
            let (stream, _addr) = self.listener.accept()?;
            let mut transport_client = TransportClient::from_stream(stream, 0)?;

            if let TransportEvent::ClientJoined { request_id } = transport_client.recv().await? {
                let _ = transport_client.send(255, TransportEvent::ProvisionAddr {
                    request_id,
                    new_addr: self.new_addr,
                }).await;
            }
        }
    }
}
