use std::any::Any;
use std::panic::resume_unwind;

use futures_util::future::lazy;
use tokio_util::sync::CancellationToken;

use trzcina::service::Service;

use crate::test_failure::TestFailure;

pub struct PanickingService {
    pub panic_payload: Box<dyn Any + Send>,
}

impl Service for PanickingService {
    type Error = TestFailure;

    fn run(
        self: Box<Self>,
        _cancellation_token: CancellationToken,
    ) -> impl Future<Output = Result<(), TestFailure>> + Send {
        let Self { panic_payload } = *self;

        lazy(move |_context| resume_unwind(panic_payload))
    }
}
