use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;

use trzcina::service::Service;
use trzcina::service_manager::ServiceManager;
use trzcina_tests::observed_run::ObservedRun;
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

    assert!(matches!(
        ObservedRun::shutdown_result_releasing_after_observation(
            manager,
            sibling_observation_receiver,
            release,
        )
        .await,
        Ok(Ok(()))
    ));
}
