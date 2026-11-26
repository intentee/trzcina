use std::future::ready;

use tokio_util::sync::CancellationToken;

use trzcina_ticker::first_tick_timing::FirstTickTiming;
use trzcina_ticker::tick_context::TickContext;
use trzcina_ticker::tick_interval::TickInterval;
use trzcina_ticker::tick_time_limit::TickTimeLimit;
use trzcina_ticker::ticker::Ticker;

use crate::record_tick_contexts::record_tick_contexts;
use crate::test_failure::TestFailure;
use crate::tick_recorder::TickRecorder;

pub struct RecordingTicker {
    first_tick_timing: FirstTickTiming,
    tick_interval: TickInterval,
    tick_recorder: TickRecorder,
    tick_time_limit: TickTimeLimit,
}

impl RecordingTicker {
    /// # Errors
    ///
    /// Returns the error that stopped the ticker.
    pub async fn record<TBeforeReceiving, TBeforeReceivingStep>(
        first_tick_timing: FirstTickTiming,
        tick_interval: TickInterval,
        tick_time_limit: TickTimeLimit,
        before_receiving: TBeforeReceiving,
        ticks_before_cancellation: usize,
    ) -> Result<Vec<TickContext>, TestFailure>
    where
        TBeforeReceiving: FnOnce(CancellationToken) -> TBeforeReceivingStep,
        TBeforeReceivingStep: Future<Output = ()>,
    {
        record_tick_contexts(
            |tick_recorder| Self {
                first_tick_timing,
                tick_interval,
                tick_recorder,
                tick_time_limit,
            },
            before_receiving,
            ticks_before_cancellation,
        )
        .await
    }
}

impl Ticker for RecordingTicker {
    type Error = TestFailure;

    fn first_tick_timing(&self) -> FirstTickTiming {
        self.first_tick_timing
    }

    fn handle_tick(
        &mut self,
        _cancellation_token: CancellationToken,
        tick_context: TickContext,
    ) -> impl Future<Output = Result<(), TestFailure>> + Send {
        ready(self.tick_recorder.record(tick_context))
    }

    fn tick_interval(&self) -> TickInterval {
        self.tick_interval
    }

    fn tick_time_limit(&self) -> TickTimeLimit {
        self.tick_time_limit
    }
}
