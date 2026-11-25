use std::time::Duration;

use tokio::time::error::Elapsed;
use tokio::time::timeout;

use crate::constants::TOKIO_SAFE_DELAY_CAP;

/// Runs `future` under `tokio::time::timeout`, except that a duration beyond `TOKIO_SAFE_DELAY_CAP` never elapses,
/// so no deadline comes near the largest `Instant`, where tokio's timer panics.
///
/// # Errors
///
/// Returns `Elapsed` when `future` does not finish within a `duration` of at most `TOKIO_SAFE_DELAY_CAP`.
pub async fn timeout_within_tokio_safe_delay_cap<TFuture: Future>(
    duration: Duration,
    future: TFuture,
) -> Result<TFuture::Output, Elapsed> {
    if duration > TOKIO_SAFE_DELAY_CAP {
        Ok(future.await)
    } else {
        timeout(duration, future).await
    }
}
