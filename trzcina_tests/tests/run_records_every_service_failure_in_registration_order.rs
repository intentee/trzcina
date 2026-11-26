use std::any::type_name;

use tokio::sync::oneshot::error::RecvError;

use trzcina::service::Service;
use trzcina::service_manager::ServiceManager;
use trzcina::service_shutdown_error::ServiceShutdownError;
use trzcina_tests::erroring_service::ErroringService;
use trzcina_tests::install_trace_logging::install_trace_logging;
use trzcina_tests::observed_run::ObservedRun;
use trzcina_tests::panicking_service::PanickingService;
use trzcina_tests::release_awaiting_control::ReleaseAwaitingControl;
use trzcina_tests::release_awaiting_service::ReleaseAwaitingService;
use trzcina_tests::test_failure::TestFailure;

const PANIC_MESSAGE: &str = "deliberately panicking to record its failure in registration order";

async fn failures_with_released_service_registered_before_shutdown_trigger<
    TReleasedService,
    TShutdownTrigger,
>(
    released_service: TReleasedService,
    shutdown_trigger: TShutdownTrigger,
) -> Result<Result<(), Vec<ServiceShutdownError>>, RecvError>
where
    TReleasedService: Service<Error = TestFailure>,
    TShutdownTrigger: Service,
{
    let mut manager = ServiceManager::default();
    let ReleaseAwaitingControl {
        cancellation_observation_receiver,
        release,
    } = ReleaseAwaitingControl::register(&mut manager, released_service);

    manager.register_service(shutdown_trigger);

    ObservedRun::shutdown_result_releasing_after_observation(
        manager,
        cancellation_observation_receiver,
        release,
    )
    .await
}

#[tokio::test]
async fn records_every_service_failure_in_registration_order() {
    assert!(install_trace_logging().is_ok());

    assert!(matches!(
        failures_with_released_service_registered_before_shutdown_trigger(
            PanickingService {
                panic_payload: Box::new(PANIC_MESSAGE),
            },
            ErroringService,
        )
        .await,
        Ok(Err(failures)) if matches!(
            failures.as_slice(),
            [
                ServiceShutdownError::Panicked { service_name: panicking_service_name, panic_message },
                ServiceShutdownError::Errored { service_name: erroring_service_name, source },
            ] if *panicking_service_name == type_name::<ReleaseAwaitingService<PanickingService>>()
                && panic_message == PANIC_MESSAGE
                && *erroring_service_name == type_name::<ErroringService>()
                && TestFailure::DeliberateServiceFailure.is_erased_as(source.as_ref())
        )
    ));
    assert!(matches!(
        failures_with_released_service_registered_before_shutdown_trigger(
            ErroringService,
            PanickingService {
                panic_payload: Box::new(PANIC_MESSAGE),
            },
        )
        .await,
        Ok(Err(failures)) if matches!(
            failures.as_slice(),
            [
                ServiceShutdownError::Errored { service_name: erroring_service_name, source },
                ServiceShutdownError::Panicked { service_name: panicking_service_name, panic_message },
            ] if *erroring_service_name == type_name::<ReleaseAwaitingService<ErroringService>>()
                && TestFailure::DeliberateServiceFailure.is_erased_as(source.as_ref())
                && *panicking_service_name == type_name::<PanickingService>()
                && panic_message == PANIC_MESSAGE
        )
    ));
}
