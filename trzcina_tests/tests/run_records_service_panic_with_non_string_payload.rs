use std::any::type_name;

use trzcina::service_manager::ServiceManager;
use trzcina::service_shutdown_error::ServiceShutdownError;
use trzcina_tests::panicking_service::PanickingService;
use trzcina_tests::run_with_default_shutdown::run_with_default_shutdown;

#[tokio::test]
async fn records_service_panic_with_non_string_payload() {
    let mut manager = ServiceManager::default();

    manager.register_service(PanickingService {
        panic_payload: Box::new(()),
    });

    let report = run_with_default_shutdown(manager).await;

    assert!(matches!(
        report.into_result().err().as_deref(),
        Some([ServiceShutdownError::PanickedWithNonStringPayload { service_name }])
            if *service_name == type_name::<PanickingService>()
    ));
}
