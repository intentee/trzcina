use std::future::ready;
use std::time::Duration;

use trzcina_tests::move_clock_off_timer_grid::move_clock_off_timer_grid;
use trzcina_tests::recording_ticker::RecordingTicker;
use trzcina_tests::timed_run::TimedRun;
use trzcina_ticker::first_tick_timing::FirstTickTiming;
use trzcina_ticker::tick_interval::TickInterval;

#[tokio::test(start_paused = true)]
async fn stops_on_cancel_before_delayed_first_tick() {
    move_clock_off_timer_grid().await;

    let TimedRun { elapsed, output } = TimedRun::measure(RecordingTicker::record(
        FirstTickTiming::AfterInterval,
        TickInterval::default(),
        |_cancellation_token| ready(()),
        0,
    ))
    .await;

    assert_eq!(output, Ok(vec![]));
    assert_eq!(elapsed, Duration::ZERO);
}
