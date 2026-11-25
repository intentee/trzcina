use tokio_util::sync::CancellationToken;

use trzcina::service::Service;

use crate::test_failure::TestFailure;
use crate::waiting_observer_service::WaitingObserverService;

pub struct ReleaseAwaitingService<TReleasedService> {
    pub cancellation_observer: WaitingObserverService,
    pub release: CancellationToken,
    pub released_service: TReleasedService,
}

impl<TReleasedService: Service<Error = TestFailure>> Service
    for ReleaseAwaitingService<TReleasedService>
{
    type Error = TestFailure;

    async fn run(
        self: Box<Self>,
        cancellation_token: CancellationToken,
    ) -> Result<(), TestFailure> {
        let Self {
            cancellation_observer,
            release,
            released_service,
        } = *self;

        Box::new(cancellation_observer)
            .run(cancellation_token.clone())
            .await?;
        release.cancelled().await;

        Box::new(released_service).run(cancellation_token).await
    }
}
