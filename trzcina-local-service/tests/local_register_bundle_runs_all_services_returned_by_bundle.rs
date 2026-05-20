use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::oneshot;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;
use trzcina_local_service::LocalService;
use trzcina_local_service::LocalServiceBundle;
use trzcina_local_service::LocalServiceManager;
use trzcina_service::Manager;
use trzcina_service::RunToCompletionOptions;
use trzcina_service::RunningCollection;

struct BundleAndService {
    observation_tx: Option<oneshot::Sender<()>>,
    sibling_senders: Vec<oneshot::Sender<()>>,
}

#[async_trait(?Send)]
impl LocalService for BundleAndService {
    async fn run(&mut self, _cancellation_token: CancellationToken) -> Result<()> {
        if let Some(observation_tx) = self.observation_tx.take() {
            observation_tx.send(()).unwrap();
        }
        Ok(())
    }
}

#[async_trait(?Send)]
impl LocalServiceBundle for BundleAndService {
    async fn services(self) -> Result<Vec<Box<dyn LocalService>>> {
        let services: Vec<Box<dyn LocalService>> = self
            .sibling_senders
            .into_iter()
            .map(|observation_tx| {
                Box::new(BundleAndService {
                    observation_tx: Some(observation_tx),
                    sibling_senders: Vec::new(),
                }) as Box<dyn LocalService>
            })
            .collect();
        Ok(services)
    }
}

#[tokio::test]
async fn local_runs_all_services_returned_by_bundle() {
    let (first_tx, mut first_rx) = oneshot::channel::<()>();
    let (second_tx, mut second_rx) = oneshot::channel::<()>();

    let bundle = BundleAndService {
        observation_tx: None,
        sibling_senders: vec![first_tx, second_tx],
    };

    let mut manager = LocalServiceManager::default();
    manager.register_bundle(bundle).await.unwrap();

    timeout(
        Duration::from_secs(5),
        manager
            .start(CancellationToken::new())
            .run_to_completion(RunToCompletionOptions {
                shutdown_deadline: Duration::from_secs(1),
            }),
    )
    .await
    .unwrap()
    .into_result()
    .unwrap();

    first_rx.try_recv().unwrap();
    second_rx.try_recv().unwrap();
}
