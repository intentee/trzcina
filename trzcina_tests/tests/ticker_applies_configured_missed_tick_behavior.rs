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

const TICKS_BEFORE_CANCELLATION: usize = 3;

const fn third_tick(elapsed_since_start: Duration, since_last_tick: Duration) -> TickContext {
    TickContext {
        elapsed_since_start,
        ticks_since_start: 2,
        since_last_tick,
    }
}

async fn record_ticks_after_first_tick_stalls(
    first_tick_timing: FirstTickTiming,
    missed_tick_behavior: MissedTickBehavior,
    first_tick_stall: Duration,
) -> Result<Vec<TickContext>, TestFailure> {
    let first_tick_began = CancellationToken::new();
    let first_tick_release = CancellationToken::new();

    record_tick_contexts(
        |tick_recorder| FirstTickStallingTicker {
            first_tick_began: first_tick_began.clone(),
            first_tick_release: first_tick_release.clone(),
            first_tick_timing,
            missed_tick_behavior,
            tick_recorder,
            tick_time_limit: TickTimeLimit::Unlimited,
        },
        |_cancellation_token| async {
            first_tick_began.cancelled().await;
            advance(first_tick_stall).await;
            first_tick_release.cancel();
        },
        TICKS_BEFORE_CANCELLATION,
    )
    .await
}

#[tokio::test(start_paused = true)]
async fn applies_configured_missed_tick_behavior() {
    let tick_period = TickInterval::default().period();
    let first_tick_stall = tick_period * 5 / 2;
    let expected_tick_contexts = ExpectedTickContexts { tick_period };

    assert_eq!(
        record_ticks_after_first_tick_stalls(
            FirstTickTiming::Immediate,
            MissedTickBehavior::Skip,
            first_tick_stall,
        )
        .await,
        Ok(vec![
            ExpectedTickContexts::first_immediate_tick(),
            expected_tick_contexts.second_immediate_tick(),
            third_tick(tick_period * 3, tick_period * 2),
        ])
    );
    assert_eq!(
        record_ticks_after_first_tick_stalls(
            FirstTickTiming::Immediate,
            MissedTickBehavior::Delay,
            first_tick_stall,
        )
        .await,
        Ok(vec![
            ExpectedTickContexts::first_immediate_tick(),
            expected_tick_contexts.second_immediate_tick(),
            third_tick(tick_period * 7 / 2, tick_period * 5 / 2),
        ])
    );
    assert_eq!(
        record_ticks_after_first_tick_stalls(
            FirstTickTiming::AfterInterval,
            MissedTickBehavior::Skip,
            first_tick_stall,
        )
        .await,
        expected_tick_contexts
            .second_delayed_tick()
            .map(|second_delayed_tick| vec![
                expected_tick_contexts.first_delayed_tick(),
                second_delayed_tick,
                third_tick(tick_period * 4, tick_period * 2),
            ])
    );
    assert_eq!(
        record_ticks_after_first_tick_stalls(
            FirstTickTiming::AfterInterval,
            MissedTickBehavior::Delay,
            first_tick_stall,
        )
        .await,
        expected_tick_contexts
            .second_delayed_tick()
            .map(|second_delayed_tick| vec![
                expected_tick_contexts.first_delayed_tick(),
                second_delayed_tick,
                third_tick(tick_period * 9 / 2, tick_period * 5 / 2),
            ])
    );
    assert_eq!(
        record_ticks_after_first_tick_stalls(
            FirstTickTiming::Immediate,
            MissedTickBehavior::Burst,
            first_tick_stall,
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
        record_ticks_after_first_tick_stalls(
            FirstTickTiming::AfterInterval,
            MissedTickBehavior::Burst,
            first_tick_stall,
        )
        .await,
        expected_tick_contexts
            .second_delayed_tick()
            .map(|second_delayed_tick| vec![
                expected_tick_contexts.first_delayed_tick(),
                second_delayed_tick,
                third_tick(tick_period * 3, tick_period),
            ])
    );
}
