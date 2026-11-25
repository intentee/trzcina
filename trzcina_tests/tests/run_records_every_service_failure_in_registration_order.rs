use std::any::type_name;

use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;

use trzcina::service::Service;
use trzcina::service_manager::ServiceManager;
use trzcina::service_shutdown_error::ServiceShutdownError;
use trzcina::service_shutdown_outcome_collection::ServiceShutdownOutcomeCollection;
use trzcina_tests::install_trace_logging::install_trace_logging;
use trzcina_tests::observed_run::ObservedRun;
use trzcina_tests::panicking_service::PanickingService;
use trzcina_tests::release_awaiting_service::ReleaseAwaitingService;
use trzcina_tests::run_with_default_shutdown::run_with_default_shutdown;
use trzcina_tests::test_failure::TestFailure;
use trzcina_tests::waiting_observer_service::WaitingObserverService;

const PANIC_MESSAGE: &str = "deliberately panicking before an earlier registered service fails";

struct ErroringService {
    release_awaiting_service: ReleaseAwaitingService,
}

impl Service for ErroringService {
    type Error = TestFailure;

    async fn run(
        self: Box<Self>,
        cancellation_token: CancellationToken,
    ) -> Result<(), TestFailure> {
        Box::new(self.release_awaiting_service)
            .run(cancellation_token)
            .await?;

        Err(TestFailure::DeliberateServiceFailure)
    }
}

#[tokio::test]
async fn records_every_service_failure_in_registration_order() {
    assert!(install_trace_logging().is_ok());

    let (cancellation_observation_sender, cancellation_observation_receiver) =
        oneshot::channel::<()>();
    let release = CancellationToken::new();
    let mut manager = ServiceManager::default();

    manager.register_service(ErroringService {
        release_awaiting_service: ReleaseAwaitingService {
            cancellation_observer: WaitingObserverService {
                observation_sender: cancellation_observation_sender.into(),
            },
            release: release.clone(),
        },
    });
    manager.register_service(PanickingService {
        panic_payload: Box::new(PANIC_MESSAGE),
    });

    let observed_run = ObservedRun::concurrently(run_with_default_shutdown(manager), async {
        let cancellation_observation = cancellation_observation_receiver.await;

        release.cancel();

        cancellation_observation
    })
    .await;

    assert!(matches!(
        observed_run
            .into_run_output()
            .map(ServiceShutdownOutcomeCollection::into_result),
        Ok(Err(failures)) if matches!(
            failures.as_slice(),
            [
                ServiceShutdownError::Errored { service_name: erroring_service_name, source },
                ServiceShutdownError::Panicked { service_name: panicking_service_name, .. },
            ] if *erroring_service_name == type_name::<ErroringService>()
                && TestFailure::DeliberateServiceFailure.is_erased_as(source.as_ref())
                && *panicking_service_name == type_name::<PanickingService>()
        )
    ));
}
