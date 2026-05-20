use std::error::Error;
use std::fmt;

use crate::service_shutdown_outcome::ServiceShutdownOutcome;
use crate::service_shutdown_outcome_with_service_name::ServiceShutdownOutcomeWithServiceName;

fn build_failure_line(entry: &ServiceShutdownOutcomeWithServiceName) -> Option<String> {
    let ServiceShutdownOutcomeWithServiceName { name, outcome } = entry;
    match outcome {
        ServiceShutdownOutcome::Completed => None,
        ServiceShutdownOutcome::Errored(service_error) => {
            Some(format!("  service {name:?} errored: {service_error:#}\n"))
        }
        ServiceShutdownOutcome::Panicked(panic_message) => {
            Some(format!("  service {name:?} panicked: {panic_message}\n"))
        }
        ServiceShutdownOutcome::AbortedByShutdownDeadline => Some(format!(
            "  service {name:?} aborted after shutdown deadline\n"
        )),
        ServiceShutdownOutcome::LeakedBeyondAbortDeadline => Some(format!(
            "  service {name:?} leaked beyond shutdown deadline\n"
        )),
    }
}

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
        f.write_str("service shutdown failed:\n")?;
        for entry in &self.failed_outcomes {
            if let Some(line) = build_failure_line(entry) {
                f.write_str(&line)?;
            }
        }
        Ok(())
    }
}

impl Error for ServiceShutdownError {}
