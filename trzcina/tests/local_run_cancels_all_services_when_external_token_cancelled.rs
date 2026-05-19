use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::oneshot;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;
use trzcina::LocalService;
use trzcina::LocalServiceManager;
use trzcina::ServiceShutdownOutcome;

struct AwaitingService {
    observation_tx: Option<oneshot::Sender<()>>,
}

#[async_trait(?Send)]
impl LocalService for AwaitingService {
    async fn run(&mut self, cancellation_token: CancellationToken) -> Result<()> {
        cancellation_token.cancelled().await;
        if let Some(observation_tx) = self.observation_tx.take() {
            observation_tx.send(()).unwrap();
        }
        Ok(())
    }
}

#[tokio::test]
async fn local_cancels_all_services_when_external_token_cancelled() {
    let cancellation_token = CancellationToken::new();
    let cancellation_token_for_run = cancellation_token.clone();
    let mut manager = LocalServiceManager::default();
    let mut observation_receivers = Vec::new();

    for _ in 0..5 {
        let (observation_tx, observation_rx) = oneshot::channel::<()>();
        manager.register_service(AwaitingService {
            observation_tx: Some(observation_tx),
        });
        observation_receivers.push(observation_rx);
    }

    let run_future = manager
        .start_local(cancellation_token_for_run)
        .run_to_completion(Duration::from_secs(1));
    let trigger_future = async move {
        cancellation_token.cancel();
    };

    let (report, ()) = timeout(Duration::from_secs(5), async {
        tokio::join!(run_future, trigger_future)
    })
    .await
    .unwrap();

    assert_eq!(report.outcomes().len(), 5);
    for named_outcome in report.outcomes() {
        assert!(matches!(
            named_outcome.outcome,
            ServiceShutdownOutcome::Completed
        ));
    }
    for mut observation_rx in observation_receivers {
        observation_rx.try_recv().unwrap();
    }
}
