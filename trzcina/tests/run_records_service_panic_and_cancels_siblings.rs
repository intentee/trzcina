use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use trzcina::Service;
use trzcina::ServiceManager;
use trzcina::ServiceShutdownOutcome;
use tokio::sync::oneshot;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;

const PANIC_MARKER: &str = "deliberately panicking for cascade test";

struct ConfiguredService {
    should_panic: bool,
    observation_tx: Option<oneshot::Sender<()>>,
}

#[async_trait]
impl Service for ConfiguredService {
    async fn run(&mut self, cancellation_token: CancellationToken) -> Result<()> {
        if self.should_panic {
            panic!("{}", PANIC_MARKER);
        }
        cancellation_token.cancelled().await;
        if let Some(observation_tx) = self.observation_tx.take() {
            observation_tx.send(()).unwrap();
        }
        Ok(())
    }
}

#[tokio::test]
async fn records_service_panic_and_cancels_siblings() {
    let mut manager = ServiceManager::default();
    manager.register_service(ConfiguredService {
        should_panic: true,
        observation_tx: None,
    });

    let mut sibling_observation_receivers = Vec::new();
    for _ in 0..4 {
        let (observation_tx, observation_rx) = oneshot::channel::<()>();
        manager.register_service(ConfiguredService {
            should_panic: false,
            observation_tx: Some(observation_tx),
        });
        sibling_observation_receivers.push(observation_rx);
    }

    let report = timeout(
        Duration::from_secs(5),
        manager
            .start(CancellationToken::new())
            .run_to_completion(Duration::from_secs(1)),
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
