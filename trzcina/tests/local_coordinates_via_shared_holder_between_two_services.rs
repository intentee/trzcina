use std::cell::RefCell;
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

const PRODUCED_VALUE: u32 = 42;

struct CoordinatingService {
    is_producer: bool,
    notify: Rc<Notify>,
    observation_tx: Option<oneshot::Sender<u32>>,
    shared_state: Rc<RefCell<Option<u32>>>,
}

#[async_trait(?Send)]
impl LocalService for CoordinatingService {
    async fn run(&mut self, cancellation_token: CancellationToken) -> Result<()> {
        if self.is_producer {
            *self.shared_state.borrow_mut() = Some(PRODUCED_VALUE);
            self.notify.notify_one();
            cancellation_token.cancelled().await;
            return Ok(());
        }
        tokio::select! {
            () = cancellation_token.cancelled() => return Ok(()),
            () = self.notify.notified() => {
                let observed_value = *self.shared_state.borrow();
                if let Some(value) = observed_value
                    && let Some(observation_tx) = self.observation_tx.take()
                {
                    observation_tx.send(value).unwrap();
                }
            }
        }
        cancellation_token.cancelled().await;
        Ok(())
    }
}

#[tokio::test]
async fn local_coordinates_via_shared_holder_between_two_services() {
    let shared_state: Rc<RefCell<Option<u32>>> = Rc::new(RefCell::new(None));
    let notify = Rc::new(Notify::new());
    let (observation_tx, observation_rx) = oneshot::channel::<u32>();
    let cancellation_token = CancellationToken::new();
    let cancellation_token_for_run = cancellation_token.clone();

    let mut manager = LocalServiceManager::default();
    manager.register_service(CoordinatingService {
        is_producer: true,
        notify: notify.clone(),
        observation_tx: None,
        shared_state: shared_state.clone(),
    });
    manager.register_service(CoordinatingService {
        is_producer: false,
        notify: notify.clone(),
        observation_tx: Some(observation_tx),
        shared_state: shared_state.clone(),
    });

    let run_future = manager
        .start_local(cancellation_token_for_run)
        .run_to_completion(Duration::from_secs(1));
    let trigger_future = async move {
        let observed = observation_rx.await.unwrap();
        assert_eq!(observed, PRODUCED_VALUE);
        cancellation_token.cancel();
    };

    let (report, ()) = timeout(Duration::from_secs(5), async {
        tokio::join!(run_future, trigger_future)
    })
    .await
    .unwrap();

    assert_eq!(report.outcomes().len(), 2);
    for named_outcome in report.outcomes() {
        assert!(matches!(
            named_outcome.outcome,
            ServiceShutdownOutcome::Completed
        ));
    }
}
