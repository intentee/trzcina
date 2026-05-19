use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::oneshot;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;
use trzcina::LocalService;
use trzcina::LocalServiceManager;
use trzcina::ServiceShutdownOutcome;

struct ActixStyleService {
    started_tx: Option<oneshot::Sender<()>>,
}

#[async_trait(?Send)]
impl LocalService for ActixStyleService {
    async fn run(&mut self, cancellation_token: CancellationToken) -> Result<()> {
        if let Some(started_tx) = self.started_tx.take() {
            started_tx.send(()).unwrap();
        }
        loop {
            if cancellation_token.is_cancelled() {
                break;
            }
            cancellation_token.cancelled().await;
        }
        Ok(())
    }
}

#[tokio::test]
async fn local_supports_actix_style_shutdown_signal_pattern() {
    let cancellation_token = CancellationToken::new();
    let cancellation_token_for_run = cancellation_token.clone();
    let (started_tx, started_rx) = oneshot::channel::<()>();

    let mut manager = LocalServiceManager::default();
    manager.register_service(ActixStyleService {
        started_tx: Some(started_tx),
    });

    let run_future = manager
        .start_local(cancellation_token_for_run)
        .run_to_completion(Duration::from_secs(1));
    let trigger_future = async move {
        started_rx.await.unwrap();
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
