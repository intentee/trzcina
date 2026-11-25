use std::any::type_name;

use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;

use trzcina::service::Service;
use trzcina::service_manager::ServiceManager;
use trzcina::service_shutdown_error::ServiceShutdownError;
use trzcina::service_shutdown_outcome_collection::ServiceShutdownOutcomeCollection;
use trzcina_tests::erroring_service::ErroringService;
use trzcina_tests::install_trace_logging::install_trace_logging;
use trzcina_tests::observed_run::ObservedRun;
use trzcina_tests::panicking_service::PanickingService;
use trzcina_tests::release_awaiting_service::ReleaseAwaitingService;
use trzcina_tests::run_with_default_shutdown::run_with_default_shutdown;
use trzcina_tests::test_failure::TestFailure;
use trzcina_tests::waiting_observer_service::WaitingObserverService;

const PANIC_MESSAGE: &str = "deliberately panicking after a later registered service failed";

struct ReleaseAwaitingPanickingService {
    panicking_service: PanickingService,
    release_awaiting_service: ReleaseAwaitingService,
}

impl Service for ReleaseAwaitingPanickingService {
    type Error = TestFailure;

    async fn run(
        self: Box<Self>,
        cancellation_token: CancellationToken,
    ) -> Result<(), TestFailure> {
        let Self {
            panicking_service,
            release_awaiting_service,
        } = *self;

        Box::new(release_awaiting_service)
            .run(cancellation_token.clone())
            .await?;

        Box::new(panicking_service).run(cancellation_token).await
    }
}

#[tokio::test]
async fn records_every_service_failure_in_registration_order() {
    assert!(install_trace_logging().is_ok());

    let (cancellation_observation_sender, cancellation_observation_receiver) =
        oneshot::channel::<()>();
    let release = CancellationToken::new();
    let mut manager = ServiceManager::default();

    manager.register_service(ReleaseAwaitingPanickingService {
        panicking_service: PanickingService {
            panic_payload: Box::new(PANIC_MESSAGE),
        },
        release_awaiting_service: ReleaseAwaitingService {
            cancellation_observer: WaitingObserverService {
                observation_sender: cancellation_observation_sender.into(),
            },
            release: release.clone(),
        },
    });
    manager.register_service(ErroringService);

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
                ServiceShutdownError::Panicked { service_name: panicking_service_name, panic_message },
                ServiceShutdownError::Errored { service_name: erroring_service_name, source },
            ] if *panicking_service_name == type_name::<ReleaseAwaitingPanickingService>()
                && panic_message == PANIC_MESSAGE
                && *erroring_service_name == type_name::<ErroringService>()
                && TestFailure::DeliberateServiceFailure.is_erased_as(source.as_ref())
        )
    ));
}
