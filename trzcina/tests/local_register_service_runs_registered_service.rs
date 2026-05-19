use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::oneshot;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;
use trzcina::LocalService;
use trzcina::LocalServiceManager;

struct ObservableService {
    observation_tx: Option<oneshot::Sender<()>>,
}

#[async_trait(?Send)]
impl LocalService for ObservableService {
    async fn run(&mut self, _cancellation_token: CancellationToken) -> Result<()> {
        if let Some(observation_tx) = self.observation_tx.take() {
            observation_tx.send(()).unwrap();
        }
        Ok(())
    }
}

#[tokio::test]
async fn local_runs_registered_service() {
    let (observation_tx, mut observation_rx) = oneshot::channel::<()>();

    let mut manager = LocalServiceManager::default();
    manager.register_service(ObservableService {
        observation_tx: Some(observation_tx),
    });

    timeout(
        Duration::from_secs(5),
        manager
            .start_local(CancellationToken::new())
            .run_to_completion(Duration::from_secs(1)),
    )
    .await
    .unwrap()
    .into_result()
    .unwrap();

    observation_rx.try_recv().unwrap();
}
