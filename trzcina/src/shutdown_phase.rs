use tokio::task::JoinError;

use crate::panic_payload::PanicPayload;
use crate::service_shutdown_outcome::ServiceShutdownOutcome;

#[derive(Clone, Copy)]
pub enum ShutdownPhase {
    Abort,
    Cooperative,
}

impl ShutdownPhase {
    pub fn outcome_of(self, join_error: JoinError) -> ServiceShutdownOutcome {
        match join_error.try_into_panic() {
            Ok(payload) => PanicPayload { payload }.into(),
            Err(_cancelled_task_error) => match self {
                Self::Abort => ServiceShutdownOutcome::AbortedByShutdownDeadline,
                Self::Cooperative => ServiceShutdownOutcome::CancelledByRuntimeShutdown,
            },
        }
    }
}
