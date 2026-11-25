use tokio_util::sync::CancellationToken;

use crate::running_service::RunningService;

pub trait SpawnableService {
    #[must_use]
    fn spawn(self, cancellation_token: CancellationToken) -> RunningService;
}
