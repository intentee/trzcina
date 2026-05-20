use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::oneshot;
use tokio::task::yield_now;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;
use trzcina_local_service::LocalService;
use trzcina_local_service::LocalServiceManager;
use trzcina_service::Manager;
use trzcina_service::RunToCompletionOptions;
use trzcina_service::RunningCollection;
use trzcina_service::ServiceShutdownOutcome;

struct NonSendCounterService {
    counter: Rc<RefCell<usize>>,
    observation_tx: Option<oneshot::Sender<usize>>,
}

#[async_trait(?Send)]
impl LocalService for NonSendCounterService {
    async fn run(&mut self, _cancellation_token: CancellationToken) -> Result<()> {
        *self.counter.borrow_mut() += 1;
        yield_now().await;
        *self.counter.borrow_mut() += 1;
        if let Some(observation_tx) = self.observation_tx.take() {
            observation_tx.send(*self.counter.borrow()).unwrap();
        }
        Ok(())
    }
}

#[tokio::test]
async fn local_runs_non_send_service() {
    let counter: Rc<RefCell<usize>> = Rc::new(RefCell::new(0));
    let (observation_tx, mut observation_rx) = oneshot::channel::<usize>();

    let mut manager = LocalServiceManager::default();
    manager.register_service(NonSendCounterService {
        counter: counter.clone(),
        observation_tx: Some(observation_tx),
    });

    let report = timeout(
        Duration::from_secs(5),
        manager
            .start(CancellationToken::new())
            .run_to_completion(RunToCompletionOptions {
                shutdown_deadline: Duration::from_secs(1),
            }),
    )
    .await
    .unwrap();

    assert_eq!(report.outcomes().len(), 1);
    assert!(matches!(
        report.outcomes()[0].outcome,
        ServiceShutdownOutcome::Completed
    ));
    assert_eq!(observation_rx.try_recv().unwrap(), 2);
    assert_eq!(*counter.borrow(), 2);
}
