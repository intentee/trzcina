use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;

use trzcina::service::Service;
use trzcina::service_manager::ServiceManager;

use crate::release_awaiting_service::ReleaseAwaitingService;
use crate::test_failure::TestFailure;
use crate::waiting_observer_service::WaitingObserverService;

pub struct ReleaseAwaitingControl {
    pub cancellation_observation_receiver: oneshot::Receiver<()>,
    pub release: CancellationToken,
}

impl ReleaseAwaitingControl {
    pub fn register<TReleasedService: Service<Error = TestFailure>>(
        service_manager: &mut ServiceManager,
        released_service: TReleasedService,
    ) -> Self {
        let (cancellation_observation_sender, cancellation_observation_receiver) =
            oneshot::channel::<()>();
        let release = CancellationToken::new();

        service_manager.register_service(ReleaseAwaitingService {
            cancellation_observer: WaitingObserverService {
                observation_sender: cancellation_observation_sender.into(),
            },
            release: release.clone(),
            released_service,
        });

        Self {
            cancellation_observation_receiver,
            release,
        }
    }
}
