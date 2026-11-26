use std::future::ready;

use tokio_util::sync::CancellationToken;

use trzcina_tests::expected_tick_contexts::ExpectedTickContexts;
use trzcina_tests::record_tick_contexts::record_tick_contexts;
use trzcina_tests::test_failure::TestFailure;
use trzcina_tests::tick_recorder::TickRecorder;
use trzcina_ticker::first_tick_timing::FirstTickTiming;
use trzcina_ticker::tick_context::TickContext;
use trzcina_ticker::tick_interval::TickInterval;
use trzcina_ticker::tick_time_limit::TickTimeLimit;
use trzcina_ticker::ticker::Ticker;

const TICKS_BEFORE_CANCELLATION: usize = 3;

async fn record_ticks_requesting_shutdown_at(
    first_tick_timing: FirstTickTiming,
    shutdown_requesting_tick: u64,
    tick_time_limit: TickTimeLimit,
) -> Result<Vec<TickContext>, TestFailure> {
    record_tick_contexts(
        |tick_recorder| ShutdownRequestingTicker {
            first_tick_timing,
            shutdown_requesting_tick,
            tick_recorder,
            tick_time_limit,
        },
        |_cancellation_token| ready(()),
        TICKS_BEFORE_CANCELLATION,
    )
    .await
}

struct ShutdownRequestingTicker {
    first_tick_timing: FirstTickTiming,
    shutdown_requesting_tick: u64,
    tick_recorder: TickRecorder,
    tick_time_limit: TickTimeLimit,
}

impl Ticker for ShutdownRequestingTicker {
    type Error = TestFailure;

    fn first_tick_timing(&self) -> FirstTickTiming {
        self.first_tick_timing
    }

    fn handle_tick(
        &mut self,
        cancellation_token: CancellationToken,
        tick_context @ TickContext {
            ticks_since_start, ..
        }: TickContext,
    ) -> impl Future<Output = Result<(), TestFailure>> + Send {
        if ticks_since_start == self.shutdown_requesting_tick {
            cancellation_token.cancel();
        }

        ready(self.tick_recorder.record(tick_context))
    }

    fn tick_time_limit(&self) -> TickTimeLimit {
        self.tick_time_limit
    }
}

#[tokio::test(start_paused = true)]
async fn passes_service_cancellation_token_to_handle_tick() {
    let expected_tick_contexts = ExpectedTickContexts {
        tick_period: TickInterval::default().period(),
    };

    assert_eq!(
        record_ticks_requesting_shutdown_at(
            FirstTickTiming::Immediate,
            0,
            TickTimeLimit::Unlimited
        )
        .await,
        Ok(vec![ExpectedTickContexts::first_immediate_tick()])
    );
    assert_eq!(
        record_ticks_requesting_shutdown_at(
            FirstTickTiming::Immediate,
            1,
            TickTimeLimit::Unlimited
        )
        .await,
        Ok(vec![
            ExpectedTickContexts::first_immediate_tick(),
            expected_tick_contexts.second_immediate_tick(),
        ])
    );
    assert_eq!(
        record_ticks_requesting_shutdown_at(
            FirstTickTiming::AfterInterval,
            0,
            TickTimeLimit::Unlimited
        )
        .await,
        Ok(vec![expected_tick_contexts.first_delayed_tick()])
    );
    assert_eq!(
        record_ticks_requesting_shutdown_at(
            FirstTickTiming::Immediate,
            0,
            TickTimeLimit::Limited(expected_tick_contexts.tick_period * 2)
        )
        .await,
        Ok(vec![ExpectedTickContexts::first_immediate_tick()])
    );
}
