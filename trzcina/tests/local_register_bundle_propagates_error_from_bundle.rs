use anyhow::Result;
use anyhow::anyhow;
use async_trait::async_trait;
use trzcina::LocalService;
use trzcina::LocalServiceBundle;
use trzcina::LocalServiceManager;

struct ErringBundle;

#[async_trait(?Send)]
impl LocalServiceBundle for ErringBundle {
    async fn services(self) -> Result<Vec<Box<dyn LocalService>>> {
        Err(anyhow!("test bundle deliberately failed"))
    }
}

#[tokio::test]
async fn local_propagates_error_from_bundle() {
    let mut manager = LocalServiceManager::default();
    manager.register_bundle(ErringBundle).await.unwrap_err();
}
