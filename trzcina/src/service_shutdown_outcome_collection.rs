use crate::service_shutdown_error::ServiceShutdownError;
use crate::service_shutdown_outcome::ServiceShutdownOutcome;
use crate::service_shutdown_outcome_with_service_name::ServiceShutdownOutcomeWithServiceName;

#[derive(Debug)]
pub struct ServiceShutdownOutcomeCollection {
    outcomes: Vec<ServiceShutdownOutcomeWithServiceName>,
}

impl ServiceShutdownOutcomeCollection {
    #[must_use]
    pub fn new(outcomes: Vec<ServiceShutdownOutcomeWithServiceName>) -> Self {
        Self { outcomes }
    }

    #[must_use]
    pub fn outcomes(&self) -> &[ServiceShutdownOutcomeWithServiceName] {
        &self.outcomes
    }

    pub fn into_result(self) -> Result<(), ServiceShutdownError> {
        let failed: Vec<ServiceShutdownOutcomeWithServiceName> = self
            .outcomes
            .into_iter()
            .filter(|named_outcome| {
                !matches!(named_outcome.outcome, ServiceShutdownOutcome::Completed)
            })
            .collect();

        if failed.is_empty() {
            Ok(())
        } else {
            Err(ServiceShutdownError::new(failed))
        }
    }
}
