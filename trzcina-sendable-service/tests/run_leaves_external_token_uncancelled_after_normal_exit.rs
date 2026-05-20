use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use tokio_util::sync::CancellationToken;
use trzcina_sendable_service::Service;
use trzcina_sendable_service::ServiceManager;
use trzcina_service::Manager;
use trzcina_service::RunToCompletionOptions;
use trzcina_service::RunningCollection;

struct ImmediatelyExitingService;

#[async_trait]
impl Service for ImmediatelyExitingService {
    async fn run(&mut self, _cancellation_token: CancellationToken) -> Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn leaves_external_token_uncancelled_after_normal_exit() {
    let external_token = CancellationToken::new();
    let mut manager = ServiceManager::default();
    manager.register_service(ImmediatelyExitingService);

    manager
        .start(external_token.clone())
        .run_to_completion(RunToCompletionOptions {
            shutdown_deadline: Duration::from_secs(1),
        })
        .await
        .into_result()
        .unwrap();

    assert!(!external_token.is_cancelled());
}
