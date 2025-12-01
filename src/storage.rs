use crate::traits::Database;

pub struct Storage { }

impl Database for Storage {
    async fn connect(url: &str) -> anyhow::Result<Self> {
        Ok(Self { })
    }
}
