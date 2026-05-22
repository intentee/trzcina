use anyhow::Result;
use async_trait::async_trait;
use tokio_util::sync::CancellationToken;

#[async_trait]
pub trait Service: Send + 'static {
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
    async fn run(self: Box<Self>, cancellation_token: CancellationToken) -> Result<()>;
}
