use tokio::time::MissedTickBehavior;
use tokio_util::sync::CancellationToken;

use trzcina_ticker::first_tick_timing::FirstTickTiming;
use trzcina_ticker::tick_context::TickContext;
use trzcina_ticker::tick_time_limit::TickTimeLimit;
use trzcina_ticker::ticker::Ticker;

use crate::first_tick_stalling_ticker::FirstTickStallingTicker;
use crate::test_failure::TestFailure;
use crate::tick_recorder::TickRecorder;

pub struct LaterTickAwaitingTicker {
    pub first_tick_stalling_ticker: FirstTickStallingTicker,
    pub later_tick_began: TickRecorder,
    pub later_tick_release: CancellationToken,
}

impl Ticker for LaterTickAwaitingTicker {
    type Error = TestFailure;

    fn first_tick_timing(&self) -> FirstTickTiming {
        self.first_tick_stalling_ticker.first_tick_timing()
    }

    async fn handle_tick(
        &mut self,
        cancellation_token: CancellationToken,
        tick_context @ TickContext {
            ticks_since_start, ..
        }: TickContext,
    ) -> Result<(), TestFailure> {
        if ticks_since_start > 0 {
            self.later_tick_began.record(tick_context)?;
            self.later_tick_release.cancelled().await;
        }

        self.first_tick_stalling_ticker
            .handle_tick(cancellation_token, tick_context)
            .await
    }

    fn missed_tick_behavior(&self) -> MissedTickBehavior {
        self.first_tick_stalling_ticker.missed_tick_behavior()
    }

    fn tick_time_limit(&self) -> TickTimeLimit {
        self.first_tick_stalling_ticker.tick_time_limit()
    }
}
