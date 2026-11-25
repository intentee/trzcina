use anyhow::Result;
use anyhow::anyhow;
use async_trait::async_trait;
use trzcina::Service;
use trzcina::ServiceBundle;
use trzcina::ServiceManager;

struct ErringBundle;

#[async_trait]
impl ServiceBundle for ErringBundle {
    async fn services(self) -> Result<Vec<Box<dyn Service>>> {
        Err(anyhow!("test bundle deliberately failed"))
    }
}

#[tokio::test]
async fn propagates_error_from_bundle() {
    let mut manager = ServiceManager::default();
    manager.register_bundle(ErringBundle).await.unwrap_err();
}
