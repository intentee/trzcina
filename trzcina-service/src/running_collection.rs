use std::future::Future;

use crate::run_to_completion_options::RunToCompletionOptions;
use crate::service_shutdown_outcome_collection::ServiceShutdownOutcomeCollection;

pub trait RunningCollection {
    fn run_to_completion(
        self,
        options: RunToCompletionOptions,
    ) -> impl Future<Output = ServiceShutdownOutcomeCollection>;
}
