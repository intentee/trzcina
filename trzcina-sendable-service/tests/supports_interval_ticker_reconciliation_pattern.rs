use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::oneshot;
use tokio::time::interval;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;
use trzcina_sendable_service::Service;
use trzcina_sendable_service::ServiceManager;
use trzcina_service::Manager;
use trzcina_service::RunToCompletionOptions;
use trzcina_service::RunningCollection;
use trzcina_service::ServiceShutdownOutcome;

struct ReconciliationService {
    first_tick_tx: Option<oneshot::Sender<()>>,
    tick_counter: Arc<AtomicUsize>,
}

#[async_trait]
impl Service for ReconciliationService {
    async fn run(&mut self, cancellation_token: CancellationToken) -> Result<()> {
        let mut ticker = interval(Duration::from_millis(10));
        loop {
            tokio::select! {
                () = cancellation_token.cancelled() => return Ok(()),
                _ = ticker.tick() => {
                    let previous = self.tick_counter.fetch_add(1, Ordering::SeqCst);
                    if previous == 0
                        && let Some(first_tick_tx) = self.first_tick_tx.take()
                    {
                        first_tick_tx.send(()).unwrap();
                    }
                }
            }
        }
    }
}

#[tokio::test]
async fn supports_interval_ticker_reconciliation_pattern() {
    let tick_counter = Arc::new(AtomicUsize::new(0));
    let cancellation_token = CancellationToken::new();
    let cancellation_token_for_run = cancellation_token.clone();
    let (first_tick_tx, first_tick_rx) = oneshot::channel::<()>();

    let mut manager = ServiceManager::default();
    manager.register_service(ReconciliationService {
        first_tick_tx: Some(first_tick_tx),
        tick_counter: tick_counter.clone(),
    });

    let run_task = tokio::spawn(async move {
        manager
            .start(cancellation_token_for_run)
            .run_to_completion(RunToCompletionOptions {
                shutdown_deadline: Duration::from_secs(1),
            })
            .await
    });

    first_tick_rx.await.unwrap();
    cancellation_token.cancel();

    let report = timeout(Duration::from_secs(5), run_task)
        .await
        .unwrap()
        .unwrap();

    assert!(
        tick_counter.load(Ordering::SeqCst) > 0,
        "ticker must have fired at least once"
    );
    assert_eq!(report.outcomes().len(), 1);
    assert!(matches!(
        report.outcomes()[0].outcome,
        ServiceShutdownOutcome::Completed
    ));
}
