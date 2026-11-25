use std::future::ready;
use std::time::Duration;

use trzcina_tests::expected_tick_contexts::ExpectedTickContexts;
use trzcina_tests::recording_ticker::RecordingTicker;
use trzcina_tests::test_failure::TestFailure;
use trzcina_ticker::first_tick_timing::FirstTickTiming;
use trzcina_ticker::tick_context::TickContext;
use trzcina_ticker::tick_interval::TickInterval;

async fn record_three_ticks(tick_period: Duration) -> Result<Vec<TickContext>, TestFailure> {
    RecordingTicker::record(
        FirstTickTiming::Immediate,
        TickInterval::new(tick_period)?,
        |_cancellation_token| ready(()),
        3,
    )
    .await
}

#[tokio::test(start_paused = true)]
async fn fires_ticks_at_configured_interval() {
    let tick_period = TickInterval::default().period() * 2;
    let expected_tick_contexts = ExpectedTickContexts { tick_period };

    assert_eq!(
        record_three_ticks(tick_period).await,
        expected_tick_contexts
            .third_immediate_tick()
            .map(|third_immediate_tick| vec![
                ExpectedTickContexts::first_immediate_tick(),
                expected_tick_contexts.second_immediate_tick(),
                third_immediate_tick,
            ])
    );
}
