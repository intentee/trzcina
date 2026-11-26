use tokio::sync::mpsc;
use tokio::sync::watch;
use tokio_util::sync::CancellationToken;

use trzcina_tests::expected_tick_contexts::ExpectedTickContexts;
use trzcina_tests::record_tick_contexts::record_tick_contexts;
use trzcina_tests::test_failure::TestFailure;
use trzcina_tests::tick_recorder::TickRecorder;
use trzcina_ticker::tick_context::TickContext;
use trzcina_ticker::tick_interval::TickInterval;
use trzcina_ticker::tick_time_limit::TickTimeLimit;
use trzcina_ticker::ticker::Ticker;

const TICKS_BEFORE_CANCELLATION: usize = 1;

struct ExternallyLimitedTicker {
    later_tick_began: TickRecorder,
    tick_recorder: TickRecorder,
    tick_time_limit: watch::Receiver<TickTimeLimit>,
}

impl Ticker for ExternallyLimitedTicker {
    type Error = TestFailure;

    async fn handle_tick(
        &mut self,
        cancellation_token: CancellationToken,
        tick_context @ TickContext {
            ticks_since_start, ..
        }: TickContext,
    ) -> Result<(), TestFailure> {
        if ticks_since_start > 0 {
            self.later_tick_began.record(tick_context)?;
            cancellation_token.cancelled().await;
        }

        self.tick_recorder.record(tick_context)
    }

    fn tick_time_limit(&self) -> TickTimeLimit {
        *self.tick_time_limit.borrow()
    }
}

#[tokio::test(start_paused = true)]
async fn reads_tick_time_limit_when_tick_handling_begins() {
    let tick_period = TickInterval::default().period();
    let expected_tick_contexts = ExpectedTickContexts { tick_period };
    let (tick_time_limit_sender, tick_time_limit) = watch::channel(TickTimeLimit::Unlimited);
    let (later_tick_began_sender, mut later_tick_began) = mpsc::unbounded_channel();

    assert_eq!(
        record_tick_contexts(
            |tick_recorder| ExternallyLimitedTicker {
                later_tick_began: later_tick_began_sender.into(),
                tick_recorder,
                tick_time_limit,
            },
            |_cancellation_token| async {
                tick_time_limit_sender.send_modify(|tick_time_limit| {
                    *tick_time_limit = TickTimeLimit::Limited(tick_period / 2);
                });
                later_tick_began.recv().await;
                later_tick_began.recv().await;
            },
            TICKS_BEFORE_CANCELLATION,
        )
        .await,
        expected_tick_contexts
            .third_immediate_tick()
            .map(|third_immediate_tick| vec![
                ExpectedTickContexts::first_immediate_tick(),
                third_immediate_tick,
            ])
    );
}
