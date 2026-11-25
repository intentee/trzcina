use tokio::time::MissedTickBehavior;
use tokio::time::advance;
use tokio_util::sync::CancellationToken;

use trzcina::constants::TOKIO_SAFE_DELAY_CAP;
use trzcina_tests::expected_tick_contexts::ExpectedTickContexts;
use trzcina_tests::first_tick_stalling_ticker::FirstTickStallingTicker;
use trzcina_tests::record_tick_contexts::record_tick_contexts;
use trzcina_ticker::first_tick_timing::FirstTickTiming;
use trzcina_ticker::tick_interval::TickInterval;
use trzcina_ticker::tick_time_limit::TickTimeLimit;

use crate::constants::SMALLEST_DURATION_STEP;
use crate::constants::TOKIO_TIMER_GRANULARITY;

const TICKS_BEFORE_CANCELLATION: usize = 1;

#[tokio::test(start_paused = true)]
async fn never_skips_tick_with_time_limit_beyond_tokio_safe_delay_cap() {
    let first_tick_release = CancellationToken::new();

    assert_eq!(
        record_tick_contexts(
            |tick_recorder| FirstTickStallingTicker {
                first_tick_release: first_tick_release.clone(),
                first_tick_timing: FirstTickTiming::Immediate,
                missed_tick_behavior: MissedTickBehavior::Delay,
                tick_recorder,
                tick_time_limit: TickTimeLimit::Limited(
                    TOKIO_SAFE_DELAY_CAP + SMALLEST_DURATION_STEP
                ),
            },
            |_cancellation_token| async {
                advance(TOKIO_SAFE_DELAY_CAP + TOKIO_TIMER_GRANULARITY).await;
                first_tick_release.cancel();
            },
            TICKS_BEFORE_CANCELLATION,
        )
        .await,
        Ok(vec![
            ExpectedTickContexts::first_immediate_tick(),
            ExpectedTickContexts {
                tick_period: TickInterval::default().period()
            }
            .second_immediate_tick(),
        ])
    );
}
