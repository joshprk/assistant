use crate::Settings;

pub trait Runnable {
    async fn run(settings: Settings) -> anyhow::Result<()>;
}
