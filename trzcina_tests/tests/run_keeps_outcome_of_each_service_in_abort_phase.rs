use std::time::Duration;

use trzcina::service_manager::ServiceManager;
use trzcina::service_shutdown_options::ServiceShutdownOptions;
use trzcina::service_shutdown_outcome::ServiceShutdownOutcome;
use trzcina::service_shutdown_outcome_collection::ServiceShutdownOutcomeCollection;
use trzcina::service_shutdown_outcome_with_service_name::ServiceShutdownOutcomeWithServiceName;
use trzcina_tests::cancellation_ignoring_service::CancellationIgnoringService;
use trzcina_tests::completing_service::CompletingService;
use trzcina_tests::erroring_service::ErroringService;
use trzcina_tests::observed_run::ObservedRun;
use trzcina_tests::panicking_service::PanickingService;
use trzcina_tests::test_failure::TestFailure;
use trzcina_tests::thread_blocking_control::ThreadBlockingControl;
use trzcina_tests::timed_run::TimedRun;

const PANIC_MESSAGE: &str = "deliberately panicking once released in the abort phase";

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn keeps_outcome_of_each_service_in_abort_phase() {
    let mut manager = ServiceManager::default();
    let ThreadBlockingControl {
        release_sender: completing_release,
        started_receiver: completing_started,
    } = ThreadBlockingControl::register(&mut manager, CompletingService);
    let ThreadBlockingControl {
        release_sender: erroring_release,
        started_receiver: erroring_started,
    } = ThreadBlockingControl::register(&mut manager, ErroringService);
    let ThreadBlockingControl {
        release_sender: panicking_release,
        started_receiver: panicking_started,
    } = ThreadBlockingControl::register(
        &mut manager,
        PanickingService {
            panic_payload: Box::new(PANIC_MESSAGE),
        },
    );
    let ThreadBlockingControl {
        release_sender: non_string_panicking_release,
        started_receiver: non_string_panicking_started,
    } = ThreadBlockingControl::register(
        &mut manager,
        PanickingService {
            panic_payload: Box::new(()),
        },
    );

    manager.register_service(CancellationIgnoringService {
        held_until_abort: vec![
            completing_release,
            erroring_release,
            panicking_release,
            non_string_panicking_release,
        ],
    });

    let ThreadBlockingControl {
        release_sender: release_after_shutdown,
        started_receiver: leaking_started,
    } = ThreadBlockingControl::register(&mut manager, CompletingService);
    let shutdown_options = ServiceShutdownOptions {
        cooperative_deadline: Duration::ZERO,
        ..ServiceShutdownOptions::default()
    };
    let TimedRun {
        elapsed,
        output: observed_run,
    } = TimedRun::measure(ObservedRun::shutdown_after_observation(
        manager,
        shutdown_options.clone(),
        async {
            completing_started.await?;
            erroring_started.await?;
            panicking_started.await?;
            non_string_panicking_started.await?;
            leaking_started.await
        },
    ))
    .await;

    drop(release_after_shutdown);

    assert!(elapsed >= shutdown_options.abort_deadline);

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
                outcome: ServiceShutdownOutcome::Errored(source),
                ..
            },
            ServiceShutdownOutcomeWithServiceName {
                outcome: ServiceShutdownOutcome::Panicked(panic_message),
                ..
            },
            ServiceShutdownOutcomeWithServiceName {
                outcome: ServiceShutdownOutcome::PanickedWithNonStringPayload,
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
        ]) if TestFailure::DeliberateServiceFailure.is_erased_as(source.as_ref())
            && panic_message == PANIC_MESSAGE
    ));
}
