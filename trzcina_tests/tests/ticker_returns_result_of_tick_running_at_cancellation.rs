use tokio::time::advance;
use tokio_util::sync::CancellationToken;

use trzcina_tests::observed_run::ObservedRun;
use trzcina_tests::test_failure::TestFailure;
use trzcina_ticker::tick_context::TickContext;
use trzcina_ticker::tick_interval::TickInterval;
use trzcina_ticker::ticker::Ticker;

struct ReleaseAwaitingFailingTicker {
    release: CancellationToken,
}

impl Ticker for ReleaseAwaitingFailingTicker {
    type Error = TestFailure;

    async fn handle_tick(
        &mut self,
        _cancellation_token: CancellationToken,
        _tick_context: TickContext,
    ) -> Result<(), TestFailure> {
        self.release.cancelled().await;

        Err(TestFailure::DeliberateTickFailure)
    }
}

#[tokio::test(start_paused = true)]
async fn returns_result_of_tick_running_at_cancellation() {
    let release = CancellationToken::new();
    let ObservedRun { run_output, .. } = ObservedRun::tick_until_observation(
        ReleaseAwaitingFailingTicker {
            release: release.clone(),
        },
        |cancellation_token| async move {
            cancellation_token.cancel();
            advance(TickInterval::default().period()).await;
            release.cancel();
        },
    )
    .await;

    assert_eq!(run_output, Err(TestFailure::DeliberateTickFailure));
}
