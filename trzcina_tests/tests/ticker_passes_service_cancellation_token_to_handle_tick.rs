use std::future::ready;

use tokio_util::sync::CancellationToken;

use trzcina_tests::expected_tick_contexts::ExpectedTickContexts;
use trzcina_tests::record_tick_contexts::record_tick_contexts;
use trzcina_tests::test_failure::TestFailure;
use trzcina_tests::tick_recorder::TickRecorder;
use trzcina_ticker::tick_context::TickContext;
use trzcina_ticker::tick_interval::TickInterval;
use trzcina_ticker::ticker::Ticker;

const TICKS_BEFORE_CANCELLATION: usize = 3;

async fn record_ticks_requesting_shutdown_at(
    shutdown_requesting_tick: u64,
) -> Result<Vec<TickContext>, TestFailure> {
    record_tick_contexts(
        |tick_recorder| ShutdownRequestingTicker {
            shutdown_requesting_tick,
            tick_recorder,
        },
        |_cancellation_token| ready(()),
        TICKS_BEFORE_CANCELLATION,
    )
    .await
}

struct ShutdownRequestingTicker {
    shutdown_requesting_tick: u64,
    tick_recorder: TickRecorder,
}

impl Ticker for ShutdownRequestingTicker {
    type Error = TestFailure;

    fn handle_tick(
        &mut self,
        cancellation_token: CancellationToken,
        tick_context @ TickContext {
            ticks_since_start, ..
        }: TickContext,
    ) -> impl Future<Output = Result<(), TestFailure>> + Send {
        if ticks_since_start == self.shutdown_requesting_tick {
            cancellation_token.cancel();
        }

        ready(self.tick_recorder.record(tick_context))
    }
}

#[tokio::test(start_paused = true)]
async fn passes_service_cancellation_token_to_handle_tick() {
    assert_eq!(
        record_ticks_requesting_shutdown_at(0).await,
        Ok(vec![ExpectedTickContexts::first_immediate_tick()])
    );
    assert_eq!(
        record_ticks_requesting_shutdown_at(1).await,
        Ok(vec![
            ExpectedTickContexts::first_immediate_tick(),
            ExpectedTickContexts {
                tick_period: TickInterval::default().period()
            }
            .second_immediate_tick(),
        ])
    );
}
