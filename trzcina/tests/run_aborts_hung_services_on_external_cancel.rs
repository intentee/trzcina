use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use tokio::task::yield_now;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;
use trzcina::Service;
use trzcina::ServiceManager;
use trzcina::ServiceShutdownOptions;
use trzcina::ServiceShutdownOutcome;

struct CancellationIgnoringService;

#[async_trait]
impl Service for CancellationIgnoringService {
    async fn run(self: Box<Self>, _cancellation_token: CancellationToken) -> Result<()> {
        loop {
            yield_now().await;
        }
    }
}

#[tokio::test]
async fn aborts_hung_services_on_external_cancel() {
    let cancellation_token = CancellationToken::new();
    let cancellation_token_for_run = cancellation_token.clone();

    let mut manager = ServiceManager::default();
    manager.register_service(CancellationIgnoringService);
    manager.register_service(CancellationIgnoringService);

    let run_task = tokio::spawn(async move {
        manager
            .start(cancellation_token_for_run)
            .run_to_completion(ServiceShutdownOptions {
                cooperative_deadline: Duration::from_millis(50),
                abort_deadline: Duration::from_millis(50),
            })
            .await
    });

    cancellation_token.cancel();

    let report = timeout(Duration::from_secs(5), run_task)
        .await
        .expect("manager must return within outer timeout when token is externally cancelled")
        .unwrap();

    assert_eq!(report.outcomes().len(), 2);
    for named_outcome in report.outcomes() {
        assert!(matches!(
            named_outcome.outcome,
            ServiceShutdownOutcome::AbortedByShutdownDeadline,
        ));
    }
    assert!(report.into_result().is_err());
}
