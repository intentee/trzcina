use std::future::ready;
use std::time::Duration;

use tokio::time::MissedTickBehavior;
use tokio_util::sync::CancellationToken;

use trzcina_tests::expected_tick_contexts::ExpectedTickContexts;
use trzcina_tests::first_tick_stalling_ticker::FirstTickStallingTicker;
use trzcina_tests::install_trace_logging::install_trace_logging;
use trzcina_tests::record_tick_contexts::record_tick_contexts;
use trzcina_tests::test_failure::TestFailure;
use trzcina_tests::timed_run::TimedRun;
use trzcina_ticker::first_tick_timing::FirstTickTiming;
use trzcina_ticker::tick_context::TickContext;
use trzcina_ticker::tick_interval::TickInterval;
use trzcina_ticker::tick_time_limit::TickTimeLimit;

const OBSERVED_TICKS: usize = 1;

async fn record_ticks_after_first_tick_stalls_beyond(
    tick_time_limit: Duration,
) -> TimedRun<Result<Vec<TickContext>, TestFailure>> {
    TimedRun::measure(record_tick_contexts(
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
    .await
}

#[tokio::test(start_paused = true)]
async fn skips_tick_that_exceeds_time_limit() {
    assert!(install_trace_logging().is_ok());

    let tick_period = TickInterval::default().period();
    let second_tick = ExpectedTickContexts { tick_period }.second_immediate_tick();
    let tick_time_limit = tick_period * 2;
    let TimedRun {
        elapsed: elapsed_with_time_limit,
        output: ticks_with_time_limit,
    } = record_ticks_after_first_tick_stalls_beyond(tick_time_limit).await;
    let TimedRun {
        elapsed: elapsed_with_zero_time_limit,
        output: ticks_with_zero_time_limit,
    } = record_ticks_after_first_tick_stalls_beyond(Duration::ZERO).await;

    assert_eq!(ticks_with_time_limit, Ok(vec![second_tick]));
    assert_eq!(elapsed_with_time_limit, tick_time_limit);
    assert_eq!(ticks_with_zero_time_limit, Ok(vec![second_tick]));
    assert_eq!(elapsed_with_zero_time_limit, tick_period);
}
