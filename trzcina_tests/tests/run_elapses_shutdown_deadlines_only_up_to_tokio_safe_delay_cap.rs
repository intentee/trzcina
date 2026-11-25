use std::io;
use std::time::Duration;

use tokio::sync::oneshot;
use tokio::sync::oneshot::error::RecvError;
use tokio::task::JoinError;
use tokio::task::spawn_blocking;
use tokio::time::advance;
use tokio_util::sync::CancellationToken;

use trzcina::service_manager::ServiceManager;
use trzcina::service_shutdown_error::ServiceShutdownError;
use trzcina::service_shutdown_options::ServiceShutdownOptions;
use trzcina::service_shutdown_outcome_collection::ServiceShutdownOutcomeCollection;
use trzcina_tests::block_on_paused_runtime::block_on_paused_runtime;
use trzcina_tests::cancellation_ignoring_service::CancellationIgnoringService;
use trzcina_tests::observed_run::ObservedRun;
use trzcina_tests::release_awaiting_service::ReleaseAwaitingService;
use trzcina_tests::services_runtime::ServicesRuntime;
use trzcina_tests::waiting_observer_service::WaitingObserverService;
use trzcina_tokio_safe_delay::constants::TOKIO_SAFE_DELAY_CAP;

use crate::constants::SMALLEST_DURATION_STEP;
use crate::constants::TOKIO_TIMER_GRANULARITY;

async fn cooperative_phase_result_released_after(
    cooperative_deadline: Duration,
    released_after: Duration,
) -> Result<Result<(), Vec<ServiceShutdownError>>, RecvError> {
    let (cancellation_observation_sender, cancellation_observation_receiver) =
        oneshot::channel::<()>();
    let cancellation_token = CancellationToken::new();
    let release = CancellationToken::new();
    let mut manager = ServiceManager::default();

    manager.register_service(ReleaseAwaitingService {
        cancellation_observer: WaitingObserverService {
            observation_sender: cancellation_observation_sender.into(),
        },
        release: release.clone(),
    });

    ObservedRun::concurrently(
        manager
            .start(cancellation_token.clone())
            .run_to_completion(ServiceShutdownOptions {
                cooperative_deadline,
                ..ServiceShutdownOptions::default()
            }),
        async {
            cancellation_token.cancel();

            let cancellation_observation = cancellation_observation_receiver.await;

            advance(released_after).await;
            release.cancel();

            cancellation_observation
        },
    )
    .await
    .into_run_output()
    .map(ServiceShutdownOutcomeCollection::into_result)
}

fn abort_phase_result_released_after(
    abort_deadline: Duration,
    released_after: Duration,
) -> io::Result<Result<Result<(), Vec<ServiceShutdownError>>, JoinError>> {
    let mut service_manager = ServiceManager::default();

    service_manager.register_service(CancellationIgnoringService {
        held_until_abort: (),
    });

    let cancellation_token = CancellationToken::new();
    let ServicesRuntime {
        running_services,
        runtime,
    } = ServicesRuntime::start(service_manager, cancellation_token.clone())?;

    cancellation_token.cancel();

    block_on_paused_runtime(async move {
        ObservedRun::concurrently(
            running_services.run_to_completion(ServiceShutdownOptions {
                abort_deadline,
                cooperative_deadline: Duration::ZERO,
            }),
            async move {
                advance(released_after).await;
                spawn_blocking(move || drop(runtime)).await
            },
        )
        .await
        .into_run_output()
        .map(ServiceShutdownOutcomeCollection::into_result)
    })
}

#[test]
fn elapses_shutdown_deadlines_only_up_to_tokio_safe_delay_cap() {
    let deadline_beyond_cap = TOKIO_SAFE_DELAY_CAP + SMALLEST_DURATION_STEP;
    let released_after_deadline_beyond_cap = TOKIO_SAFE_DELAY_CAP + TOKIO_TIMER_GRANULARITY;

    assert!(matches!(
        block_on_paused_runtime(cooperative_phase_result_released_after(
            TOKIO_SAFE_DELAY_CAP,
            TOKIO_SAFE_DELAY_CAP
        )),
        Ok(Ok(Err(failures))) if matches!(
            failures.as_slice(),
            [ServiceShutdownError::AbortedByShutdownDeadline { .. }]
        )
    ));
    assert!(matches!(
        block_on_paused_runtime(cooperative_phase_result_released_after(
            deadline_beyond_cap,
            released_after_deadline_beyond_cap
        )),
        Ok(Ok(Ok(())))
    ));
    assert!(matches!(
        abort_phase_result_released_after(deadline_beyond_cap, released_after_deadline_beyond_cap),
        Ok(Ok(Err(failures))) if matches!(
            failures.as_slice(),
            [ServiceShutdownError::AbortedByShutdownDeadline { .. }]
        )
    ));
}
