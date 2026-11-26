use std::future::ready;
use std::time::Duration;

use tokio::time::advance;

use trzcina_tests::recording_ticker::RecordingTicker;
use trzcina_tests::timed_run::TimedRun;
use trzcina_ticker::first_tick_timing::FirstTickTiming;
use trzcina_ticker::tick_interval::TickInterval;
use trzcina_ticker::tick_time_limit::TickTimeLimit;

use crate::constants::SMALLEST_DURATION_STEP;

#[tokio::test(start_paused = true)]
async fn stops_on_cancel_before_delayed_first_tick() {
    advance(SMALLEST_DURATION_STEP).await;

    let TimedRun { elapsed, output } = TimedRun::measure(RecordingTicker::record(
        FirstTickTiming::AfterInterval,
        TickInterval::default(),
        TickTimeLimit::Unlimited,
        |_cancellation_token| ready(()),
        0,
    ))
    .await;

    assert_eq!(output, Ok(vec![]));
    assert_eq!(elapsed, Duration::ZERO);
}
