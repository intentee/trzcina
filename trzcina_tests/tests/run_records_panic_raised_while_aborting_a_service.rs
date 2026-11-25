use std::any::type_name;

use trzcina::service_manager::ServiceManager;
use trzcina::service_shutdown_error::ServiceShutdownError;
use trzcina::service_shutdown_options::ServiceShutdownOptions;
use trzcina_tests::cancellation_ignoring_service::CancellationIgnoringService;
use trzcina_tests::drop_panicking_value::DropPanickingValue;
use trzcina_tests::observed_run::ObservedRun;

const ABORT_PANIC_MESSAGE: &str = "held value panicked while the aborted service was dropped";

#[tokio::test(start_paused = true)]
async fn records_panic_raised_while_aborting_a_service() {
    let mut manager = ServiceManager::default();

    manager.register_service(CancellationIgnoringService {
        held_until_abort: DropPanickingValue {
            panic_message: ABORT_PANIC_MESSAGE,
        },
    });

    let observed_run = ObservedRun::shutdown_after_observation(
        manager,
        ServiceShutdownOptions::default(),
        async {},
    )
    .await;

    assert!(matches!(
        observed_run.run_output.into_result().err().as_deref(),
        Some([ServiceShutdownError::Panicked { service_name, panic_message }])
            if *service_name == type_name::<CancellationIgnoringService<DropPanickingValue>>()
                && panic_message == ABORT_PANIC_MESSAGE
    ));
}
