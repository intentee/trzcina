use std::time::Duration;

use tokio::time::interval;

use trzcina_ticker::tick_interval::TickInterval;
use trzcina_ticker::ticker_error::TickerError;
use trzcina_tokio_safe_delay::tokio_safe_delay_error::TokioSafeDelayError;

use crate::constants::SMALLEST_DURATION_STEP;

#[tokio::test]
async fn accepts_periods_up_to_tokio_interval_cap() {
    let tokio_interval_period_cap = interval(Duration::MAX).period();
    let period_beyond_cap = tokio_interval_period_cap + SMALLEST_DURATION_STEP;

    assert_eq!(
        TickInterval::new(tokio_interval_period_cap).map(TickInterval::period),
        Ok(tokio_interval_period_cap)
    );
    assert_eq!(
        TickInterval::new(period_beyond_cap),
        Err(TickerError::TickIntervalTooLong {
            source: TokioSafeDelayError::DurationBeyondCap {
                duration: period_beyond_cap,
                longest_duration: tokio_interval_period_cap,
            },
        })
    );
}
