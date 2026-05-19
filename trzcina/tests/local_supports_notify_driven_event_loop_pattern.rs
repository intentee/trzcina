use std::collections::VecDeque;
use std::rc::Rc;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::Notify;
use tokio::sync::oneshot;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;
use trzcina::LocalService;
use trzcina::LocalServiceManager;
use trzcina::ServiceShutdownOutcome;

struct NotifyDrivenService {
    notify: Rc<Notify>,
    work_observers: VecDeque<oneshot::Sender<()>>,
}

#[async_trait(?Send)]
impl LocalService for NotifyDrivenService {
    async fn run(&mut self, cancellation_token: CancellationToken) -> Result<()> {
        loop {
            if let Some(work_observer) = self.work_observers.pop_front() {
                work_observer.send(()).unwrap();
            }
            tokio::select! {
                () = cancellation_token.cancelled() => return Ok(()),
                () = self.notify.notified() => continue,
            }
        }
    }
}

#[tokio::test]
async fn local_supports_notify_driven_event_loop_pattern() {
    let notify = Rc::new(Notify::new());
    let (first_work_tx, first_work_rx) = oneshot::channel::<()>();
    let (second_work_tx, second_work_rx) = oneshot::channel::<()>();
    let (third_work_tx, third_work_rx) = oneshot::channel::<()>();
    let cancellation_token = CancellationToken::new();
    let cancellation_token_for_run = cancellation_token.clone();

    let mut manager = LocalServiceManager::default();
    manager.register_service(NotifyDrivenService {
        notify: notify.clone(),
        work_observers: VecDeque::from(vec![first_work_tx, second_work_tx, third_work_tx]),
    });

    let run_future = manager
        .start_local(cancellation_token_for_run)
        .run_to_completion(Duration::from_secs(1));
    let trigger_future = async move {
        first_work_rx.await.unwrap();
        notify.notify_one();
        second_work_rx.await.unwrap();
        notify.notify_one();
        third_work_rx.await.unwrap();
        cancellation_token.cancel();
    };

    let (report, ()) = timeout(Duration::from_secs(5), async {
        tokio::join!(run_future, trigger_future)
    })
    .await
    .unwrap();

    assert_eq!(report.outcomes().len(), 1);
    assert!(matches!(
        report.outcomes()[0].outcome,
        ServiceShutdownOutcome::Completed
    ));
}
