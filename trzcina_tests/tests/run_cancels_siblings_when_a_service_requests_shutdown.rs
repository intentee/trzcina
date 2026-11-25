use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;

use trzcina::service::Service;
use trzcina::service_manager::ServiceManager;
use trzcina::service_shutdown_outcome_collection::ServiceShutdownOutcomeCollection;
use trzcina_tests::observed_run::ObservedRun;
use trzcina_tests::run_with_default_shutdown::run_with_default_shutdown;
use trzcina_tests::test_failure::TestFailure;
use trzcina_tests::waiting_observer_service::WaitingObserverService;

struct ShutdownRequestingService {
    release: CancellationToken,
}

impl Service for ShutdownRequestingService {
    type Error = TestFailure;

    async fn run(
        self: Box<Self>,
        cancellation_token: CancellationToken,
    ) -> Result<(), TestFailure> {
        cancellation_token.cancel();
        self.release.cancelled().await;

        Ok(())
    }
}

#[tokio::test]
async fn cancels_siblings_when_a_service_requests_shutdown() {
    let (sibling_observation_sender, sibling_observation_receiver) = oneshot::channel::<()>();
    let release = CancellationToken::new();
    let mut manager = ServiceManager::default();

    manager.register_service(ShutdownRequestingService {
        release: release.clone(),
    });
    manager.register_service(WaitingObserverService {
        observation_sender: sibling_observation_sender.into(),
    });

    let observed_run = ObservedRun::concurrently(run_with_default_shutdown(manager), async {
        let sibling_observation = sibling_observation_receiver.await;

        release.cancel();

        sibling_observation
    })
    .await;

    assert!(matches!(
        observed_run
            .into_run_output()
            .map(ServiceShutdownOutcomeCollection::into_result),
        Ok(Ok(()))
    ));
}
