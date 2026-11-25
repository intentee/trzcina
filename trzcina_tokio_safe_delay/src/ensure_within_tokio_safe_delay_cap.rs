use std::time::Duration;

use crate::constants::TOKIO_SAFE_DELAY_CAP;
use crate::tokio_safe_delay_error::TokioSafeDelayError;

/// Returns `duration` when it is at most `TOKIO_SAFE_DELAY_CAP`, the 30 years tokio caps sleeps and interval
/// periods at.
///
/// # Errors
///
/// Returns `TokioSafeDelayError::DurationBeyondCap` when `duration` is longer than `TOKIO_SAFE_DELAY_CAP`.
pub const fn ensure_within_tokio_safe_delay_cap(
    duration: Duration,
) -> Result<Duration, TokioSafeDelayError> {
    if duration.as_nanos() > TOKIO_SAFE_DELAY_CAP.as_nanos() {
        Err(TokioSafeDelayError::DurationBeyondCap {
            duration,
            longest_duration: TOKIO_SAFE_DELAY_CAP,
        })
    } else {
        Ok(duration)
    }
}
