use crate::Settings;

pub trait Runnable {
    async fn connect(settings: Settings) -> anyhow::Result<Self>
        where Self: Sized;
    async fn run(&mut self) -> anyhow::Result<()>;
}
