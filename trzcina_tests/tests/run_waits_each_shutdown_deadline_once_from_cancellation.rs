use std::io;
use std::time::Duration;

use tokio::time::advance;
use tokio_util::sync::CancellationToken;

use trzcina::service_manager::ServiceManager;
use trzcina::service_shutdown_options::ServiceShutdownOptions;
use trzcina::service_shutdown_outcome_collection::ServiceShutdownOutcomeCollection;
use trzcina_tests::cancellation_ignoring_service::CancellationIgnoringService;
use trzcina_tests::observed_run::ObservedRun;
use trzcina_tests::services_runtime::ServicesRuntime;
use trzcina_tests::timed_run::TimedRun;

fn shut_down_services_ignoring_abort_on_undriven_runtime(
    shutdown_options: ServiceShutdownOptions,
    cancelled_after: Duration,
) -> io::Result<TimedRun<ServiceShutdownOutcomeCollection>> {
    let mut service_manager = ServiceManager::default();

    service_manager.register_service(CancellationIgnoringService {
        held_until_abort: (),
    });
    service_manager.register_service(CancellationIgnoringService {
        held_until_abort: (),
    });

    let cancellation_token = CancellationToken::new();
    let services_runtime = ServicesRuntime::start(service_manager, cancellation_token.clone())?;

    services_runtime.shut_down_while_undriven(|running_services| async move {
        let ObservedRun { run_output, .. } = ObservedRun::concurrently(
            TimedRun::measure(running_services.run_to_completion(shutdown_options)),
            async {
                advance(cancelled_after).await;
                cancellation_token.cancel();
            },
        )
        .await;

        run_output
    })
}

fn shutdown_waits_each_deadline_once_from_cancellation(
    shutdown_options @ ServiceShutdownOptions {
        abort_deadline,
        cooperative_deadline,
    }: ServiceShutdownOptions,
    cancelled_after: Duration,
) -> bool {
    matches!(
        shut_down_services_ignoring_abort_on_undriven_runtime(shutdown_options, cancelled_after),
        Ok(TimedRun { elapsed, .. })
            if cancelled_after
                .checked_add(cooperative_deadline)
                .and_then(|waited_through_cooperative_deadline| waited_through_cooperative_deadline.checked_add(abort_deadline))
                == Some(elapsed)
    )
}

#[test]
fn waits_each_shutdown_deadline_once_from_cancellation() {
    let ServiceShutdownOptions {
        cooperative_deadline,
        ..
    } = ServiceShutdownOptions::default();

    assert!(shutdown_waits_each_deadline_once_from_cancellation(
        ServiceShutdownOptions {
            abort_deadline: cooperative_deadline * 2,
            cooperative_deadline,
        },
        cooperative_deadline,
    ));
    assert!(shutdown_waits_each_deadline_once_from_cancellation(
        ServiceShutdownOptions {
            abort_deadline: cooperative_deadline / 2,
            cooperative_deadline,
        },
        cooperative_deadline,
    ));
}
