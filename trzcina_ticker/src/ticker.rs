use std::error::Error;

use tokio::time::MissedTickBehavior;
use tokio_util::sync::CancellationToken;

use crate::first_tick_timing::FirstTickTiming;
use crate::tick_context::TickContext;
use crate::tick_interval::TickInterval;
use crate::tick_time_limit::TickTimeLimit;

/// A periodic task, run as a service by wrapping it in a `TickerService`.
pub trait Ticker: Send + 'static {
    type Error: Error + Send + Sync + 'static;

    fn first_tick_timing(&self) -> FirstTickTiming {
        FirstTickTiming::Immediate
    }

    fn handle_tick(
        &mut self,
        cancellation_token: CancellationToken,
        tick_context: TickContext,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;

    fn missed_tick_behavior(&self) -> MissedTickBehavior {
        MissedTickBehavior::Delay
    }

    fn tick_interval(&self) -> TickInterval {
        TickInterval::default()
    }

    fn tick_time_limit(&self) -> TickTimeLimit {
        TickTimeLimit::Unlimited
    }
}

#[cfg(test)]
mod tests {
    use std::future::ready;

    use tokio::time::MissedTickBehavior;
    use tokio_util::sync::CancellationToken;

    use crate::tick_context::TickContext;
    use crate::tick_time_limit::TickTimeLimit;
    use crate::ticker_error::TickerError;

    use super::Ticker;

    struct IdleTicker;

    impl Ticker for IdleTicker {
        type Error = TickerError;

        fn handle_tick(
            &mut self,
            _cancellation_token: CancellationToken,
            _tick_context: TickContext,
        ) -> impl Future<Output = Result<(), TickerError>> + Send {
            ready(Ok(()))
        }
    }

    #[test]
    fn defaults_to_delay_missed_tick_behavior() {
        assert_eq!(IdleTicker.missed_tick_behavior(), MissedTickBehavior::Delay);
    }

    #[test]
    fn defaults_to_unlimited_tick_time_limit() {
        assert_eq!(IdleTicker.tick_time_limit(), TickTimeLimit::Unlimited);
    }
}
