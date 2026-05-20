use tokio::sync::oneshot;

use crate::service_shutdown_outcome::ServiceShutdownOutcome;

pub struct RunningService {
    pub name: &'static str,
    pub outcome_receiver: oneshot::Receiver<ServiceShutdownOutcome>,
}

impl RunningService {
    #[must_use]
    pub fn new(
        name: &'static str,
        outcome_receiver: oneshot::Receiver<ServiceShutdownOutcome>,
    ) -> Self {
        Self {
            name,
            outcome_receiver,
        }
    }
}
