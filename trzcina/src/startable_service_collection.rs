use tokio_util::sync::CancellationToken;

use crate::registered_service::RegisteredService;

pub trait StartableServiceCollection {
    #[must_use]
    fn start(
        registered_services: Vec<RegisteredService>,
        cancellation_token: CancellationToken,
    ) -> Self;
}
