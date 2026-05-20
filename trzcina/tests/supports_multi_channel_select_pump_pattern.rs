use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;
use trzcina::Service;
use trzcina::ServiceManager;
use trzcina::ServiceShutdownOptions;
use trzcina::ServiceShutdownOutcome;

struct MultiChannelPumpService {
    primary_observed_tx: Option<oneshot::Sender<()>>,
    primary_rx: mpsc::Receiver<()>,
    secondary_observed_tx: Option<oneshot::Sender<()>>,
    secondary_rx: mpsc::Receiver<()>,
}

#[async_trait]
impl Service for MultiChannelPumpService {
    async fn run(&mut self, cancellation_token: CancellationToken) -> Result<()> {
        loop {
            tokio::select! {
                () = cancellation_token.cancelled() => return Ok(()),
                Some(()) = self.primary_rx.recv() => {
                    if let Some(primary_observed_tx) = self.primary_observed_tx.take() {
                        primary_observed_tx.send(()).unwrap();
                    }
                }
                Some(()) = self.secondary_rx.recv() => {
                    if let Some(secondary_observed_tx) = self.secondary_observed_tx.take() {
                        secondary_observed_tx.send(()).unwrap();
                    }
                }
            }
        }
    }
}

#[tokio::test]
async fn supports_multi_channel_select_pump_pattern() {
    let (primary_tx, primary_rx) = mpsc::channel::<()>(1);
    let (secondary_tx, secondary_rx) = mpsc::channel::<()>(1);
    let (primary_observed_tx, primary_observed_rx) = oneshot::channel::<()>();
    let (secondary_observed_tx, secondary_observed_rx) = oneshot::channel::<()>();
    let cancellation_token = CancellationToken::new();
    let cancellation_token_for_run = cancellation_token.clone();

    let mut manager = ServiceManager::default();
    manager.register_service(MultiChannelPumpService {
        primary_observed_tx: Some(primary_observed_tx),
        primary_rx,
        secondary_observed_tx: Some(secondary_observed_tx),
        secondary_rx,
    });

    let run_task = tokio::spawn(async move {
        manager
            .start(cancellation_token_for_run)
            .run_to_completion(ServiceShutdownOptions::default())
            .await
    });

    primary_tx.send(()).await.unwrap();
    secondary_tx.send(()).await.unwrap();

    primary_observed_rx.await.unwrap();
    secondary_observed_rx.await.unwrap();

    cancellation_token.cancel();

    let report = timeout(Duration::from_secs(5), run_task)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(report.outcomes().len(), 1);
    assert!(matches!(
        report.outcomes()[0].outcome,
        ServiceShutdownOutcome::Completed
    ));
}
