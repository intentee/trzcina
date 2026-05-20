use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::oneshot;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;
use trzcina::Service;
use trzcina::ServiceBundle;
use trzcina::ServiceManager;
use trzcina::ServiceShutdownOptions;

struct BundleAndService {
    observation_tx: Option<oneshot::Sender<()>>,
    sibling_senders: Vec<oneshot::Sender<()>>,
}

#[async_trait]
impl Service for BundleAndService {
    async fn run(&mut self, _cancellation_token: CancellationToken) -> Result<()> {
        if let Some(observation_tx) = self.observation_tx.take() {
            observation_tx.send(()).unwrap();
        }
        Ok(())
    }
}

#[async_trait]
impl ServiceBundle for BundleAndService {
    async fn services(self) -> Result<Vec<Box<dyn Service>>> {
        let services: Vec<Box<dyn Service>> = self
            .sibling_senders
            .into_iter()
            .map(|observation_tx| {
                Box::new(BundleAndService {
                    observation_tx: Some(observation_tx),
                    sibling_senders: Vec::new(),
                }) as Box<dyn Service>
            })
            .collect();
        Ok(services)
    }
}

#[tokio::test]
async fn runs_all_services_returned_by_bundle() {
    let (first_tx, mut first_rx) = oneshot::channel::<()>();
    let (second_tx, mut second_rx) = oneshot::channel::<()>();

    let bundle = BundleAndService {
        observation_tx: None,
        sibling_senders: vec![first_tx, second_tx],
    };

    let mut manager = ServiceManager::default();
    manager.register_bundle(bundle).await.unwrap();

    timeout(
        Duration::from_secs(5),
        manager
            .start(CancellationToken::new())
            .run_to_completion(ServiceShutdownOptions::default()),
    )
    .await
    .unwrap()
    .into_result()
    .unwrap();

    first_rx.try_recv().unwrap();
    second_rx.try_recv().unwrap();
}
