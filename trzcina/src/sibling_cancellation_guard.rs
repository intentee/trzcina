use tokio_util::sync::CancellationToken;

pub struct SiblingCancellationGuard {
    cancellation_token: CancellationToken,
}

impl SiblingCancellationGuard {
    #[must_use]
    pub fn new(cancellation_token: CancellationToken) -> Self {
        Self { cancellation_token }
    }
}

impl Drop for SiblingCancellationGuard {
    fn drop(&mut self) {
        self.cancellation_token.cancel();
    }
}
