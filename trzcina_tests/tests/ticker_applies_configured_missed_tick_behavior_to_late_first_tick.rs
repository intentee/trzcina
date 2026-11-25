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

use crate::constants::SMALLEST_DURATION_STEP;

const TICKS_BEFORE_CANCELLATION: usize = 2;

async fn record_ticks_after_first_tick_is_processed_late(
    first_tick_timing: FirstTickTiming,
    missed_tick_behavior: MissedTickBehavior,
    first_tick_processed_at: Duration,
) -> Result<Vec<TickContext>, TestFailure> {
    let first_tick_release = CancellationToken::new();

    first_tick_release.cancel();
    advance(SMALLEST_DURATION_STEP).await;

    record_tick_contexts(
        |tick_recorder| FirstTickStallingTicker {
            first_tick_began: CancellationToken::new(),
            first_tick_release,
            first_tick_timing,
            missed_tick_behavior,
            tick_recorder,
            tick_time_limit: TickTimeLimit::Unlimited,
        },
        |_cancellation_token| advance(first_tick_processed_at),
        TICKS_BEFORE_CANCELLATION,
    )
    .await
}

#[tokio::test(start_paused = true)]
async fn applies_configured_missed_tick_behavior_to_late_first_tick() {
    let tick_period = TickInterval::default().period();
    let first_tick_processed_at = tick_period * 5 / 2;
    let expected_tick_contexts = ExpectedTickContexts { tick_period };

    assert_eq!(
        record_ticks_after_first_tick_is_processed_late(
            FirstTickTiming::Immediate,
            MissedTickBehavior::Skip,
            first_tick_processed_at,
        )
        .await,
        Ok(vec![
            ExpectedTickContexts::first_immediate_tick(),
            TickContext {
                elapsed_since_start: tick_period * 3,
                ticks_since_start: 1,
                since_last_tick: tick_period * 3,
            },
        ])
    );
    assert_eq!(
        record_ticks_after_first_tick_is_processed_late(
            FirstTickTiming::Immediate,
            MissedTickBehavior::Delay,
            first_tick_processed_at,
        )
        .await,
        Ok(vec![
            ExpectedTickContexts::first_immediate_tick(),
            TickContext {
                elapsed_since_start: tick_period * 7 / 2,
                ticks_since_start: 1,
                since_last_tick: tick_period * 7 / 2,
            },
        ])
    );
    assert_eq!(
        record_ticks_after_first_tick_is_processed_late(
            FirstTickTiming::AfterInterval,
            MissedTickBehavior::Skip,
            first_tick_processed_at,
        )
        .await,
        Ok(vec![
            expected_tick_contexts.first_delayed_tick(),
            TickContext {
                elapsed_since_start: tick_period * 3,
                ticks_since_start: 1,
                since_last_tick: tick_period * 2,
            },
        ])
    );
    assert_eq!(
        record_ticks_after_first_tick_is_processed_late(
            FirstTickTiming::AfterInterval,
            MissedTickBehavior::Delay,
            first_tick_processed_at,
        )
        .await,
        Ok(vec![
            expected_tick_contexts.first_delayed_tick(),
            TickContext {
                elapsed_since_start: tick_period * 7 / 2,
                ticks_since_start: 1,
                since_last_tick: tick_period * 5 / 2,
            },
        ])
    );
    assert_eq!(
        record_ticks_after_first_tick_is_processed_late(
            FirstTickTiming::Immediate,
            MissedTickBehavior::Burst,
            first_tick_processed_at,
        )
        .await,
        expected_tick_contexts
            .third_immediate_tick()
            .map(|third_immediate_tick| vec![
                ExpectedTickContexts::first_immediate_tick(),
                expected_tick_contexts.second_immediate_tick(),
                third_immediate_tick,
            ])
    );
    assert_eq!(
        record_ticks_after_first_tick_is_processed_late(
            FirstTickTiming::AfterInterval,
            MissedTickBehavior::Burst,
            first_tick_processed_at,
        )
        .await,
        expected_tick_contexts
            .second_delayed_tick()
            .map(|second_delayed_tick| vec![
                expected_tick_contexts.first_delayed_tick(),
                second_delayed_tick,
            ])
    );
}
