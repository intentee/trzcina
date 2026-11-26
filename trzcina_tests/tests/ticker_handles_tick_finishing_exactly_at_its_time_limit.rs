use std::future::ready;
use std::time::Duration;

use trzcina_tests::failing_ticker::FailingTicker;
use trzcina_tests::observed_run::ObservedRun;
use trzcina_tests::test_failure::TestFailure;
use trzcina_ticker::first_tick_timing::FirstTickTiming;
use trzcina_ticker::tick_time_limit::TickTimeLimit;

#[tokio::test(start_paused = true)]
async fn handles_tick_finishing_exactly_at_its_time_limit() {
    let ObservedRun { run_output, .. } = ObservedRun::tick_until_observation(
        FailingTicker {
            failing_tick: 0,
            first_tick_timing: FirstTickTiming::Immediate,
            tick_time_limit: TickTimeLimit::Limited(Duration::ZERO),
        },
        |_cancellation_token| ready(()),
    )
    .await;

    assert_eq!(run_output, Err(TestFailure::DeliberateTickFailure));
}
