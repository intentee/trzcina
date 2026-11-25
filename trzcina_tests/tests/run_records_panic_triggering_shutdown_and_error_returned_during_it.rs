use std::any::type_name;

use tokio_util::sync::CancellationToken;

use trzcina::service::Service;
use trzcina::service_manager::ServiceManager;
use trzcina::service_shutdown_error::ServiceShutdownError;
use trzcina_tests::panicking_service::PanickingService;
use trzcina_tests::run_with_default_shutdown::run_with_default_shutdown;
use trzcina_tests::test_failure::TestFailure;

const PANIC_MESSAGE: &str = "deliberately panicking to trigger shutdown";

struct ShutdownErroringService;

impl Service for ShutdownErroringService {
    type Error = TestFailure;

    async fn run(
        self: Box<Self>,
        cancellation_token: CancellationToken,
    ) -> Result<(), TestFailure> {
        cancellation_token.cancelled().await;

        Err(TestFailure::DeliberateServiceFailure)
    }
}

#[tokio::test]
async fn records_panic_triggering_shutdown_and_error_returned_during_it() {
    let mut manager = ServiceManager::default();

    manager.register_service(ShutdownErroringService);
    manager.register_service(PanickingService {
        panic_payload: Box::new(PANIC_MESSAGE),
    });

    let report = run_with_default_shutdown(manager).await;

    assert!(matches!(
        report.into_result().err().as_deref(),
        Some([
            ServiceShutdownError::Errored { service_name: erroring_service_name, source },
            ServiceShutdownError::Panicked { service_name: panicking_service_name, panic_message },
        ]) if *erroring_service_name == type_name::<ShutdownErroringService>()
            && TestFailure::DeliberateServiceFailure.is_erased_as(source.as_ref())
            && *panicking_service_name == type_name::<PanickingService>()
            && panic_message == PANIC_MESSAGE
    ));
}
