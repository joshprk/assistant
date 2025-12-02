use std::io;
use std::path::Path;

use futures::SinkExt;
use futures::StreamExt;
use serde::Deserialize;
use serde::Serialize;
use tokio::net::UnixStream;
use tokio_util::bytes::Bytes;
use tokio_util::codec::Framed;
use tokio_util::codec::LengthDelimitedCodec;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum TransportEvent {
    ToolCall { name: String, arguments: String },
    MessageDelta { delta: String },
}

#[derive(Debug)]
pub struct TransportClient {
    framed: Framed<UnixStream, LengthDelimitedCodec>
}

impl TransportClient {
    pub async fn connect(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let stream = UnixStream::connect(path).await?;
        Self::from_stream(stream)
    }

    pub fn from_stream(stream: UnixStream) -> anyhow::Result<Self> {
        let codec = LengthDelimitedCodec::new();
        let framed = Framed::new(stream, codec);
        Ok(Self { framed })
    }

    pub async fn send(&mut self, msg: TransportEvent) -> io::Result<()> {
        let payload = serde_json::to_vec(&msg)?;
        self.framed.send(Bytes::from(payload)).await?;
        Ok(())
    }

    pub async fn recv(&mut self) -> io::Result<Option<TransportEvent>> {
        match self.framed.next().await {
            Some(Ok(f)) => {
                let evt = serde_json::from_slice::<TransportEvent>(&f)?;
                Ok(Some(evt))
            },
            Some(Err(e)) => Err(io::Error::other(e)),
            None => Ok(None),
        }
    }
}
