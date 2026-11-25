use trzcina::service_manager::ServiceManager;
use trzcina::service_shutdown_outcome::ServiceShutdownOutcome;
use trzcina::service_shutdown_outcome_with_service_name::ServiceShutdownOutcomeWithServiceName;
use trzcina_tests::drop_panicking_value::DropPanickingValue;
use trzcina_tests::panicking_service::PanickingService;
use trzcina_tests::run_with_default_shutdown::run_with_default_shutdown;

const PAYLOAD_DROP_PANIC_MESSAGE: &str = "panic payload panicked while being dropped";

#[tokio::test]
async fn records_panic_escaping_service_panic_isolation() {
    let mut manager = ServiceManager::default();

    manager.register_service(PanickingService {
        panic_payload: Box::new(DropPanickingValue {
            panic_message: PAYLOAD_DROP_PANIC_MESSAGE,
        }),
    });

    let report = run_with_default_shutdown(manager).await;

    assert!(matches!(
        report.outcomes(),
        [ServiceShutdownOutcomeWithServiceName {
            outcome: ServiceShutdownOutcome::Panicked(panic_message),
            ..
        }] if panic_message == PAYLOAD_DROP_PANIC_MESSAGE
    ));
}
