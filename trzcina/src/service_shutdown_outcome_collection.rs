use crate::assemblable_service_shutdown_outcome_collection::AssemblableServiceShutdownOutcomeCollection;
use crate::service_shutdown_error::ServiceShutdownError;
use crate::service_shutdown_failure_source::ServiceShutdownFailureSource;
use crate::service_shutdown_outcome_with_service_name::ServiceShutdownOutcomeWithServiceName;

/// How every service stopped, in registration order.
#[derive(Debug)]
pub struct ServiceShutdownOutcomeCollection {
    outcomes: Vec<ServiceShutdownOutcomeWithServiceName>,
}

impl ServiceShutdownOutcomeCollection {
    /// # Errors
    ///
    /// Returns every service that did not complete, each as the error that describes how it failed.
    pub fn into_result(self) -> Result<(), Vec<ServiceShutdownError>> {
        let failures: Vec<ServiceShutdownError> = self
            .outcomes
            .into_iter()
            .filter_map(ServiceShutdownOutcomeWithServiceName::into_failure)
            .collect();

        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }

    #[must_use]
    pub fn outcomes(&self) -> &[ServiceShutdownOutcomeWithServiceName] {
        &self.outcomes
    }
}

impl AssemblableServiceShutdownOutcomeCollection for ServiceShutdownOutcomeCollection {
    fn assemble(outcomes: Vec<ServiceShutdownOutcomeWithServiceName>) -> Self {
        Self { outcomes }
    }
}
