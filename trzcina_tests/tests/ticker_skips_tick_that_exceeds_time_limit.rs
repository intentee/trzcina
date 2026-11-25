use std::future::ready;

use tokio::time::MissedTickBehavior;
use tokio_util::sync::CancellationToken;

use trzcina_tests::expected_tick_contexts::ExpectedTickContexts;
use trzcina_tests::first_tick_stalling_ticker::FirstTickStallingTicker;
use trzcina_tests::install_trace_logging::install_trace_logging;
use trzcina_tests::record_tick_contexts::record_tick_contexts;
use trzcina_tests::timed_run::TimedRun;
use trzcina_ticker::first_tick_timing::FirstTickTiming;
use trzcina_ticker::tick_interval::TickInterval;
use trzcina_ticker::tick_time_limit::TickTimeLimit;

const OBSERVED_TICKS: usize = 1;

#[tokio::test(start_paused = true)]
async fn skips_tick_that_exceeds_time_limit() {
    assert!(install_trace_logging().is_ok());

    let tick_period = TickInterval::default().period();
    let tick_time_limit = tick_period * 2;
    let TimedRun { elapsed, output } = TimedRun::measure(record_tick_contexts(
        |tick_recorder| FirstTickStallingTicker {
            first_tick_began: CancellationToken::new(),
            first_tick_release: CancellationToken::new(),
            first_tick_timing: FirstTickTiming::Immediate,
            missed_tick_behavior: MissedTickBehavior::Delay,
            tick_recorder,
            tick_time_limit: TickTimeLimit::Limited(tick_time_limit),
        },
        |_cancellation_token| ready(()),
        OBSERVED_TICKS,
    ))
    .await;

    assert_eq!(
        output,
        Ok(vec![
            ExpectedTickContexts { tick_period }.second_immediate_tick()
        ])
    );
    assert_eq!(elapsed, tick_time_limit);
}
