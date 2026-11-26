use std::convert::Infallible;
use std::future::pending;

use tokio_util::sync::CancellationToken;

use trzcina::service::Service;
use trzcina::service_manager::ServiceManager;

use crate::test_failure::TestFailure;

pub struct CancellationIgnoringService<THeldUntilAbort> {
    pub held_until_abort: THeldUntilAbort,
}

impl CancellationIgnoringService<()> {
    pub fn register(service_manager: &mut ServiceManager, service_count: usize) {
        for _service_index in 0..service_count {
            service_manager.register_service(Self {
                held_until_abort: (),
            });
        }
    }
}

impl<THeldUntilAbort: Send + 'static> Service for CancellationIgnoringService<THeldUntilAbort> {
    type Error = TestFailure;

    async fn run(
        self: Box<Self>,
        _cancellation_token: CancellationToken,
    ) -> Result<(), TestFailure> {
        let _held_until_abort = self.held_until_abort;

        match pending::<Infallible>().await {}
    }
}
