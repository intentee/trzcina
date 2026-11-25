use tokio::time::MissedTickBehavior;
use tokio_util::sync::CancellationToken;

use trzcina_tests::first_tick_stalling_ticker::FirstTickStallingTicker;
use trzcina_tests::record_tick_contexts::record_tick_contexts;
use trzcina_tests::timed_run::TimedRun;
use trzcina_ticker::first_tick_timing::FirstTickTiming;
use trzcina_ticker::tick_interval::TickInterval;
use trzcina_ticker::tick_time_limit::TickTimeLimit;

const TICKS_BEFORE_CANCELLATION: usize = 0;

#[tokio::test(start_paused = true)]
async fn stops_once_tick_running_at_cancellation_exceeds_time_limit() {
    let first_tick_began = CancellationToken::new();
    let tick_time_limit = TickInterval::default().period() * 2;
    let TimedRun { elapsed, output } = TimedRun::measure(record_tick_contexts(
        |tick_recorder| FirstTickStallingTicker {
            first_tick_began: first_tick_began.clone(),
            first_tick_release: CancellationToken::new(),
            first_tick_timing: FirstTickTiming::Immediate,
            missed_tick_behavior: MissedTickBehavior::Delay,
            tick_recorder,
            tick_time_limit: TickTimeLimit::Limited(tick_time_limit),
        },
        |_cancellation_token| first_tick_began.cancelled(),
        TICKS_BEFORE_CANCELLATION,
    ))
    .await;

    assert_eq!(output, Ok(vec![]));
    assert_eq!(elapsed, tick_time_limit);
}
