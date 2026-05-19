use std::time::Duration;

use tokio::sync::oneshot;
use tokio::task::JoinSet;
use tokio::task::LocalSet;
use tokio_util::sync::CancellationToken;

use crate::local_registered_service::LocalRegisteredService;
use crate::running_service::RunningService;
use crate::service_outcome_classifier::classify_future_outcome;
use crate::service_shutdown_outcome::ServiceShutdownOutcome;
use crate::service_shutdown_outcome_collection::ServiceShutdownOutcomeCollection;
use crate::service_shutdown_outcome_with_service_name::ServiceShutdownOutcomeWithServiceName;
use crate::service_task_drainer::drain_to_completion;
use crate::sibling_cancellation_guard::SiblingCancellationGuard;

pub struct LocalRunningServiceCollection {
    cancellation_token: CancellationToken,
    local_set: LocalSet,
    running_services: Vec<RunningService>,
    task_set: JoinSet<()>,
}

impl LocalRunningServiceCollection {
    #[must_use]
    pub fn start(
        registered: Vec<LocalRegisteredService>,
        cancellation_token: CancellationToken,
    ) -> Self {
        let mut running_services: Vec<RunningService> = Vec::with_capacity(registered.len());
        let mut task_set: JoinSet<()> = JoinSet::new();
        let local_set = LocalSet::new();

        for LocalRegisteredService { name, service } in registered {
            let (outcome_sender, outcome_receiver) = oneshot::channel::<ServiceShutdownOutcome>();
            let service_cancellation_token = cancellation_token.clone();

            task_set.spawn_local_on(
                async move {
                    let _sibling_cancellation_guard =
                        SiblingCancellationGuard::new(service_cancellation_token.clone());
                    let mut service = service;
                    let outcome =
                        classify_future_outcome(name, service.run(service_cancellation_token))
                            .await;
                    let _ = outcome_sender.send(outcome);
                },
                &local_set,
            );

            running_services.push(RunningService::new(name, outcome_receiver));
        }

        Self {
            cancellation_token,
            local_set,
            running_services,
            task_set,
        }
    }

    pub async fn run_to_completion(
        self,
        shutdown_deadline: Duration,
    ) -> ServiceShutdownOutcomeCollection {
        let Self {
            cancellation_token,
            local_set,
            running_services,
            mut task_set,
        } = self;

        let has_running_services = !running_services.is_empty();

        local_set
            .run_until(async {
                drain_to_completion(
                    &mut task_set,
                    &cancellation_token,
                    has_running_services,
                    shutdown_deadline,
                )
                .await;
            })
            .await;

        let outcomes: Vec<ServiceShutdownOutcomeWithServiceName> =
            running_services.into_iter().map(Into::into).collect();

        ServiceShutdownOutcomeCollection::new(outcomes)
    }
}
