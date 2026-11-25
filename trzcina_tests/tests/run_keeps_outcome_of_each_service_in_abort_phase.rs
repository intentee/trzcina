use std::time::Duration;

use trzcina::service_manager::ServiceManager;
use trzcina::service_shutdown_options::ServiceShutdownOptions;
use trzcina::service_shutdown_outcome::ServiceShutdownOutcome;
use trzcina::service_shutdown_outcome_collection::ServiceShutdownOutcomeCollection;
use trzcina::service_shutdown_outcome_with_service_name::ServiceShutdownOutcomeWithServiceName;
use trzcina_tests::cancellation_ignoring_service::CancellationIgnoringService;
use trzcina_tests::observed_run::ObservedRun;
use trzcina_tests::thread_blocking_control::ThreadBlockingControl;

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn keeps_outcome_of_each_service_in_abort_phase() {
    let mut manager = ServiceManager::default();
    let ThreadBlockingControl {
        release_sender: release_on_abort,
        started_receiver: released_on_abort_started,
    } = ThreadBlockingControl::register(&mut manager);

    manager.register_service(CancellationIgnoringService {
        held_until_abort: release_on_abort,
    });

    let ThreadBlockingControl {
        release_sender: release_after_shutdown,
        started_receiver: leaking_started,
    } = ThreadBlockingControl::register(&mut manager);
    let observed_run = ObservedRun::shutdown_after_observation(
        manager,
        ServiceShutdownOptions {
            cooperative_deadline: Duration::ZERO,
            ..ServiceShutdownOptions::default()
        },
        async {
            released_on_abort_started.await?;
            leaking_started.await
        },
    )
    .await;

    drop(release_after_shutdown);

    assert!(matches!(
        observed_run
            .into_run_output()
            .as_ref()
            .map(ServiceShutdownOutcomeCollection::outcomes),
        Ok([
            ServiceShutdownOutcomeWithServiceName {
                outcome: ServiceShutdownOutcome::Completed,
                ..
            },
            ServiceShutdownOutcomeWithServiceName {
                outcome: ServiceShutdownOutcome::AbortedByShutdownDeadline,
                ..
            },
            ServiceShutdownOutcomeWithServiceName {
                outcome: ServiceShutdownOutcome::LeakedBeyondShutdownDeadline,
                ..
            },
        ])
    ));
}
