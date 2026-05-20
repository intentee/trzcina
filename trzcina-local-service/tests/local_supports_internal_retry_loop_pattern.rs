use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::oneshot;
use tokio::time::sleep;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;
use trzcina_local_service::LocalService;
use trzcina_local_service::LocalServiceManager;
use trzcina_service::Manager;
use trzcina_service::RunToCompletionOptions;
use trzcina_service::RunningCollection;
use trzcina_service::ServiceShutdownOutcome;

struct RetryLoopService {
    backoff_started_tx: Option<oneshot::Sender<()>>,
}

#[async_trait(?Send)]
impl LocalService for RetryLoopService {
    async fn run(&mut self, cancellation_token: CancellationToken) -> Result<()> {
        loop {
            if let Some(backoff_started_tx) = self.backoff_started_tx.take() {
                backoff_started_tx.send(()).unwrap();
            }
            tokio::select! {
                () = cancellation_token.cancelled() => return Ok(()),
                () = sleep(Duration::from_secs(10)) => continue,
            }
        }
    }
}

#[tokio::test]
async fn local_supports_internal_retry_loop_pattern() {
    let (backoff_started_tx, backoff_started_rx) = oneshot::channel::<()>();
    let cancellation_token = CancellationToken::new();
    let cancellation_token_for_run = cancellation_token.clone();

    let mut manager = LocalServiceManager::default();
    manager.register_service(RetryLoopService {
        backoff_started_tx: Some(backoff_started_tx),
    });

    let run_future =
        manager
            .start(cancellation_token_for_run)
            .run_to_completion(RunToCompletionOptions {
                shutdown_deadline: Duration::from_secs(1),
            });
    let trigger_future = async move {
        backoff_started_rx.await.unwrap();
        cancellation_token.cancel();
    };

    let (report, ()) = timeout(Duration::from_secs(5), async {
        tokio::join!(run_future, trigger_future)
    })
    .await
    .unwrap();

    assert_eq!(report.outcomes().len(), 1);
    assert!(matches!(
        report.outcomes()[0].outcome,
        ServiceShutdownOutcome::Completed
    ));
}
