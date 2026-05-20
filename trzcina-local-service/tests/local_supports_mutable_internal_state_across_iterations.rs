use std::collections::VecDeque;
use std::rc::Rc;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::Notify;
use tokio::sync::oneshot;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;
use trzcina_local_service::LocalService;
use trzcina_local_service::LocalServiceManager;
use trzcina_service::Manager;
use trzcina_service::RunToCompletionOptions;
use trzcina_service::RunningCollection;
use trzcina_service::ServiceShutdownOutcome;

struct StatefulService {
    iteration_count: usize,
    notify: Rc<Notify>,
    work_observers: VecDeque<oneshot::Sender<usize>>,
}

#[async_trait(?Send)]
impl LocalService for StatefulService {
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
async fn local_supports_mutable_internal_state_across_iterations() {
    let notify = Rc::new(Notify::new());
    let (first_work_tx, first_work_rx) = oneshot::channel::<usize>();
    let (second_work_tx, second_work_rx) = oneshot::channel::<usize>();
    let (third_work_tx, third_work_rx) = oneshot::channel::<usize>();
    let cancellation_token = CancellationToken::new();
    let cancellation_token_for_run = cancellation_token.clone();

    let mut manager = LocalServiceManager::default();
    manager.register_service(StatefulService {
        iteration_count: 0,
        notify: notify.clone(),
        work_observers: VecDeque::from(vec![first_work_tx, second_work_tx, third_work_tx]),
    });

    let run_future =
        manager
            .start(cancellation_token_for_run)
            .run_to_completion(RunToCompletionOptions {
                shutdown_deadline: Duration::from_secs(1),
            });
    let trigger_future = async move {
        let first_count = first_work_rx.await.unwrap();
        assert_eq!(first_count, 1);

        notify.notify_one();
        let second_count = second_work_rx.await.unwrap();
        assert_eq!(second_count, 2);

        notify.notify_one();
        let third_count = third_work_rx.await.unwrap();
        assert_eq!(third_count, 3);

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
