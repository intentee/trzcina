use std::time::Duration;

use anyhow::Result;
use anyhow::anyhow;
use async_trait::async_trait;
use tokio::sync::oneshot;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;
use trzcina::LocalService;
use trzcina::LocalServiceManager;
use trzcina::ServiceShutdownOutcome;

struct ConfiguredService {
    return_err: bool,
    observation_tx: Option<oneshot::Sender<()>>,
}

#[async_trait(?Send)]
impl LocalService for ConfiguredService {
    async fn run(&mut self, cancellation_token: CancellationToken) -> Result<()> {
        if self.return_err {
            return Err(anyhow!("erroring service deliberately failed"));
        }
        cancellation_token.cancelled().await;
        if let Some(observation_tx) = self.observation_tx.take() {
            observation_tx.send(()).unwrap();
        }
        Ok(())
    }
}

#[tokio::test]
async fn local_records_service_error_and_cancels_siblings() {
    let mut manager = LocalServiceManager::default();
    manager.register_service(ConfiguredService {
        return_err: true,
        observation_tx: None,
    });

    let mut sibling_observation_receivers = Vec::new();
    for _ in 0..4 {
        let (observation_tx, observation_rx) = oneshot::channel::<()>();
        manager.register_service(ConfiguredService {
            return_err: false,
            observation_tx: Some(observation_tx),
        });
        sibling_observation_receivers.push(observation_rx);
    }

    let report = timeout(
        Duration::from_secs(5),
        manager
            .start_local(CancellationToken::new())
            .run_to_completion(Duration::from_secs(1)),
    )
    .await
    .unwrap();

    assert_eq!(report.outcomes().len(), 5);
    assert!(matches!(
        report.outcomes()[0].outcome,
        ServiceShutdownOutcome::Errored(_)
    ));
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
