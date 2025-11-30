use std::io;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::os::unix::net::UnixStream;

use rand::Rng;
use serde::Deserialize;
use serde::Serialize;

pub const SERVER_ADDR: u8 = 0;
pub const BROADCAST_ADDR: u8 = 255;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum TransportEvent {
    ClientJoined { request_id: u64 },
    ProvisionAddr { request_id: u64, new_addr: u8 },
    Ping { },
}

#[derive(Debug)]
pub struct TransportClient {
    addr: u8,
    stream: UnixStream,
}

impl TransportClient {
    pub async fn connect(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let stream = UnixStream::connect(path)?;
        let mut client = Self { addr: BROADCAST_ADDR, stream };

        let provision_request_id = rand::rng().random_range(..u64::MAX);
        let provision_request = TransportEvent::ClientJoined { request_id: provision_request_id };

        client.send(0, provision_request).await?;

        loop {
            let event = client.recv().await?;

            match event {
                TransportEvent::ProvisionAddr { request_id, new_addr } => {
                    if request_id == provision_request_id {
                        client.addr = new_addr;
                        return Ok(client)
                    }
                },
                _ => continue,
            };
        }
    }

    pub fn connect_with_addr(path: impl AsRef<Path>, addr: u8) -> anyhow::Result<Self> {
        let stream = UnixStream::connect(path)?;
        Ok(Self { addr, stream })
    }

    pub fn from_stream(stream: UnixStream, addr: u8) -> anyhow::Result<Self> {
        Ok(Self { addr, stream })
    }

    pub async fn send(&mut self, recipient: u8, msg: TransportEvent) -> io::Result<()> {
        let payload = serde_json::to_vec(&msg)?;
        let len_buf = (payload.len() as u32).to_be_bytes();
        let recipient = recipient.to_be_bytes();

        self.stream.write_all(&len_buf)?;
        self.stream.write_all(&recipient)?;
        self.stream.write_all(&payload)?;

        self.stream.flush()
    }

    pub async fn recv(&mut self) -> io::Result<TransportEvent> {
        loop {
            let mut len_buf = [0u8; 4];
            let mut recipient_buf = [0u8; 1];

            self.stream.read_exact(&mut len_buf)?;
            self.stream.read_exact(&mut recipient_buf)?;

            let len = u32::from_be_bytes(len_buf);
            let recipient = recipient_buf[0];

            let mut buf = vec![0; len as usize];

            self.stream.read_exact(&mut buf)?;

            if recipient == self.addr || recipient == BROADCAST_ADDR {
                let event = serde_json::from_slice::<TransportEvent>(&buf)?;
                return Ok(event)
            }
        }
    }
}
