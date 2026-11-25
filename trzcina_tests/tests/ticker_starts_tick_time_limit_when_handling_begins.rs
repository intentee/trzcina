use std::time::Duration;

use tokio::time::MissedTickBehavior;
use tokio::time::advance;
use tokio_util::sync::CancellationToken;

use trzcina_tests::expected_tick_contexts::ExpectedTickContexts;
use trzcina_tests::first_tick_stalling_ticker::FirstTickStallingTicker;
use trzcina_tests::record_tick_contexts::record_tick_contexts;
use trzcina_tests::test_failure::TestFailure;
use trzcina_ticker::first_tick_timing::FirstTickTiming;
use trzcina_ticker::tick_context::TickContext;
use trzcina_ticker::tick_interval::TickInterval;
use trzcina_ticker::tick_time_limit::TickTimeLimit;
use trzcina_ticker::ticker::Ticker;

const TICKS_BEFORE_CANCELLATION: usize = 1;

async fn record_ticks_with_later_tick_released_after(
    tick_time_limit: Duration,
    later_tick_released_after_handling_begins: Duration,
) -> Result<Vec<TickContext>, TestFailure> {
    let later_tick_release = CancellationToken::new();

    record_tick_contexts(
        |tick_recorder| LaterTickAwaitingTicker {
            first_tick_stalling_ticker: FirstTickStallingTicker {
                first_tick_release: CancellationToken::new(),
                first_tick_timing: FirstTickTiming::Immediate,
                missed_tick_behavior: MissedTickBehavior::Delay,
                tick_recorder,
                tick_time_limit: TickTimeLimit::Limited(tick_time_limit),
            },
            later_tick_release: later_tick_release.clone(),
        },
        |_cancellation_token| async {
            advance(tick_time_limit).await;
            advance(later_tick_released_after_handling_begins).await;
            later_tick_release.cancel();
        },
        TICKS_BEFORE_CANCELLATION,
    )
    .await
}

struct LaterTickAwaitingTicker {
    first_tick_stalling_ticker: FirstTickStallingTicker,
    later_tick_release: CancellationToken,
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

#[tokio::test(start_paused = true)]
async fn starts_tick_time_limit_when_handling_begins() {
    let tick_period = TickInterval::default().period();
    let tick_time_limit = tick_period * 2;
    let third_tick_handled_late = TickContext {
        elapsed_since_start: tick_period * 3,
        ticks_since_start: 2,
        since_last_tick: tick_period * 2,
    };

    assert_eq!(
        record_ticks_with_later_tick_released_after(tick_time_limit, tick_period * 3 / 2).await,
        Ok(vec![
            ExpectedTickContexts { tick_period }.second_immediate_tick(),
            third_tick_handled_late,
        ])
    );
    assert_eq!(
        record_ticks_with_later_tick_released_after(
            tick_time_limit,
            tick_time_limit + tick_period / 2
        )
        .await,
        Ok(vec![third_tick_handled_late])
    );
}
