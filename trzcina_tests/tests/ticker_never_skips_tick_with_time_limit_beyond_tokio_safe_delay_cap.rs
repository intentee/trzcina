use std::time::Duration;

use tokio::sync::mpsc;
use tokio::time::MissedTickBehavior;
use tokio::time::advance;
use tokio_util::sync::CancellationToken;

use trzcina_tests::expected_tick_contexts::ExpectedTickContexts;
use trzcina_tests::first_tick_stalling_ticker::FirstTickStallingTicker;
use trzcina_tests::later_tick_awaiting_ticker::LaterTickAwaitingTicker;
use trzcina_tests::record_tick_contexts::record_tick_contexts;
use trzcina_tests::test_failure::TestFailure;
use trzcina_ticker::first_tick_timing::FirstTickTiming;
use trzcina_ticker::tick_context::TickContext;
use trzcina_ticker::tick_interval::TickInterval;
use trzcina_ticker::tick_time_limit::TickTimeLimit;
use trzcina_tokio_safe_delay::constants::TOKIO_SAFE_DELAY_CAP;

use crate::constants::SMALLEST_DURATION_STEP;
use crate::constants::TOKIO_TIMER_GRANULARITY;

const TICKS_BEFORE_CANCELLATION: usize = 1;

async fn record_ticks_each_stalling_beyond_tick_time_limit(
    first_tick_timing: FirstTickTiming,
    tick_time_limit: Duration,
    tick_stall: Duration,
) -> Result<Vec<TickContext>, TestFailure> {
    let first_tick_release = CancellationToken::new();
    let later_tick_release = CancellationToken::new();
    let (later_tick_began_sender, mut later_tick_began) = mpsc::unbounded_channel();

    record_tick_contexts(
        |tick_recorder| LaterTickAwaitingTicker {
            first_tick_stalling_ticker: FirstTickStallingTicker {
                first_tick_began: CancellationToken::new(),
                first_tick_release: first_tick_release.clone(),
                first_tick_timing,
                missed_tick_behavior: MissedTickBehavior::Delay,
                tick_recorder,
                tick_time_limit: TickTimeLimit::Limited(tick_time_limit),
            },
            later_tick_began: later_tick_began_sender.into(),
            later_tick_release: later_tick_release.clone(),
        },
        |_cancellation_token| async {
            advance(tick_stall).await;
            first_tick_release.cancel();
            later_tick_began.recv().await;
            advance(tick_stall).await;
            later_tick_release.cancel();
        },
        TICKS_BEFORE_CANCELLATION,
    )
    .await
}

#[tokio::test(start_paused = true)]
async fn never_skips_tick_with_time_limit_beyond_tokio_safe_delay_cap() {
    let tick_period = TickInterval::default().period();
    let expected_tick_contexts = ExpectedTickContexts { tick_period };
    let tick_time_limit = TOKIO_SAFE_DELAY_CAP + SMALLEST_DURATION_STEP;
    let tick_stall = TOKIO_SAFE_DELAY_CAP + TOKIO_TIMER_GRANULARITY;

    assert_eq!(
        record_ticks_each_stalling_beyond_tick_time_limit(
            FirstTickTiming::Immediate,
            tick_time_limit,
            tick_stall
        )
        .await,
        Ok(vec![
            ExpectedTickContexts::first_immediate_tick(),
            expected_tick_contexts.second_immediate_tick(),
        ])
    );
    assert_eq!(
        record_ticks_each_stalling_beyond_tick_time_limit(
            FirstTickTiming::AfterInterval,
            tick_time_limit,
            tick_stall
        )
        .await,
        Ok(vec![
            expected_tick_contexts.first_delayed_tick(),
            TickContext {
                elapsed_since_start: tick_stall + tick_period,
                ticks_since_start: 1,
                since_last_tick: tick_stall,
            },
        ])
    );
}
