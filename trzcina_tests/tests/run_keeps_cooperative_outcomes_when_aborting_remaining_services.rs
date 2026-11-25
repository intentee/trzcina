use std::any::type_name;

use trzcina::service_shutdown_options::ServiceShutdownOptions;
use trzcina::service_shutdown_outcome::ServiceShutdownOutcome;
use trzcina::service_shutdown_outcome_with_service_name::ServiceShutdownOutcomeWithServiceName;
use trzcina_tests::cancellation_ignoring_service::CancellationIgnoringService;
use trzcina_tests::cooperative_shutdown::CooperativeShutdown;
use trzcina_tests::timed_run::TimedRun;

#[tokio::test(start_paused = true)]
async fn keeps_cooperative_outcomes_when_aborting_remaining_services() {
    let CooperativeShutdown {
        timed_shutdown: TimedRun {
            output: shutdown_report,
            ..
        },
        ..
    } = CooperativeShutdown::measure_alongside_aborted_service(ServiceShutdownOptions::default())
        .await;

    assert!(matches!(
        shutdown_report.outcomes(),
        [
            ServiceShutdownOutcomeWithServiceName {
                outcome: ServiceShutdownOutcome::Completed,
                ..
            },
            ServiceShutdownOutcomeWithServiceName {
                name,
                outcome: ServiceShutdownOutcome::AbortedByShutdownDeadline,
            },
        ] if *name == type_name::<CancellationIgnoringService<()>>()
    ));
}
