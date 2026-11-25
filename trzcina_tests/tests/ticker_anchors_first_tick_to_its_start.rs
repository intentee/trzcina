use std::future::ready;

use trzcina_tests::expected_tick_contexts::ExpectedTickContexts;
use trzcina_tests::recording_ticker::RecordingTicker;
use trzcina_tests::test_failure::TestFailure;
use trzcina_ticker::first_tick_timing::FirstTickTiming;
use trzcina_ticker::tick_context::TickContext;
use trzcina_ticker::tick_interval::TickInterval;

async fn record_first_tick(
    first_tick_timing: FirstTickTiming,
) -> Result<Vec<TickContext>, TestFailure> {
    RecordingTicker::record(
        first_tick_timing,
        TickInterval::default(),
        |_cancellation_token| ready(()),
        1,
    )
    .await
}

#[tokio::test]
async fn anchors_first_tick_to_its_start() {
    let tick_period = TickInterval::default().period();

    assert_eq!(
        record_first_tick(FirstTickTiming::Immediate).await,
        Ok(vec![ExpectedTickContexts::first_immediate_tick()])
    );
    assert_eq!(
        record_first_tick(FirstTickTiming::AfterInterval).await,
        Ok(vec![
            ExpectedTickContexts { tick_period }.first_delayed_tick()
        ])
    );
}
