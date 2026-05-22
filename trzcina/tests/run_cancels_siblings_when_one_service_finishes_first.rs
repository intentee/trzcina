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

struct ConfiguredService {
    finish_immediately: bool,
    observation_tx: Option<oneshot::Sender<()>>,
}

#[async_trait]
impl Service for ConfiguredService {
    async fn run(&mut self, cancellation_token: CancellationToken) -> Result<()> {
        if self.finish_immediately {
            return Ok(());
        }
        cancellation_token.cancelled().await;
        if let Some(observation_tx) = self.observation_tx.take() {
            observation_tx.send(()).unwrap();
        }
        Ok(())
    }
}

#[tokio::test]
async fn cancels_siblings_when_one_service_finishes_first() {
    let mut manager = ServiceManager::default();
    manager.register_service(ConfiguredService {
        finish_immediately: true,
        observation_tx: None,
    });

    let mut sibling_observation_receivers = Vec::new();
    for _ in 0..4 {
        let (observation_tx, observation_rx) = oneshot::channel::<()>();
        manager.register_service(ConfiguredService {
            finish_immediately: false,
            observation_tx: Some(observation_tx),
        });
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
    for named_outcome in report.outcomes() {
        assert!(matches!(
            named_outcome.outcome,
            ServiceShutdownOutcome::Completed
        ));
    }
    for mut observation_rx in sibling_observation_receivers {
        observation_rx.try_recv().unwrap();
    }
}
