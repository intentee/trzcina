use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::Notify;
use tokio::sync::oneshot;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;
use trzcina_sendable_service::Service;
use trzcina_sendable_service::ServiceManager;
use trzcina_service::Manager;
use trzcina_service::RunToCompletionOptions;
use trzcina_service::RunningCollection;
use trzcina_service::ServiceShutdownOutcome;

const PRODUCED_VALUE: u32 = 42;

struct CoordinatingService {
    is_producer: bool,
    notify: Arc<Notify>,
    observation_tx: Option<oneshot::Sender<u32>>,
    shared_state: Arc<Mutex<Option<u32>>>,
}

#[async_trait]
impl Service for CoordinatingService {
    async fn run(&mut self, cancellation_token: CancellationToken) -> Result<()> {
        if self.is_producer {
            {
                let mut guard = self.shared_state.lock().unwrap();
                *guard = Some(PRODUCED_VALUE);
            }
            self.notify.notify_one();
            cancellation_token.cancelled().await;
            return Ok(());
        }
        tokio::select! {
            () = cancellation_token.cancelled() => return Ok(()),
            () = self.notify.notified() => {
                let observed_value = *self.shared_state.lock().unwrap();
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
async fn coordinates_via_shared_holder_between_two_services() {
    let shared_state: Arc<Mutex<Option<u32>>> = Arc::new(Mutex::new(None));
    let notify = Arc::new(Notify::new());
    let (observation_tx, observation_rx) = oneshot::channel::<u32>();
    let cancellation_token = CancellationToken::new();
    let cancellation_token_for_run = cancellation_token.clone();

    let mut manager = ServiceManager::default();
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

    let run_task = tokio::spawn(async move {
        manager
            .start(cancellation_token_for_run)
            .run_to_completion(RunToCompletionOptions {
                shutdown_deadline: Duration::from_secs(1),
            })
            .await
    });

    let observed = observation_rx.await.unwrap();
    assert_eq!(observed, PRODUCED_VALUE);

    cancellation_token.cancel();

    let report = timeout(Duration::from_secs(5), run_task)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(report.outcomes().len(), 2);
    for named_outcome in report.outcomes() {
        assert!(matches!(
            named_outcome.outcome,
            ServiceShutdownOutcome::Completed
        ));
    }
}
