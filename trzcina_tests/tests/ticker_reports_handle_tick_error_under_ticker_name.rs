use std::any::type_name;

use trzcina::service_manager::ServiceManager;
use trzcina::service_shutdown_error::ServiceShutdownError;
use trzcina_tests::failing_ticker::FailingTicker;
use trzcina_tests::run_with_default_shutdown::run_with_default_shutdown;
use trzcina_tests::test_failure::TestFailure;
use trzcina_ticker::tick_time_limit::TickTimeLimit;
use trzcina_ticker::ticker_service::TickerService;

#[tokio::test(start_paused = true)]
async fn reports_handle_tick_error_under_ticker_name() {
    let mut manager = ServiceManager::default();

    manager.register_service(TickerService {
        ticker: FailingTicker {
            failing_tick: 1,
            tick_time_limit: TickTimeLimit::Unlimited,
        },
    });

    let report = run_with_default_shutdown(manager).await;

    assert!(matches!(
        report.into_result().err().as_deref(),
        Some([ServiceShutdownError::Errored { service_name, source }])
            if *service_name == type_name::<FailingTicker>()
                && TestFailure::DeliberateTickFailure.is_erased_as(source.as_ref())
    ));
}
