use std::future::ready;

use tokio_util::sync::CancellationToken;

use trzcina::service::Service;

use crate::test_failure::TestFailure;

pub struct CompletingService;

impl Service for CompletingService {
    type Error = TestFailure;

    fn run(
        self: Box<Self>,
        _cancellation_token: CancellationToken,
    ) -> impl Future<Output = Result<(), TestFailure>> + Send {
        ready(Ok(()))
    }
}
