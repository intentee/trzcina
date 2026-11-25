use std::time::Duration;

use futures_util::future::join_all;
use log::error;

use crate::service_shutdown_outcome_collection::ServiceShutdownOutcomeCollection;
use crate::service_shutdown_progress::ServiceShutdownProgress;

pub struct ServiceShutdownProgressCollection {
    service_progress: Vec<ServiceShutdownProgress>,
}

impl ServiceShutdownProgressCollection {
    pub async fn abort_within(self, abort_deadline: Duration) -> Self {
        let aborted_progress = Self::new(
            join_all(
                self.service_progress
                    .into_iter()
                    .map(|service_progress| service_progress.abort_within(abort_deadline)),
            )
            .await,
        );

        if aborted_progress.has_still_running_services() {
            error!(
                "Abort drain exceeded {abort_deadline:?}; one or more services ignored the abort signal and are leaked beyond the manager's lifetime"
            );
        }

        aborted_progress
    }

    #[must_use]
    pub const fn new(service_progress: Vec<ServiceShutdownProgress>) -> Self {
        Self { service_progress }
    }

    #[must_use]
    pub fn has_still_running_services(&self) -> bool {
        self.service_progress
            .iter()
            .any(ServiceShutdownProgress::is_still_running)
    }

    #[must_use]
    pub fn into_outcome_collection(self) -> ServiceShutdownOutcomeCollection {
        ServiceShutdownOutcomeCollection::new(
            self.service_progress
                .into_iter()
                .map(ServiceShutdownProgress::into_outcome_with_service_name)
                .collect(),
        )
    }
}
