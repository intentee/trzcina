use std::future::ready;

use tokio_util::sync::CancellationToken;

use trzcina::service::Service;

use crate::observation_sender::ObservationSender;
use crate::test_failure::TestFailure;

pub struct ObservableService {
    pub observation_sender: ObservationSender,
}

impl Service for ObservableService {
    type Error = TestFailure;

    fn run(
        self: Box<Self>,
        _cancellation_token: CancellationToken,
    ) -> impl Future<Output = Result<(), TestFailure>> + Send {
        ready(self.observation_sender.observe())
    }
}
