use std::time::Duration;

use log::error;
use log::info;
use tokio::task::JoinSet;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;

async fn wait_for_shutdown_signal(
    cancellation_token: &CancellationToken,
    has_running_services: bool,
) {
    if !has_running_services {
        return;
    }

    cancellation_token.cancelled().await;
    info!("Service is shutting down");
}

async fn drain_within_deadline(task_set: &mut JoinSet<()>, deadline: Duration) -> bool {
    timeout(deadline, async {
        while task_set.join_next().await.is_some() {}
    })
    .await
    .is_ok()
}

async fn abort_and_drain(task_set: &mut JoinSet<()>, abort_deadline: Duration) {
    error!("Shutdown deadline exceeded; aborting remaining services");
    task_set.abort_all();

    let abort_drain_result = timeout(abort_deadline, async {
        while task_set.join_next().await.is_some() {}
    })
    .await;

    if abort_drain_result.is_err() {
        error!(
            "Abort drain exceeded {abort_deadline:?}; one or more services ignored the abort signal and are leaked beyond the manager's lifetime",
        );
    }
}

pub async fn drain_to_completion(
    task_set: &mut JoinSet<()>,
    cancellation_token: &CancellationToken,
    has_running_services: bool,
    shutdown_deadline: Duration,
) {
    wait_for_shutdown_signal(cancellation_token, has_running_services).await;

    if !drain_within_deadline(task_set, shutdown_deadline).await {
        abort_and_drain(task_set, shutdown_deadline).await;
    }
}
