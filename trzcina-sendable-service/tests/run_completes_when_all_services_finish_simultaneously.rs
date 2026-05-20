use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;
use trzcina_sendable_service::Service;
use trzcina_sendable_service::ServiceManager;
use trzcina_service::Manager;
use trzcina_service::RunToCompletionOptions;
use trzcina_service::RunningCollection;
use trzcina_service::ServiceShutdownOutcome;

struct InstantOkService;

#[async_trait]
impl Service for InstantOkService {
    async fn run(&mut self, _cancellation_token: CancellationToken) -> Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn completes_when_all_services_finish_simultaneously() {
    let mut manager = ServiceManager::default();
    for _ in 0..5 {
        manager.register_service(InstantOkService);
    }

    let report = timeout(
        Duration::from_secs(5),
        manager
            .start(CancellationToken::new())
            .run_to_completion(RunToCompletionOptions {
                shutdown_deadline: Duration::from_secs(1),
            }),
    )
    .await
    .unwrap();

    assert_eq!(report.outcomes().len(), 5);
    for named_outcome in report.outcomes() {
        assert!(matches!(
            named_outcome.outcome,
            ServiceShutdownOutcome::Completed
        ));
    }
}
