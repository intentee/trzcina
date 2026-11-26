use tokio::sync::oneshot;

use crate::test_failure::TestFailure;

pub struct ObservationSender {
    sender: oneshot::Sender<()>,
}

impl ObservationSender {
    /// # Errors
    ///
    /// Returns `TestFailure::ObservationNotDelivered` when the observing side was dropped.
    pub fn observe(self) -> Result<(), TestFailure> {
        self.sender
            .send(())
            .map_err(|()| TestFailure::ObservationNotDelivered)
    }
}

impl From<oneshot::Sender<()>> for ObservationSender {
    fn from(sender: oneshot::Sender<()>) -> Self {
        Self { sender }
    }
}
