use std::convert::Infallible;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::RecvError;

use tokio::task::block_in_place;
use tokio_util::sync::CancellationToken;

use trzcina::service::Service;

use crate::observation_sender::ObservationSender;
use crate::test_failure::TestFailure;

pub struct ThreadBlockingService {
    pub release_receiver: Receiver<Infallible>,
    pub started_sender: ObservationSender,
}

impl Service for ThreadBlockingService {
    type Error = TestFailure;

    fn run(
        self: Box<Self>,
        _cancellation_token: CancellationToken,
    ) -> impl Future<Output = Result<(), TestFailure>> + Send {
        let Self {
            release_receiver,
            started_sender,
        } = *self;

        async move {
            started_sender.observe()?;
            block_in_place(|| {
                let Err(RecvError) = release_receiver.recv();
            });

            Ok(())
        }
    }
}
