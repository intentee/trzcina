use std::time::Duration;

use log::error;

use trzcina_tokio_safe_delay::timeout_within_tokio_safe_delay_cap::timeout_within_tokio_safe_delay_cap;

use crate::running_service::RunningService;
use crate::service_shutdown_outcome::ServiceShutdownOutcome;
use crate::service_shutdown_outcome_with_service_name::ServiceShutdownOutcomeWithServiceName;
use crate::shutdown_phase::ShutdownPhase;

pub enum ServiceShutdownProgress {
    StillRunning(RunningService),
    Stopped(ServiceShutdownOutcomeWithServiceName),
}

impl ServiceShutdownProgress {
    pub async fn abort_within(self, abort_deadline: Duration) -> Self {
        if let Self::StillRunning(RunningService { name, task }) = &self {
            error!("Service {name:?} exceeded its shutdown deadline; aborting it");
            task.abort();
        }

        let aborted_progress = self.wait_within(abort_deadline, ShutdownPhase::Abort).await;

        if let Self::StillRunning(RunningService { name, .. }) = &aborted_progress {
            error!(
                "Service {name:?} ignored the abort signal for {abort_deadline:?} and is leaked beyond the manager's lifetime"
            );
        }

        aborted_progress
    }

    pub async fn wait_within(self, deadline: Duration, shutdown_phase: ShutdownPhase) -> Self {
        match self {
            Self::StillRunning(RunningService { name, mut task }) => {
                match timeout_within_tokio_safe_delay_cap(deadline, &mut task).await {
                    Ok(Ok(outcome)) => {
                        Self::Stopped(ServiceShutdownOutcomeWithServiceName { name, outcome })
                    }
                    Ok(Err(join_error)) => Self::Stopped(ServiceShutdownOutcomeWithServiceName {
                        name,
                        outcome: shutdown_phase.outcome_of(join_error),
                    }),
                    Err(_deadline_elapsed) => Self::StillRunning(RunningService { name, task }),
                }
            }
            stopped @ Self::Stopped(_) => stopped,
        }
    }

    #[must_use]
    pub fn into_outcome_with_service_name(self) -> ServiceShutdownOutcomeWithServiceName {
        match self {
            Self::StillRunning(RunningService { name, .. }) => {
                ServiceShutdownOutcomeWithServiceName {
                    name,
                    outcome: ServiceShutdownOutcome::LeakedBeyondShutdownDeadline,
                }
            }
            Self::Stopped(outcome_with_service_name) => outcome_with_service_name,
        }
    }
}
