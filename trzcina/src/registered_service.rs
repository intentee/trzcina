use std::panic::AssertUnwindSafe;

use futures_util::FutureExt;
use log::error;
use log::info;
use tokio::spawn;
use tokio_util::sync::CancellationToken;
use tokio_util::sync::DropGuard;
use tokio_util::task::AbortOnDropHandle;

use crate::panic_payload::PanicPayload;
use crate::runnable_service::RunnableService;
use crate::running_service::RunningService;
use crate::service::Service;
use crate::service_shutdown_outcome::ServiceShutdownOutcome;
use crate::spawnable_service::SpawnableService;

const NON_STRING_PANIC_MESSAGE: &str = "non-string panic payload";

pub struct RegisteredService {
    name: &'static str,
    service: Box<dyn RunnableService>,
}

impl RegisteredService {
    pub fn new(service: impl Service) -> Self {
        Self {
            name: service.name(),
            service: Box::new(service),
        }
    }

    async fn run_cancelling_siblings(
        self,
        sibling_cancellation_guard: DropGuard,
        cancellation_token: CancellationToken,
    ) -> ServiceShutdownOutcome {
        let outcome = self.run_isolating_panics(cancellation_token).await;

        drop(sibling_cancellation_guard);

        outcome
    }

    async fn run_isolating_panics(
        self,
        cancellation_token: CancellationToken,
    ) -> ServiceShutdownOutcome {
        let Self { name, service } = self;

        info!("Service {name:?} starting");

        match AssertUnwindSafe(service.run_to_outcome(name, cancellation_token))
            .catch_unwind()
            .await
        {
            Ok(outcome) => outcome,
            Err(payload) => {
                let panic_payload = PanicPayload { payload };

                error!(
                    "Service {name:?} panicked: {}",
                    panic_payload.message().unwrap_or(NON_STRING_PANIC_MESSAGE)
                );

                ServiceShutdownOutcome::from(panic_payload)
            }
        }
    }
}

impl SpawnableService for RegisteredService {
    fn spawn(self, cancellation_token: CancellationToken) -> RunningService {
        RunningService {
            name: self.name,
            task: AbortOnDropHandle::new(spawn(self.run_cancelling_siblings(
                cancellation_token.clone().drop_guard(),
                cancellation_token,
            ))),
        }
    }
}
