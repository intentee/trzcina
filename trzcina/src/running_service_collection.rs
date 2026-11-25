use std::time::Duration;

use futures_util::future::join_all;
use log::error;
use log::info;
use tokio_util::sync::CancellationToken;

use crate::registered_service::RegisteredService;
use crate::running_service::RunningService;
use crate::service_shutdown_options::ServiceShutdownOptions;
use crate::service_shutdown_outcome_collection::ServiceShutdownOutcomeCollection;
use crate::service_shutdown_progress::ServiceShutdownProgress;
use crate::service_shutdown_progress_collection::ServiceShutdownProgressCollection;
use crate::shutdown_phase::ShutdownPhase;
use crate::spawnable_service::SpawnableService;
use crate::startable_service_collection::StartableServiceCollection;

pub struct RunningServiceCollection {
    cancellation_token: CancellationToken,
    running_services: Vec<RunningService>,
}

impl RunningServiceCollection {
    pub async fn run_to_completion(
        self,
        ServiceShutdownOptions {
            cooperative_deadline,
            abort_deadline,
        }: ServiceShutdownOptions,
    ) -> ServiceShutdownOutcomeCollection {
        self.wait_for_shutdown_signal().await;

        let cooperative_progress = self.stop_cooperatively(cooperative_deadline).await;
        let final_progress = if cooperative_progress.has_still_running_services() {
            error!("Shutdown deadline exceeded; aborting remaining services");
            cooperative_progress.abort_within(abort_deadline).await
        } else {
            cooperative_progress
        };

        final_progress.into_outcome_collection()
    }

    async fn stop_cooperatively(
        self,
        cooperative_deadline: Duration,
    ) -> ServiceShutdownProgressCollection {
        ServiceShutdownProgressCollection::new(
            join_all(self.running_services.into_iter().map(|running_service| {
                ServiceShutdownProgress::StillRunning(running_service)
                    .wait_within(cooperative_deadline, ShutdownPhase::Cooperative)
            }))
            .await,
        )
    }

    async fn wait_for_shutdown_signal(&self) {
        if self.running_services.is_empty() {
            return;
        }

        self.cancellation_token.cancelled().await;
        info!("Service is shutting down");
    }
}

impl StartableServiceCollection for RunningServiceCollection {
    fn start(
        registered_services: Vec<RegisteredService>,
        cancellation_token: CancellationToken,
    ) -> Self {
        let running_services: Vec<RunningService> = registered_services
            .into_iter()
            .map(|registered_service| registered_service.spawn(cancellation_token.clone()))
            .collect();

        Self {
            cancellation_token,
            running_services,
        }
    }
}

#[cfg(test)]
mod tests {
    use futures_util::FutureExt;
    use tokio_util::sync::CancellationToken;

    use crate::service_shutdown_options::ServiceShutdownOptions;
    use crate::startable_service_collection::StartableServiceCollection;

    use super::RunningServiceCollection;

    #[test]
    fn completes_immediately_without_services() {
        assert!(matches!(
            RunningServiceCollection::start(Vec::new(), CancellationToken::new())
                .run_to_completion(ServiceShutdownOptions::default())
                .now_or_never(),
            Some(shutdown_report) if shutdown_report.outcomes().is_empty()
        ));
    }
}
