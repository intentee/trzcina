use tokio_util::sync::CancellationToken;

use crate::running_collection::RunningCollection;

pub trait Manager: Default {
    type Running: RunningCollection;

    fn start(self, cancellation_token: CancellationToken) -> Self::Running;
}
