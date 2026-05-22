use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::oneshot;
use tokio::time::sleep;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;
use trzcina::Service;
use trzcina::ServiceManager;
use trzcina::ServiceShutdownOptions;
use trzcina::ServiceShutdownOutcome;

struct RetryLoopService {
    backoff_started_tx: Option<oneshot::Sender<()>>,
}

#[async_trait]
impl Service for RetryLoopService {
    async fn run(mut self: Box<Self>, cancellation_token: CancellationToken) -> Result<()> {
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
async fn supports_internal_retry_loop_pattern() {
    let (backoff_started_tx, backoff_started_rx) = oneshot::channel::<()>();
    let cancellation_token = CancellationToken::new();
    let cancellation_token_for_run = cancellation_token.clone();

    let mut manager = ServiceManager::default();
    manager.register_service(RetryLoopService {
        backoff_started_tx: Some(backoff_started_tx),
    });

    let run_task = tokio::spawn(async move {
        manager
            .start(cancellation_token_for_run)
            .run_to_completion(ServiceShutdownOptions::default())
            .await
    });

    backoff_started_rx.await.unwrap();
    cancellation_token.cancel();

    let report = timeout(Duration::from_secs(5), run_task)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(report.outcomes().len(), 1);
    assert!(matches!(
        report.outcomes()[0].outcome,
        ServiceShutdownOutcome::Completed
    ));
}
