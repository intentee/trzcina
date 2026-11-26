use std::any::type_name;

use trzcina::service_manager::ServiceManager;
use trzcina::service_shutdown_options::ServiceShutdownOptions;
use trzcina::service_shutdown_outcome::ServiceShutdownOutcome;
use trzcina::service_shutdown_outcome_with_service_name::ServiceShutdownOutcomeWithServiceName;
use trzcina_tests::cancellation_ignoring_service::CancellationIgnoringService;
use trzcina_tests::observed_run::ObservedRun;
use trzcina_tests::waiting_observers::WaitingObservers;

const COOPERATIVE_SERVICES: usize = 1;

#[tokio::test(start_paused = true)]
async fn keeps_cooperative_outcomes_when_aborting_remaining_services() {
    let mut manager = ServiceManager::default();

    manager.register_service(CancellationIgnoringService {
        held_until_abort: (),
    });

    let _cooperative_observers = WaitingObservers::register(&mut manager, COOPERATIVE_SERVICES);

    manager.register_service(CancellationIgnoringService {
        held_until_abort: (),
    });

    let ObservedRun { run_output, .. } = ObservedRun::shutdown_after_observation(
        manager,
        ServiceShutdownOptions::default(),
        async {},
    )
    .await;

    assert!(matches!(
        run_output.outcomes(),
        [
            ServiceShutdownOutcomeWithServiceName {
                name: first_name,
                outcome: ServiceShutdownOutcome::AbortedByShutdownDeadline,
            },
            ServiceShutdownOutcomeWithServiceName {
                outcome: ServiceShutdownOutcome::Completed,
                ..
            },
            ServiceShutdownOutcomeWithServiceName {
                name: last_name,
                outcome: ServiceShutdownOutcome::AbortedByShutdownDeadline,
            },
        ] if *first_name == type_name::<CancellationIgnoringService<()>>()
            && *last_name == type_name::<CancellationIgnoringService<()>>()
    ));
}
