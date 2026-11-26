use std::time::Duration;

use futures_util::future::join_all;

use crate::assemblable_service_shutdown_outcome_collection::AssemblableServiceShutdownOutcomeCollection;
use crate::service_shutdown_outcome_collection::ServiceShutdownOutcomeCollection;
use crate::service_shutdown_progress::ServiceShutdownProgress;

pub struct ServiceShutdownProgressCollection {
    service_progress: Vec<ServiceShutdownProgress>,
}

impl ServiceShutdownProgressCollection {
    pub async fn abort_within(self, abort_deadline: Duration) -> Self {
        Self::new(
            join_all(
                self.service_progress
                    .into_iter()
                    .map(|service_progress| service_progress.abort_within(abort_deadline)),
            )
            .await,
        )
    }

    #[must_use]
    pub const fn new(service_progress: Vec<ServiceShutdownProgress>) -> Self {
        Self { service_progress }
    }

    #[must_use]
    pub fn into_outcome_collection(self) -> ServiceShutdownOutcomeCollection {
        ServiceShutdownOutcomeCollection::assemble(
            self.service_progress
                .into_iter()
                .map(ServiceShutdownProgress::into_outcome_with_service_name)
                .collect(),
        )
    }
}
