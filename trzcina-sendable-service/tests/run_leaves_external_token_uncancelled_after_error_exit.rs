use std::time::Duration;

use anyhow::Result;
use anyhow::anyhow;
use async_trait::async_trait;
use tokio_util::sync::CancellationToken;
use trzcina_sendable_service::Service;
use trzcina_sendable_service::ServiceManager;
use trzcina_service::Manager;
use trzcina_service::RunToCompletionOptions;
use trzcina_service::RunningCollection;

struct ImmediatelyErroringService;

#[async_trait]
impl Service for ImmediatelyErroringService {
    async fn run(&mut self, _cancellation_token: CancellationToken) -> Result<()> {
        Err(anyhow!("service failed"))
    }
}

#[tokio::test]
async fn leaves_external_token_uncancelled_after_error_exit() {
    let external_token = CancellationToken::new();
    let mut manager = ServiceManager::default();
    manager.register_service(ImmediatelyErroringService);

    let _ = manager
        .start(external_token.clone())
        .run_to_completion(RunToCompletionOptions {
            shutdown_deadline: Duration::from_secs(1),
        })
        .await
        .into_result();

    assert!(!external_token.is_cancelled());
}
