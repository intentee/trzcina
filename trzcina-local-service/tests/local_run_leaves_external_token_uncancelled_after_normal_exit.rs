use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use tokio_util::sync::CancellationToken;
use trzcina_local_service::LocalService;
use trzcina_local_service::LocalServiceManager;
use trzcina_service::Manager;
use trzcina_service::RunToCompletionOptions;
use trzcina_service::RunningCollection;

struct ImmediatelyExitingLocalService;

#[async_trait(?Send)]
impl LocalService for ImmediatelyExitingLocalService {
    async fn run(&mut self, _cancellation_token: CancellationToken) -> Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn local_leaves_external_token_uncancelled_after_normal_exit() {
    let external_token = CancellationToken::new();
    let mut manager = LocalServiceManager::default();
    manager.register_service(ImmediatelyExitingLocalService);

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
