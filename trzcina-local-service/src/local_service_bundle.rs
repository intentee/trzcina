use anyhow::Result;
use async_trait::async_trait;

use crate::local_service::LocalService;

#[async_trait(?Send)]
pub trait LocalServiceBundle {
    async fn services(self) -> Result<Vec<Box<dyn LocalService>>>;
}
