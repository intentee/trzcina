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

struct ThreadBlockingService {
    block_duration: Duration,
}

#[async_trait]
impl Service for ThreadBlockingService {
    async fn run(&mut self, _cancellation_token: CancellationToken) -> Result<()> {
        std::thread::sleep(self.block_duration);
        Ok(())
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn reports_leaked_beyond_abort_deadline_when_service_ignores_abort() {
    let cancellation_token = CancellationToken::new();
    let cancellation_token_for_run = cancellation_token.clone();

    let mut manager = ServiceManager::default();
    manager.register_service(ThreadBlockingService {
        block_duration: Duration::from_millis(500),
    });

    let run_task = tokio::spawn(async move {
        manager
            .start(cancellation_token_for_run)
            .run_to_completion(RunToCompletionOptions {
                shutdown_deadline: Duration::from_millis(50),
            })
            .await
    });

    cancellation_token.cancel();

    let report = timeout(Duration::from_secs(2), run_task)
        .await
        .expect("manager must return within bounded time even when service ignores abort")
        .unwrap();

    assert_eq!(report.outcomes().len(), 1);
    assert!(matches!(
        report.outcomes()[0].outcome,
        ServiceShutdownOutcome::LeakedBeyondAbortDeadline,
    ));
    assert!(report.into_result().is_err());
}
