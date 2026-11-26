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
use trzcina_tokio_safe_delay::constants::TOKIO_SAFE_DELAY_CAP;

const OBSERVED_TICKS: usize = 1;

async fn record_ticks_after_first_tick_stalls_beyond(
    first_tick_timing: FirstTickTiming,
    tick_time_limit: Duration,
) -> TimedRun<Result<Vec<TickContext>, TestFailure>> {
    TimedRun::measure(record_tick_contexts(
        |tick_recorder| FirstTickStallingTicker {
            first_tick_began: CancellationToken::new(),
            first_tick_release: CancellationToken::new(),
            first_tick_timing,
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
    let expected_tick_contexts = ExpectedTickContexts { tick_period };
    let second_tick = expected_tick_contexts.second_immediate_tick();
    let ticks_after_skipped_delayed_first_tick = expected_tick_contexts
        .second_delayed_tick()
        .map(|second_delayed_tick| vec![second_delayed_tick]);
    let tick_time_limit = tick_period * 2;
    let TimedRun {
        elapsed: elapsed_with_time_limit,
        output: ticks_with_time_limit,
    } = record_ticks_after_first_tick_stalls_beyond(FirstTickTiming::Immediate, tick_time_limit)
        .await;
    let TimedRun {
        elapsed: elapsed_with_zero_time_limit,
        output: ticks_with_zero_time_limit,
    } = record_ticks_after_first_tick_stalls_beyond(FirstTickTiming::Immediate, Duration::ZERO)
        .await;
    let TimedRun {
        elapsed: elapsed_with_time_limit_at_cap,
        output: ticks_with_time_limit_at_cap,
    } = record_ticks_after_first_tick_stalls_beyond(
        FirstTickTiming::Immediate,
        TOKIO_SAFE_DELAY_CAP,
    )
    .await;
    let TimedRun {
        elapsed: elapsed_with_delayed_first_tick,
        output: ticks_with_delayed_first_tick,
    } = record_ticks_after_first_tick_stalls_beyond(
        FirstTickTiming::AfterInterval,
        tick_time_limit,
    )
    .await;
    let TimedRun {
        elapsed: elapsed_with_delayed_first_tick_and_zero_time_limit,
        output: ticks_with_delayed_first_tick_and_zero_time_limit,
    } = record_ticks_after_first_tick_stalls_beyond(FirstTickTiming::AfterInterval, Duration::ZERO)
        .await;
    let TimedRun {
        elapsed: elapsed_with_delayed_first_tick_and_time_limit_at_cap,
        output: ticks_with_delayed_first_tick_and_time_limit_at_cap,
    } = record_ticks_after_first_tick_stalls_beyond(
        FirstTickTiming::AfterInterval,
        TOKIO_SAFE_DELAY_CAP,
    )
    .await;

    assert_eq!(ticks_with_time_limit, Ok(vec![second_tick]));
    assert_eq!(elapsed_with_time_limit, tick_time_limit);
    assert_eq!(ticks_with_zero_time_limit, Ok(vec![second_tick]));
    assert_eq!(elapsed_with_zero_time_limit, tick_period);
    assert_eq!(ticks_with_time_limit_at_cap, Ok(vec![second_tick]));
    assert_eq!(elapsed_with_time_limit_at_cap, TOKIO_SAFE_DELAY_CAP);
    assert_eq!(
        ticks_with_delayed_first_tick,
        ticks_after_skipped_delayed_first_tick
    );
    assert_eq!(
        elapsed_with_delayed_first_tick,
        tick_period + tick_time_limit
    );
    assert_eq!(
        ticks_with_delayed_first_tick_and_zero_time_limit,
        ticks_after_skipped_delayed_first_tick
    );
    assert_eq!(
        elapsed_with_delayed_first_tick_and_zero_time_limit,
        tick_period * 2
    );
    assert_eq!(
        ticks_with_delayed_first_tick_and_time_limit_at_cap,
        ticks_after_skipped_delayed_first_tick
    );
    assert_eq!(
        elapsed_with_delayed_first_tick_and_time_limit_at_cap,
        tick_period + TOKIO_SAFE_DELAY_CAP
    );
}
