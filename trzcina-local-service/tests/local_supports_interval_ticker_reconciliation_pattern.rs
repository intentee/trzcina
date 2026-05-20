use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::oneshot;
use tokio::time::interval;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;
use trzcina_local_service::LocalService;
use trzcina_local_service::LocalServiceManager;
use trzcina_service::Manager;
use trzcina_service::RunToCompletionOptions;
use trzcina_service::RunningCollection;
use trzcina_service::ServiceShutdownOutcome;

struct ReconciliationService {
    first_tick_tx: Option<oneshot::Sender<()>>,
    tick_counter: Rc<Cell<usize>>,
}

#[async_trait(?Send)]
impl LocalService for ReconciliationService {
    async fn run(&mut self, cancellation_token: CancellationToken) -> Result<()> {
        let mut ticker = interval(Duration::from_millis(10));
        loop {
            tokio::select! {
                () = cancellation_token.cancelled() => return Ok(()),
                _ = ticker.tick() => {
                    let previous = self.tick_counter.get();
                    self.tick_counter.set(previous + 1);
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
async fn local_supports_interval_ticker_reconciliation_pattern() {
    let tick_counter: Rc<Cell<usize>> = Rc::new(Cell::new(0));
    let cancellation_token = CancellationToken::new();
    let cancellation_token_for_run = cancellation_token.clone();
    let (first_tick_tx, first_tick_rx) = oneshot::channel::<()>();

    let mut manager = LocalServiceManager::default();
    manager.register_service(ReconciliationService {
        first_tick_tx: Some(first_tick_tx),
        tick_counter: tick_counter.clone(),
    });

    let run_future =
        manager
            .start(cancellation_token_for_run)
            .run_to_completion(RunToCompletionOptions {
                shutdown_deadline: Duration::from_secs(1),
            });
    let trigger_future = async move {
        first_tick_rx.await.unwrap();
        cancellation_token.cancel();
    };

    let (report, ()) = timeout(Duration::from_secs(5), async {
        tokio::join!(run_future, trigger_future)
    })
    .await
    .unwrap();

    assert!(
        tick_counter.get() > 0,
        "ticker must have fired at least once"
    );
    assert_eq!(report.outcomes().len(), 1);
    assert!(matches!(
        report.outcomes()[0].outcome,
        ServiceShutdownOutcome::Completed
    ));
}
