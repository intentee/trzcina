use std::error::Error;
use std::fmt;

use crate::service_shutdown_outcome::ServiceShutdownOutcome;
use crate::service_shutdown_outcome_with_service_name::ServiceShutdownOutcomeWithServiceName;

#[derive(Debug)]
pub struct ServiceShutdownError {
    failed_outcomes: Vec<ServiceShutdownOutcomeWithServiceName>,
}

impl ServiceShutdownError {
    #[must_use]
    pub fn new(failed_outcomes: Vec<ServiceShutdownOutcomeWithServiceName>) -> Self {
        Self { failed_outcomes }
    }

    #[must_use]
    pub fn failed_outcomes(&self) -> &[ServiceShutdownOutcomeWithServiceName] {
        &self.failed_outcomes
    }
}

impl fmt::Display for ServiceShutdownError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "service shutdown failed:")?;

        for ServiceShutdownOutcomeWithServiceName { name, outcome } in &self.failed_outcomes {
            match outcome {
                ServiceShutdownOutcome::Completed => {}
                ServiceShutdownOutcome::Errored(service_error) => {
                    writeln!(f, "  service {name:?} errored: {service_error:#}")?;
                }
                ServiceShutdownOutcome::Panicked(panic_message) => {
                    writeln!(f, "  service {name:?} panicked: {panic_message}")?;
                }
                ServiceShutdownOutcome::AbortedByShutdownDeadline => {
                    writeln!(f, "  service {name:?} aborted after shutdown deadline")?;
                }
                ServiceShutdownOutcome::LeakedBeyondAbortDeadline => {
                    writeln!(f, "  service {name:?} leaked beyond shutdown deadline")?;
                }
            }
        }

        Ok(())
    }
}

impl Error for ServiceShutdownError {}
