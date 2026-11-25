use std::fmt;

use crate::service_shutdown_outcome::ServiceShutdownOutcome;
use crate::service_shutdown_outcome_with_service_name::ServiceShutdownOutcomeWithServiceName;

#[derive(Debug)]
pub struct FailedServiceOutcomes {
    outcomes: Vec<ServiceShutdownOutcomeWithServiceName>,
}

impl FailedServiceOutcomes {
    #[must_use]
    pub fn new(outcomes: Vec<ServiceShutdownOutcomeWithServiceName>) -> Self {
        Self { outcomes }
    }

    #[must_use]
    pub fn as_slice(&self) -> &[ServiceShutdownOutcomeWithServiceName] {
        &self.outcomes
    }
}

impl fmt::Display for FailedServiceOutcomes {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "service shutdown failed:")?;

        for ServiceShutdownOutcomeWithServiceName { name, outcome } in &self.outcomes {
            match outcome {
                ServiceShutdownOutcome::Completed => Ok(()),
                ServiceShutdownOutcome::Errored(service_error) => {
                    writeln!(formatter, "  service {name:?} errored: {service_error:#}")
                }
                ServiceShutdownOutcome::Panicked(panic_message) => {
                    writeln!(formatter, "  service {name:?} panicked: {panic_message}")
                }
                ServiceShutdownOutcome::AbortedByShutdownDeadline => {
                    writeln!(
                        formatter,
                        "  service {name:?} aborted after shutdown deadline"
                    )
                }
                ServiceShutdownOutcome::LeakedBeyondShutdownDeadline => {
                    writeln!(
                        formatter,
                        "  service {name:?} leaked beyond shutdown deadline"
                    )
                }
            }?;
        }

        Ok(())
    }
}
