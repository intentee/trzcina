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

const PANIC_MARKER: &str = "deliberately panicking for cascade test";

struct PanickingService;

#[async_trait]
impl Service for PanickingService {
    async fn run(self: Box<Self>, _cancellation_token: CancellationToken) -> Result<()> {
        panic!("{}", PANIC_MARKER);
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
async fn records_service_panic_and_cancels_siblings() {
    let mut manager = ServiceManager::default();
    manager.register_service(PanickingService);

    let mut sibling_observation_receivers = Vec::new();
    for _ in 0..4 {
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
    match &report.outcomes()[0].outcome {
        ServiceShutdownOutcome::Panicked(panic_message) => {
            assert!(panic_message.contains(PANIC_MARKER));
        }
        other_outcome => panic!("expected ServiceShutdownOutcome::Panicked, got {other_outcome:?}"),
    }
    for sibling_outcome in &report.outcomes()[1..] {
        assert!(matches!(
            sibling_outcome.outcome,
            ServiceShutdownOutcome::Completed
        ));
    }
    for mut observation_rx in sibling_observation_receivers {
        observation_rx.try_recv().unwrap();
    }
    assert!(report.into_result().is_err());
}
