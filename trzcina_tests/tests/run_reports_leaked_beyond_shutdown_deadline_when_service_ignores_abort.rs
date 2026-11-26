use std::any::type_name;
use std::io;

use tokio::sync::oneshot;
use tokio::sync::oneshot::error::RecvError;
use tokio_util::sync::CancellationToken;

use trzcina::service_manager::ServiceManager;
use trzcina::service_shutdown_error::ServiceShutdownError;
use trzcina::service_shutdown_options::ServiceShutdownOptions;
use trzcina::service_shutdown_outcome_collection::ServiceShutdownOutcomeCollection;
use trzcina_tests::cancellation_ignoring_service::CancellationIgnoringService;
use trzcina_tests::install_trace_logging::install_trace_logging;
use trzcina_tests::observable_service::ObservableService;
use trzcina_tests::services_runtime::ServicesRuntime;

fn shutdown_result_after_sibling_stopped()
-> io::Result<Result<Result<(), Vec<ServiceShutdownError>>, RecvError>> {
    let (stop_observation_sender, stop_observation_receiver) = oneshot::channel::<()>();
    let mut service_manager = ServiceManager::default();

    service_manager.register_service(ObservableService {
        observation_sender: stop_observation_sender.into(),
    });
    service_manager.register_service(CancellationIgnoringService {
        held_until_abort: (),
    });

    let services_runtime = ServicesRuntime::start(service_manager, CancellationToken::new())?;
    let stop_observation = services_runtime.runtime.block_on(stop_observation_receiver);

    services_runtime
        .shut_down_while_undriven(|running_services| {
            running_services.run_to_completion(ServiceShutdownOptions::default())
        })
        .map(|shutdown_report| stop_observation.map(|()| shutdown_report.into_result()))
}

fn shutdown_result_of_abort_ignoring_service_alone()
-> io::Result<Result<(), Vec<ServiceShutdownError>>> {
    let cancellation_token = CancellationToken::new();
    let mut service_manager = ServiceManager::default();

    service_manager.register_service(CancellationIgnoringService {
        held_until_abort: (),
    });

    let services_runtime = ServicesRuntime::start(service_manager, cancellation_token.clone())?;

    cancellation_token.cancel();

    services_runtime
        .shut_down_while_undriven(|running_services| {
            running_services.run_to_completion(ServiceShutdownOptions::default())
        })
        .map(ServiceShutdownOutcomeCollection::into_result)
}

#[test]
fn reports_leaked_beyond_shutdown_deadline_when_service_ignores_abort() {
    assert!(install_trace_logging().is_ok());

    assert!(matches!(
        shutdown_result_after_sibling_stopped(),
        Ok(Ok(Err(failures))) if matches!(
            failures.as_slice(),
            [ServiceShutdownError::LeakedBeyondShutdownDeadline { service_name }]
                if *service_name == type_name::<CancellationIgnoringService<()>>()
        )
    ));
    assert!(matches!(
        shutdown_result_of_abort_ignoring_service_alone(),
        Ok(Err(failures)) if matches!(
            failures.as_slice(),
            [ServiceShutdownError::LeakedBeyondShutdownDeadline { service_name }]
                if *service_name == type_name::<CancellationIgnoringService<()>>()
        )
    ));
}
