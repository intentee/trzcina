use anyhow::Result;
use async_trait::async_trait;

use crate::service::Service;

#[async_trait]
pub trait ServiceBundle {
    async fn services(self) -> Result<Vec<Box<dyn Service>>>;
}
