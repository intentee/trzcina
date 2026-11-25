use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::oneshot;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;
use trzcina::ServiceManager;
use trzcina::ServiceShutdownOptions;
use trzcina::ServiceShutdownOutcome;
use trzcina::TickContext;
use trzcina::Ticker;

struct CountingTicker {
    first_tick_sender: Option<oneshot::Sender<()>>,
    tick_counter: Arc<AtomicUsize>,
}

#[async_trait]
impl Ticker for CountingTicker {
    fn tick_interval(&self) -> Duration {
        Duration::from_millis(10)
    }

    async fn handle_tick(
        &mut self,
        _cancellation_token: CancellationToken,
        _tick_context: TickContext,
    ) -> Result<()> {
        let previous_tick_count = self.tick_counter.fetch_add(1, Ordering::SeqCst);
        if previous_tick_count == 0
            && let Some(first_tick_sender) = self.first_tick_sender.take()
        {
            first_tick_sender.send(()).unwrap();
        }

        Ok(())
    }
}

#[tokio::test]
async fn ticker_runs_periodically_until_cancelled() {
    let tick_counter = Arc::new(AtomicUsize::new(0));
    let cancellation_token = CancellationToken::new();
    let cancellation_token_for_run = cancellation_token.clone();
    let (first_tick_sender, first_tick_receiver) = oneshot::channel::<()>();

    let mut manager = ServiceManager::default();
    manager.register_service(CountingTicker {
        first_tick_sender: Some(first_tick_sender),
        tick_counter: tick_counter.clone(),
    });

    let run_task = tokio::spawn(async move {
        manager
            .start(cancellation_token_for_run)
            .run_to_completion(ServiceShutdownOptions::default())
            .await
    });

    first_tick_receiver.await.unwrap();
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
