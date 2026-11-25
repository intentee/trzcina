use trzcina::service_shutdown_options::ServiceShutdownOptions;
use trzcina_tests::cooperative_shutdown::CooperativeShutdown;
use trzcina_tests::timed_run::TimedRun;

#[tokio::test(start_paused = true)]
async fn ends_abort_phase_once_aborted_services_stop() {
    let ServiceShutdownOptions {
        cooperative_deadline,
        ..
    } = ServiceShutdownOptions::default();
    let CooperativeShutdown {
        timed_shutdown: TimedRun { elapsed, .. },
        ..
    } = CooperativeShutdown::measure_alongside_aborted_service(ServiceShutdownOptions {
        abort_deadline: cooperative_deadline * 2,
        cooperative_deadline,
    })
    .await;

    assert_eq!(elapsed, cooperative_deadline);
}
