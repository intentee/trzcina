use tokio_util::sync::CancellationToken;

use trzcina::service::Service;

use crate::observation_sender::ObservationSender;
use crate::test_failure::TestFailure;

pub struct WaitingObserverService {
    pub observation_sender: ObservationSender,
}

impl Service for WaitingObserverService {
    type Error = TestFailure;

    async fn run(
        self: Box<Self>,
        cancellation_token: CancellationToken,
    ) -> Result<(), TestFailure> {
        cancellation_token.cancelled().await;

        self.observation_sender.observe()
    }
}
