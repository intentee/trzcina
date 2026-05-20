use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use tokio::task::yield_now;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;
use trzcina_sendable_service::Service;
use trzcina_sendable_service::ServiceManager;
use trzcina_service::Manager;
use trzcina_service::RunToCompletionOptions;
use trzcina_service::RunningCollection;
use trzcina_service::ServiceShutdownOutcome;

struct ConfiguredService {
    hang_ignoring_cancellation: bool,
}

#[async_trait]
impl Service for ConfiguredService {
    async fn run(&mut self, _cancellation_token: CancellationToken) -> Result<()> {
        if self.hang_ignoring_cancellation {
            loop {
                yield_now().await;
            }
        }
        Ok(())
    }
}

#[tokio::test]
async fn aborts_hung_service_after_shutdown_deadline() {
    let mut manager = ServiceManager::default();
    manager.register_service(ConfiguredService {
        hang_ignoring_cancellation: false,
    });
    manager.register_service(ConfiguredService {
        hang_ignoring_cancellation: true,
    });

    let report = timeout(
        Duration::from_secs(5),
        manager
            .start(CancellationToken::new())
            .run_to_completion(RunToCompletionOptions {
                shutdown_deadline: Duration::from_millis(50),
            }),
    )
    .await
    .unwrap();

    assert_eq!(report.outcomes().len(), 2);
    assert!(matches!(
        report.outcomes()[0].outcome,
        ServiceShutdownOutcome::Completed
    ));
    assert!(matches!(
        report.outcomes()[1].outcome,
        ServiceShutdownOutcome::AbortedByShutdownDeadline
    ));
    assert!(report.into_result().is_err());
}
