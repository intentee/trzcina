use std::time::Duration;

use trzcina::constants::TOKIO_SAFE_DELAY_CAP;

use crate::ticker_error::TickerError;

/// The period between ticks: longer than zero and at most the 30 years tokio intervals can schedule.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TickInterval {
    period: Duration,
}

impl TickInterval {
    /// # Errors
    ///
    /// Returns `TickerError::ZeroTickInterval` when the period is zero, which no ticker can run on, and
    /// `TickerError::TickIntervalTooLong` when the period is longer than tokio's intervals can schedule.
    pub const fn new(period: Duration) -> Result<Self, TickerError> {
        if period.is_zero() {
            Err(TickerError::ZeroTickInterval)
        } else if period.as_nanos() > TOKIO_SAFE_DELAY_CAP.as_nanos() {
            Err(TickerError::TickIntervalTooLong {
                longest_period: TOKIO_SAFE_DELAY_CAP,
                period,
            })
        } else {
            Ok(Self { period })
        }
    }

    #[must_use]
    pub const fn period(self) -> Duration {
        self.period
    }
}

impl Default for TickInterval {
    fn default() -> Self {
        Self {
            period: Duration::from_secs(1),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::TickInterval;
    use super::TickerError;

    const SMALLEST_POSITIVE_PERIOD: Duration = Duration::from_nanos(1);

    #[test]
    fn accepts_smallest_positive_period() {
        assert_eq!(
            TickInterval::new(SMALLEST_POSITIVE_PERIOD).map(TickInterval::period),
            Ok(SMALLEST_POSITIVE_PERIOD)
        );
    }

    #[test]
    fn defaults_to_one_second_period() {
        assert_eq!(TickInterval::default().period(), Duration::from_secs(1));
    }

    #[test]
    fn rejects_zero_period() {
        assert_eq!(
            TickInterval::new(Duration::ZERO),
            Err(TickerError::ZeroTickInterval)
        );
    }
}
