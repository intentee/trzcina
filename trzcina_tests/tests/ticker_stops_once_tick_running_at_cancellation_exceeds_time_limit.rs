use std::time::Duration;

use tokio::sync::mpsc;
use tokio::time::MissedTickBehavior;
use tokio_util::sync::CancellationToken;

use trzcina_tests::expected_tick_contexts::ExpectedTickContexts;
use trzcina_tests::first_tick_stalling_ticker::FirstTickStallingTicker;
use trzcina_tests::later_tick_awaiting_ticker::LaterTickAwaitingTicker;
use trzcina_tests::record_tick_contexts::record_tick_contexts;
use trzcina_tests::test_failure::TestFailure;
use trzcina_tests::timed_run::TimedRun;
use trzcina_ticker::first_tick_timing::FirstTickTiming;
use trzcina_ticker::tick_context::TickContext;
use trzcina_ticker::tick_interval::TickInterval;
use trzcina_ticker::tick_time_limit::TickTimeLimit;
use trzcina_tokio_safe_delay::constants::TOKIO_SAFE_DELAY_CAP;

const TICKS_BEFORE_CANCELLATION: usize = 0;

async fn record_ticks_until_first_tick_running_at_cancellation_stops(
    first_tick_timing: FirstTickTiming,
    tick_time_limit: Duration,
) -> TimedRun<Result<Vec<TickContext>, TestFailure>> {
    let first_tick_began = CancellationToken::new();

    TimedRun::measure(record_tick_contexts(
        |tick_recorder| FirstTickStallingTicker {
            first_tick_began: first_tick_began.clone(),
            first_tick_release: CancellationToken::new(),
            first_tick_timing,
            missed_tick_behavior: MissedTickBehavior::Delay,
            tick_recorder,
            tick_time_limit: TickTimeLimit::Limited(tick_time_limit),
        },
        |_cancellation_token| first_tick_began.cancelled(),
        TICKS_BEFORE_CANCELLATION,
    ))
    .await
}

async fn record_ticks_until_later_tick_running_at_cancellation_stops(
    tick_time_limit: Duration,
) -> TimedRun<Result<Vec<TickContext>, TestFailure>> {
    let first_tick_release = CancellationToken::new();
    let (later_tick_began_sender, mut later_tick_began) = mpsc::unbounded_channel();

    first_tick_release.cancel();

    TimedRun::measure(record_tick_contexts(
        |tick_recorder| LaterTickAwaitingTicker {
            first_tick_stalling_ticker: FirstTickStallingTicker {
                first_tick_began: CancellationToken::new(),
                first_tick_release,
                first_tick_timing: FirstTickTiming::Immediate,
                missed_tick_behavior: MissedTickBehavior::Delay,
                tick_recorder,
                tick_time_limit: TickTimeLimit::Limited(tick_time_limit),
            },
            later_tick_began: later_tick_began_sender.into(),
            later_tick_release: CancellationToken::new(),
        },
        |_cancellation_token| async move {
            later_tick_began.recv().await;
        },
        TICKS_BEFORE_CANCELLATION,
    ))
    .await
}

#[tokio::test(start_paused = true)]
async fn stops_once_tick_running_at_cancellation_exceeds_time_limit() {
    let tick_period = TickInterval::default().period();
    let tick_time_limit = tick_period * 2;
    let TimedRun {
        elapsed: elapsed_with_first_tick_running,
        output: ticks_with_first_tick_running,
    } = record_ticks_until_first_tick_running_at_cancellation_stops(
        FirstTickTiming::Immediate,
        tick_time_limit,
    )
    .await;
    let TimedRun {
        elapsed: elapsed_with_delayed_first_tick_running,
        output: ticks_with_delayed_first_tick_running,
    } = record_ticks_until_first_tick_running_at_cancellation_stops(
        FirstTickTiming::AfterInterval,
        tick_time_limit,
    )
    .await;
    let TimedRun {
        elapsed: elapsed_with_later_tick_running,
        output: ticks_with_later_tick_running,
    } = record_ticks_until_later_tick_running_at_cancellation_stops(tick_time_limit).await;
    let TimedRun {
        elapsed: elapsed_with_later_tick_beyond_zero_limit,
        output: ticks_with_later_tick_beyond_zero_limit,
    } = record_ticks_until_later_tick_running_at_cancellation_stops(Duration::ZERO).await;
    let TimedRun {
        elapsed: elapsed_with_later_tick_at_cap_limit,
        output: ticks_with_later_tick_at_cap_limit,
    } = record_ticks_until_later_tick_running_at_cancellation_stops(TOKIO_SAFE_DELAY_CAP).await;

    assert_eq!(ticks_with_first_tick_running, Ok(vec![]));
    assert_eq!(elapsed_with_first_tick_running, tick_time_limit);
    assert_eq!(ticks_with_delayed_first_tick_running, Ok(vec![]));
    assert_eq!(
        elapsed_with_delayed_first_tick_running,
        tick_period + tick_time_limit
    );
    assert_eq!(
        ticks_with_later_tick_running,
        Ok(vec![ExpectedTickContexts::first_immediate_tick()])
    );
    assert_eq!(
        elapsed_with_later_tick_running,
        tick_period + tick_time_limit
    );
    assert_eq!(
        ticks_with_later_tick_beyond_zero_limit,
        Ok(vec![ExpectedTickContexts::first_immediate_tick()])
    );
    assert_eq!(elapsed_with_later_tick_beyond_zero_limit, tick_period);
    assert_eq!(
        ticks_with_later_tick_at_cap_limit,
        Ok(vec![ExpectedTickContexts::first_immediate_tick()])
    );
    assert_eq!(
        elapsed_with_later_tick_at_cap_limit,
        tick_period + TOKIO_SAFE_DELAY_CAP
    );
}
