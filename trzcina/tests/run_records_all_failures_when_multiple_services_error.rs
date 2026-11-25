use std::time::Duration;

use anyhow::Result;
use anyhow::anyhow;
use async_trait::async_trait;
use tokio::sync::oneshot;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;
use trzcina::Service;
use trzcina::ServiceManager;
use trzcina::ServiceShutdownOptions;
use trzcina::ServiceShutdownOutcome;

struct ErroringService;

#[async_trait]
impl Service for ErroringService {
    async fn run(self: Box<Self>, _cancellation_token: CancellationToken) -> Result<()> {
        Err(anyhow!("erroring service deliberately failed"))
    }
}

struct WaitingObserverService {
    observation_tx: oneshot::Sender<()>,
}

#[async_trait]
impl Service for WaitingObserverService {
    async fn run(self: Box<Self>, cancellation_token: CancellationToken) -> Result<()> {
        cancellation_token.cancelled().await;
        self.observation_tx.send(()).unwrap();
        Ok(())
    }
}

#[tokio::test]
async fn records_all_failures_when_multiple_services_error() {
    let mut manager = ServiceManager::default();
    for _ in 0..3 {
        manager.register_service(ErroringService);
    }

    let mut sibling_observation_receivers = Vec::new();
    for _ in 0..2 {
        let (observation_tx, observation_rx) = oneshot::channel::<()>();
        manager.register_service(WaitingObserverService { observation_tx });
        sibling_observation_receivers.push(observation_rx);
    }

    let report = timeout(
        Duration::from_secs(5),
        manager
            .start(CancellationToken::new())
            .run_to_completion(ServiceShutdownOptions::default()),
    )
    .await
    .unwrap();

    assert_eq!(report.outcomes().len(), 5);
    let errored_count = report
        .outcomes()
        .iter()
        .filter(|named_outcome| matches!(named_outcome.outcome, ServiceShutdownOutcome::Errored(_)))
        .count();
    let completed_count = report
        .outcomes()
        .iter()
        .filter(|named_outcome| matches!(named_outcome.outcome, ServiceShutdownOutcome::Completed))
        .count();
    assert_eq!(errored_count, 3);
    assert_eq!(completed_count, 2);
    for mut observation_rx in sibling_observation_receivers {
        observation_rx.try_recv().unwrap();
    }
    assert!(report.into_result().is_err());
}
