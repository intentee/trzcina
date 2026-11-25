use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::oneshot;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;
use trzcina::Service;
use trzcina::ServiceManager;
use trzcina::ServiceShutdownOptions;
use trzcina::ServiceShutdownOutcome;

struct AwaitingService {
    observation_tx: oneshot::Sender<()>,
}

#[async_trait]
impl Service for AwaitingService {
    async fn run(self: Box<Self>, cancellation_token: CancellationToken) -> Result<()> {
        cancellation_token.cancelled().await;
        self.observation_tx.send(()).unwrap();
        Ok(())
    }
}

#[tokio::test]
async fn cancels_all_services_when_external_token_cancelled() {
    let cancellation_token = CancellationToken::new();
    let cancellation_token_for_run = cancellation_token.clone();
    let mut manager = ServiceManager::default();
    let mut observation_receivers = Vec::new();

    for _ in 0..5 {
        let (observation_tx, observation_rx) = oneshot::channel::<()>();
        manager.register_service(AwaitingService { observation_tx });
        observation_receivers.push(observation_rx);
    }

    let run_task = tokio::spawn(async move {
        manager
            .start(cancellation_token_for_run)
            .run_to_completion(ServiceShutdownOptions::default())
            .await
    });

    cancellation_token.cancel();

    let report = timeout(Duration::from_secs(5), run_task)
        .await
        .unwrap()
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
