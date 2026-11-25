use tokio::time::MissedTickBehavior;
use tokio_util::sync::CancellationToken;

use trzcina_ticker::first_tick_timing::FirstTickTiming;
use trzcina_ticker::tick_context::TickContext;
use trzcina_ticker::tick_time_limit::TickTimeLimit;
use trzcina_ticker::ticker::Ticker;

use crate::test_failure::TestFailure;
use crate::tick_recorder::TickRecorder;

pub struct FirstTickStallingTicker {
    pub first_tick_began: CancellationToken,
    pub first_tick_release: CancellationToken,
    pub first_tick_timing: FirstTickTiming,
    pub missed_tick_behavior: MissedTickBehavior,
    pub tick_recorder: TickRecorder,
    pub tick_time_limit: TickTimeLimit,
}

impl Ticker for FirstTickStallingTicker {
    type Error = TestFailure;

    fn first_tick_timing(&self) -> FirstTickTiming {
        self.first_tick_timing
    }

    async fn handle_tick(
        &mut self,
        _cancellation_token: CancellationToken,
        tick_context @ TickContext {
            ticks_since_start, ..
        }: TickContext,
    ) -> Result<(), TestFailure> {
        if ticks_since_start == 0 {
            self.first_tick_began.cancel();
            self.first_tick_release.cancelled().await;
        }

        self.tick_recorder.record(tick_context)
    }

    fn missed_tick_behavior(&self) -> MissedTickBehavior {
        self.missed_tick_behavior
    }

    fn tick_time_limit(&self) -> TickTimeLimit {
        self.tick_time_limit
    }
}
