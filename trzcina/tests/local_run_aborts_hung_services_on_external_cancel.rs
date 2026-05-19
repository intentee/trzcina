use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use tokio::task::yield_now;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;
use trzcina::LocalService;
use trzcina::LocalServiceManager;
use trzcina::ServiceShutdownOutcome;

struct CancellationIgnoringService;

#[async_trait(?Send)]
impl LocalService for CancellationIgnoringService {
    async fn run(&mut self, _cancellation_token: CancellationToken) -> Result<()> {
        loop {
            yield_now().await;
        }
    }
}

#[tokio::test]
async fn local_aborts_hung_services_on_external_cancel() {
    let cancellation_token = CancellationToken::new();
    let cancellation_token_for_run = cancellation_token.clone();

    let mut manager = LocalServiceManager::default();
    manager.register_service(CancellationIgnoringService);
    manager.register_service(CancellationIgnoringService);

    let run_future = manager
        .start_local(cancellation_token_for_run)
        .run_to_completion(Duration::from_millis(50));
    let trigger_future = async move {
        cancellation_token.cancel();
    };

    let (report, ()) = timeout(Duration::from_secs(5), async {
        tokio::join!(run_future, trigger_future)
    })
    .await
    .expect("manager must return within outer timeout when token is externally cancelled");

    assert_eq!(report.outcomes().len(), 2);
    for named_outcome in report.outcomes() {
        assert!(matches!(
            named_outcome.outcome,
            ServiceShutdownOutcome::AbortedByShutdownDeadline,
        ));
    }
    assert!(report.into_result().is_err());
}
