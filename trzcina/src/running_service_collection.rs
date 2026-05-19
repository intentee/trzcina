use std::any::Any;
use std::panic::AssertUnwindSafe;
use std::time::Duration;

use futures_util::FutureExt;
use log::error;
use log::info;
use tokio::sync::oneshot;
use tokio::task::JoinSet;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;

use crate::registered_service::RegisteredService;
use crate::running_service::RunningService;
use crate::service::Service;
use crate::service_shutdown_outcome::ServiceShutdownOutcome;
use crate::service_shutdown_outcome_collection::ServiceShutdownOutcomeCollection;
use crate::service_shutdown_outcome_with_service_name::ServiceShutdownOutcomeWithServiceName;
use crate::sibling_cancellation_guard::SiblingCancellationGuard;

fn panic_payload_to_string(panic_payload: Box<dyn Any + Send>) -> String {
    if let Some(static_str_message) = panic_payload.downcast_ref::<&'static str>() {
        return (*static_str_message).to_owned();
    }
    if let Ok(boxed_message) = panic_payload.downcast::<String>() {
        return *boxed_message;
    }
    "non-string panic payload".to_owned()
}

async fn run_service_with_sibling_cancellation_on_return(
    service_name: &'static str,
    mut service: Box<dyn Service>,
    cancellation_token: CancellationToken,
) -> ServiceShutdownOutcome {
    let _sibling_cancellation_guard = SiblingCancellationGuard::new(cancellation_token.clone());
    classify_service_outcome(service_name, &mut service, cancellation_token).await
}

async fn classify_service_outcome(
    service_name: &'static str,
    service: &mut Box<dyn Service>,
    cancellation_token: CancellationToken,
) -> ServiceShutdownOutcome {
    info!("Service {service_name:?} starting");
    let panic_caught_outcome = AssertUnwindSafe(service.run(cancellation_token))
        .catch_unwind()
        .await;

    match panic_caught_outcome {
        Ok(Ok(())) => {
            info!("Service {service_name:?} stopped");
            ServiceShutdownOutcome::Completed
        }
        Ok(Err(service_error)) => {
            error!("Service {service_name:?} error: {service_error:#?}");
            ServiceShutdownOutcome::Errored(service_error)
        }
        Err(panic_payload) => {
            let panic_message = panic_payload_to_string(panic_payload);
            error!("Service {service_name:?} panicked: {panic_message}");
            ServiceShutdownOutcome::Panicked(panic_message)
        }
    }
}

pub struct RunningServiceCollection {
    cancellation_token: CancellationToken,
    running_services: Vec<RunningService>,
    task_set: JoinSet<()>,
}

impl RunningServiceCollection {
    pub(crate) fn start(
        registered: Vec<RegisteredService>,
        cancellation_token: CancellationToken,
    ) -> Self {
        let mut running_services: Vec<RunningService> = Vec::with_capacity(registered.len());
        let mut task_set: JoinSet<()> = JoinSet::new();

        for RegisteredService { name, service } in registered {
            let (outcome_sender, outcome_receiver) = oneshot::channel::<ServiceShutdownOutcome>();
            let service_cancellation_token = cancellation_token.clone();

            task_set.spawn(async move {
                let outcome = run_service_with_sibling_cancellation_on_return(
                    name,
                    service,
                    service_cancellation_token,
                )
                .await;
                let _ = outcome_sender.send(outcome);
            });

            running_services.push(RunningService::new(name, outcome_receiver));
        }

        Self {
            cancellation_token,
            running_services,
            task_set,
        }
    }

    pub async fn run_to_completion(
        mut self,
        shutdown_deadline: Duration,
    ) -> ServiceShutdownOutcomeCollection {
        self.wait_for_shutdown_signal().await;

        if !self.drain_within_deadline(shutdown_deadline).await {
            self.abort_and_drain(shutdown_deadline).await;
        }

        let outcomes: Vec<ServiceShutdownOutcomeWithServiceName> =
            self.running_services.into_iter().map(Into::into).collect();

        ServiceShutdownOutcomeCollection::new(outcomes)
    }

    async fn wait_for_shutdown_signal(&self) {
        if self.running_services.is_empty() {
            return;
        }

        self.cancellation_token.cancelled().await;
        info!("Service is shutting down");
    }

    async fn drain_within_deadline(&mut self, deadline: Duration) -> bool {
        timeout(deadline, async {
            while self.task_set.join_next().await.is_some() {}
        })
        .await
        .is_ok()
    }

    async fn abort_and_drain(&mut self, abort_deadline: Duration) {
        error!("Shutdown deadline exceeded; aborting remaining services");
        self.task_set.abort_all();

        let abort_drain_result = timeout(abort_deadline, async {
            while self.task_set.join_next().await.is_some() {}
        })
        .await;

        if abort_drain_result.is_err() {
            error!(
                "Abort drain exceeded {abort_deadline:?}; one or more services ignored the abort signal and are leaked beyond the manager's lifetime",
            );
        }
    }
}
