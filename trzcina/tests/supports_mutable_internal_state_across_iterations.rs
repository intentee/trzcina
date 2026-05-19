use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use trzcina::Service;
use trzcina::ServiceManager;
use trzcina::ServiceShutdownOutcome;
use tokio::sync::Notify;
use tokio::sync::oneshot;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;

struct StatefulService {
    iteration_count: usize,
    notify: Arc<Notify>,
    work_observers: VecDeque<oneshot::Sender<usize>>,
}

#[async_trait]
impl Service for StatefulService {
    async fn run(&mut self, cancellation_token: CancellationToken) -> Result<()> {
        loop {
            self.iteration_count += 1;
            if let Some(work_observer) = self.work_observers.pop_front() {
                work_observer.send(self.iteration_count).unwrap();
            }
            tokio::select! {
                () = cancellation_token.cancelled() => return Ok(()),
                () = self.notify.notified() => continue,
            }
        }
    }
}

#[tokio::test]
async fn supports_mutable_internal_state_across_iterations() {
    let notify = Arc::new(Notify::new());
    let (first_work_tx, first_work_rx) = oneshot::channel::<usize>();
    let (second_work_tx, second_work_rx) = oneshot::channel::<usize>();
    let (third_work_tx, third_work_rx) = oneshot::channel::<usize>();
    let cancellation_token = CancellationToken::new();
    let cancellation_token_for_run = cancellation_token.clone();

    let mut manager = ServiceManager::default();
    manager.register_service(StatefulService {
        iteration_count: 0,
        notify: notify.clone(),
        work_observers: VecDeque::from(vec![first_work_tx, second_work_tx, third_work_tx]),
    });

    let run_task = tokio::spawn(async move {
        manager
            .start(cancellation_token_for_run)
            .run_to_completion(Duration::from_secs(1))
            .await
    });

    let first_count = first_work_rx.await.unwrap();
    assert_eq!(first_count, 1);

    notify.notify_one();
    let second_count = second_work_rx.await.unwrap();
    assert_eq!(second_count, 2);

    notify.notify_one();
    let third_count = third_work_rx.await.unwrap();
    assert_eq!(third_count, 3);

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
