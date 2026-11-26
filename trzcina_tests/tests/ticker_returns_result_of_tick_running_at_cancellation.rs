use tokio::time::advance;
use tokio_util::sync::CancellationToken;

use trzcina_tests::observed_run::ObservedRun;
use trzcina_tests::test_failure::TestFailure;
use trzcina_ticker::tick_context::TickContext;
use trzcina_ticker::tick_interval::TickInterval;
use trzcina_ticker::tick_time_limit::TickTimeLimit;
use trzcina_ticker::ticker::Ticker;

async fn result_of_tick_released_after_cancellation(
    failing_tick: u64,
    tick_time_limit: TickTimeLimit,
) -> Result<(), TestFailure> {
    let failing_tick_began = CancellationToken::new();
    let release = CancellationToken::new();
    let ObservedRun { run_output, .. } = ObservedRun::tick_until_observation(
        ReleaseAwaitingFailingTicker {
            failing_tick,
            failing_tick_began: failing_tick_began.clone(),
            release: release.clone(),
            tick_time_limit,
        },
        |cancellation_token| async move {
            failing_tick_began.cancelled().await;
            cancellation_token.cancel();
            advance(TickInterval::default().period()).await;
            release.cancel();
        },
    )
    .await;

    run_output
}

struct ReleaseAwaitingFailingTicker {
    failing_tick: u64,
    failing_tick_began: CancellationToken,
    release: CancellationToken,
    tick_time_limit: TickTimeLimit,
}

impl Ticker for ReleaseAwaitingFailingTicker {
    type Error = TestFailure;

    async fn handle_tick(
        &mut self,
        _cancellation_token: CancellationToken,
        TickContext {
            ticks_since_start, ..
        }: TickContext,
    ) -> Result<(), TestFailure> {
        if ticks_since_start < self.failing_tick {
            return Ok(());
        }

        self.failing_tick_began.cancel();
        self.release.cancelled().await;

        Err(TestFailure::DeliberateTickFailure)
    }

    fn tick_time_limit(&self) -> TickTimeLimit {
        self.tick_time_limit
    }
}

#[tokio::test(start_paused = true)]
async fn returns_result_of_tick_running_at_cancellation() {
    let tick_period = TickInterval::default().period();

    assert_eq!(
        result_of_tick_released_after_cancellation(0, TickTimeLimit::Unlimited).await,
        Err(TestFailure::DeliberateTickFailure)
    );
    assert_eq!(
        result_of_tick_released_after_cancellation(0, TickTimeLimit::Limited(tick_period * 2))
            .await,
        Err(TestFailure::DeliberateTickFailure)
    );
    assert_eq!(
        result_of_tick_released_after_cancellation(1, TickTimeLimit::Unlimited).await,
        Err(TestFailure::DeliberateTickFailure)
    );
}
