use tokio_util::task::AbortOnDropHandle;

use crate::service_shutdown_outcome::ServiceShutdownOutcome;

pub struct RunningService {
    pub name: &'static str,
    pub task: AbortOnDropHandle<ServiceShutdownOutcome>,
}
