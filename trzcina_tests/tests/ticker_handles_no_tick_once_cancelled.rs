use tokio::time::advance;

use trzcina_tests::expected_tick_contexts::ExpectedTickContexts;
use trzcina_tests::recording_ticker::RecordingTicker;
use trzcina_ticker::first_tick_timing::FirstTickTiming;
use trzcina_ticker::tick_interval::TickInterval;

const TICKS_BEFORE_CANCELLATION: usize = 0;

#[tokio::test(start_paused = true)]
async fn handles_no_tick_once_cancelled() {
    let tick_interval = TickInterval::default();

    assert_eq!(
        RecordingTicker::record(
            FirstTickTiming::Immediate,
            tick_interval,
            |cancellation_token| async move {
                cancellation_token.cancel();
                advance(tick_interval.period()).await;
            },
            TICKS_BEFORE_CANCELLATION,
        )
        .await,
        Ok(vec![ExpectedTickContexts::first_immediate_tick()])
    );
}
