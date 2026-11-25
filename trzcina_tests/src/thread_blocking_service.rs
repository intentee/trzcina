use std::convert::Infallible;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::RecvError;

use tokio::task::block_in_place;
use tokio_util::sync::CancellationToken;

use trzcina::service::Service;

use crate::observation_sender::ObservationSender;
use crate::test_failure::TestFailure;

pub struct ThreadBlockingService<TReleasedService> {
    pub release_receiver: Receiver<Infallible>,
    pub released_service: TReleasedService,
    pub started_sender: ObservationSender,
}

impl<TReleasedService: Service<Error = TestFailure>> Service
    for ThreadBlockingService<TReleasedService>
{
    type Error = TestFailure;

    fn run(
        self: Box<Self>,
        cancellation_token: CancellationToken,
    ) -> impl Future<Output = Result<(), TestFailure>> + Send {
        let Self {
            release_receiver,
            released_service,
            started_sender,
        } = *self;

        async move {
            started_sender.observe()?;
            block_in_place(|| {
                let Err(RecvError) = release_receiver.recv();
            });

            Box::new(released_service).run(cancellation_token).await
        }
    }
}
