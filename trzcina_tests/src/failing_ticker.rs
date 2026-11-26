use std::future::ready;

use tokio_util::sync::CancellationToken;

use trzcina_ticker::first_tick_timing::FirstTickTiming;
use trzcina_ticker::tick_context::TickContext;
use trzcina_ticker::tick_time_limit::TickTimeLimit;
use trzcina_ticker::ticker::Ticker;

use crate::test_failure::TestFailure;

pub struct FailingTicker {
    pub failing_tick: u64,
    pub first_tick_timing: FirstTickTiming,
    pub tick_time_limit: TickTimeLimit,
}

impl Ticker for FailingTicker {
    type Error = TestFailure;

    fn first_tick_timing(&self) -> FirstTickTiming {
        self.first_tick_timing
    }

    fn handle_tick(
        &mut self,
        _cancellation_token: CancellationToken,
        TickContext {
            ticks_since_start, ..
        }: TickContext,
    ) -> impl Future<Output = Result<(), TestFailure>> + Send {
        ready(if ticks_since_start == self.failing_tick {
            Err(TestFailure::DeliberateTickFailure)
        } else {
            Ok(())
        })
    }

    fn tick_time_limit(&self) -> TickTimeLimit {
        self.tick_time_limit
    }
}
