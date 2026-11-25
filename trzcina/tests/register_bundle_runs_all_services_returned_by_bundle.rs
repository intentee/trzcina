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

struct ObservableService {
    observation_tx: oneshot::Sender<()>,
}

#[async_trait]
impl Service for ObservableService {
    async fn run(self: Box<Self>, _cancellation_token: CancellationToken) -> Result<()> {
        self.observation_tx.send(()).unwrap();
        Ok(())
    }
}

struct SiblingsBundle {
    sibling_senders: Vec<oneshot::Sender<()>>,
}

#[async_trait]
impl ServiceBundle for SiblingsBundle {
    async fn services(self) -> Result<Vec<Box<dyn Service>>> {
        let services: Vec<Box<dyn Service>> = self
            .sibling_senders
            .into_iter()
            .map(|observation_tx| {
                Box::new(ObservableService { observation_tx }) as Box<dyn Service>
            })
            .collect();
        Ok(services)
    }
}

#[tokio::test]
async fn runs_all_services_returned_by_bundle() {
    let (first_tx, mut first_rx) = oneshot::channel::<()>();
    let (second_tx, mut second_rx) = oneshot::channel::<()>();

    let bundle = SiblingsBundle {
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
