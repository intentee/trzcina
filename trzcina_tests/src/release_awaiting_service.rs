use tokio_util::sync::CancellationToken;

use trzcina::service::Service;

use crate::test_failure::TestFailure;
use crate::waiting_observer_service::WaitingObserverService;

pub struct ReleaseAwaitingService {
    pub cancellation_observer: WaitingObserverService,
    pub release: CancellationToken,
}

impl Service for ReleaseAwaitingService {
    type Error = TestFailure;

    async fn run(
        self: Box<Self>,
        cancellation_token: CancellationToken,
    ) -> Result<(), TestFailure> {
        let Self {
            cancellation_observer,
            release,
        } = *self;

        Box::new(cancellation_observer)
            .run(cancellation_token)
            .await?;
        release.cancelled().await;

        Ok(())
    }
}
