use std::any::type_name;
use std::error::Error;

use tokio_util::sync::CancellationToken;

/// A long-lived async service run by a `ServiceManager`.
///
/// When any service stops, the cancellation token every service receives is cancelled, so its siblings stop too.
/// A service that returns an error is reported as `ServiceShutdownError::Errored`, with that error as its source,
/// and a service that panics is reported as panicked.
pub trait Service: Send + 'static {
    type Error: Error + Send + Sync + 'static;

    fn name(&self) -> &'static str {
        type_name::<Self>()
    }

    fn run(
        self: Box<Self>,
        cancellation_token: CancellationToken,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}
