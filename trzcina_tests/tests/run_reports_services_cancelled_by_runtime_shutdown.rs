use std::any::type_name;
use std::io;
use std::time::Duration;

use tokio_util::sync::CancellationToken;

use trzcina::service_manager::ServiceManager;
use trzcina::service_shutdown_error::ServiceShutdownError;
use trzcina::service_shutdown_options::ServiceShutdownOptions;
use trzcina::service_shutdown_outcome_collection::ServiceShutdownOutcomeCollection;
use trzcina_tests::block_on_paused_runtime::block_on_paused_runtime;
use trzcina_tests::cancellation_ignoring_service::CancellationIgnoringService;
use trzcina_tests::services_runtime::ServicesRuntime;

fn shut_down_after_services_runtime_shutdown(
    service_manager: ServiceManager,
) -> io::Result<ServiceShutdownOutcomeCollection> {
    let ServicesRuntime {
        running_services,
        runtime,
    } = ServicesRuntime::start(service_manager, CancellationToken::new())?;

    drop(runtime);

    block_on_paused_runtime(running_services.run_to_completion(ServiceShutdownOptions {
        cooperative_deadline: Duration::ZERO,
        ..ServiceShutdownOptions::default()
    }))
}

#[test]
fn reports_services_cancelled_by_runtime_shutdown() {
    let mut manager = ServiceManager::default();

    CancellationIgnoringService::register(&mut manager, 2);

    assert!(matches!(
        shut_down_after_services_runtime_shutdown(manager)
            .map(ServiceShutdownOutcomeCollection::into_result),
        Ok(Err(failures)) if matches!(
            failures.as_slice(),
            [
                ServiceShutdownError::CancelledByRuntimeShutdown { service_name: first_service_name },
                ServiceShutdownError::CancelledByRuntimeShutdown { service_name: second_service_name },
            ] if *first_service_name == type_name::<CancellationIgnoringService<()>>()
                && *second_service_name == type_name::<CancellationIgnoringService<()>>()
        )
    ));
}
