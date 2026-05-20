use tokio::sync::oneshot;
use tokio::task::JoinSet;
use tokio::task::LocalSet;
use tokio_util::sync::CancellationToken;
use trzcina_service::RunToCompletionOptions;
use trzcina_service::RunningCollection;
use trzcina_service::RunningService;
use trzcina_service::ServiceShutdownOutcome;
use trzcina_service::ServiceShutdownOutcomeCollection;
use trzcina_service::ServiceShutdownOutcomeWithServiceName;
use trzcina_service::SiblingCancellationGuard;
use trzcina_service::classify_future_outcome;
use trzcina_service::drain_to_completion;

use crate::local_registered_service::LocalRegisteredService;

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
        let internal_cancellation_token = cancellation_token.child_token();

        for LocalRegisteredService { name, service } in registered {
            let (outcome_sender, outcome_receiver) = oneshot::channel::<ServiceShutdownOutcome>();
            let service_cancellation_token = internal_cancellation_token.clone();

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
            cancellation_token: internal_cancellation_token,
            local_set,
            running_services,
            task_set,
        }
    }
}

impl RunningCollection for LocalRunningServiceCollection {
    async fn run_to_completion(
        self,
        options: RunToCompletionOptions,
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
                    options.shutdown_deadline,
                )
                .await;
            })
            .await;

        let outcomes: Vec<ServiceShutdownOutcomeWithServiceName> =
            running_services.into_iter().map(Into::into).collect();

        ServiceShutdownOutcomeCollection::new(outcomes)
    }
}
