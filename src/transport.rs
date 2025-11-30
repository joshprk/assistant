use std::io;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::os::unix::net::UnixStream;

use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum TransportEvent {
    ToolCall { name: String, arguments: String },
    MessageDelta { delta: String },
}

#[derive(Debug)]
pub struct TransportClient {
    stream: UnixStream
}

impl TransportClient {
    pub async fn connect(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let stream = UnixStream::connect(path)?;
        Ok(Self { stream })
    }

    pub fn from_stream(stream: UnixStream) -> anyhow::Result<Self> {
        Ok(Self { stream })
    }

    pub async fn send(&mut self, msg: TransportEvent) -> io::Result<()> {
        let payload = serde_json::to_vec(&msg)?;
        let len_buf = (payload.len() as u32).to_be_bytes();

        self.stream.write_all(&len_buf)?;
        self.stream.write_all(&payload)?;

        self.stream.flush()
    }

    pub async fn recv(&mut self) -> io::Result<TransportEvent> {
        let mut len_buf = [0u8; 4];

        self.stream.read_exact(&mut len_buf)?;

        let len = u32::from_be_bytes(len_buf);
        let mut payload = vec![0; len as usize];

        self.stream.read_exact(&mut payload)?;

        Ok(serde_json::from_slice::<TransportEvent>(&payload)?)
    }
}
