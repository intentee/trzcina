use std::future::ready;
use std::time::Duration;

use trzcina_tests::expected_tick_contexts::ExpectedTickContexts;
use trzcina_tests::recording_ticker::RecordingTicker;
use trzcina_tests::test_failure::TestFailure;
use trzcina_ticker::first_tick_timing::FirstTickTiming;
use trzcina_ticker::tick_context::TickContext;
use trzcina_ticker::tick_interval::TickInterval;
use trzcina_ticker::tick_time_limit::TickTimeLimit;

async fn record_three_ticks(
    first_tick_timing: FirstTickTiming,
    tick_period: Duration,
    tick_time_limit: TickTimeLimit,
) -> Result<Vec<TickContext>, TestFailure> {
    RecordingTicker::record(
        first_tick_timing,
        TickInterval::new(tick_period)?,
        tick_time_limit,
        |_cancellation_token| ready(()),
        3,
    )
    .await
}

#[tokio::test(start_paused = true)]
async fn fires_ticks_at_configured_interval() {
    let tick_period = TickInterval::default().period() * 2;
    let expected_tick_contexts = ExpectedTickContexts { tick_period };

    for tick_time_limit in [
        TickTimeLimit::Unlimited,
        TickTimeLimit::Limited(tick_period),
    ] {
        assert_eq!(
            record_three_ticks(FirstTickTiming::Immediate, tick_period, tick_time_limit).await,
            expected_tick_contexts
                .third_immediate_tick()
                .map(|third_immediate_tick| vec![
                    ExpectedTickContexts::first_immediate_tick(),
                    expected_tick_contexts.second_immediate_tick(),
                    third_immediate_tick,
                ])
        );
    }
    assert_eq!(
        record_three_ticks(
            FirstTickTiming::AfterInterval,
            tick_period,
            TickTimeLimit::Unlimited
        )
        .await,
        expected_tick_contexts
            .second_delayed_tick()
            .map(|second_delayed_tick| vec![
                expected_tick_contexts.first_delayed_tick(),
                second_delayed_tick,
                TickContext {
                    elapsed_since_start: tick_period * 3,
                    ticks_since_start: 2,
                    since_last_tick: tick_period,
                },
            ])
    );
}
