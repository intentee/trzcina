use tokio::sync::oneshot::error::TryRecvError;

use crate::running_service::RunningService;
use crate::service_shutdown_outcome::ServiceShutdownOutcome;

#[derive(Debug)]
pub struct ServiceShutdownOutcomeWithServiceName {
    pub name: &'static str,
    pub outcome: ServiceShutdownOutcome,
}

impl From<RunningService> for ServiceShutdownOutcomeWithServiceName {
    fn from(mut running_service: RunningService) -> Self {
        let outcome = match running_service.outcome_receiver.try_recv() {
            Ok(outcome) => outcome,
            Err(TryRecvError::Closed) => ServiceShutdownOutcome::AbortedByShutdownDeadline,
            Err(TryRecvError::Empty) => ServiceShutdownOutcome::LeakedBeyondAbortDeadline,
        };

        Self {
            name: running_service.name,
            outcome,
        }
    }
}
