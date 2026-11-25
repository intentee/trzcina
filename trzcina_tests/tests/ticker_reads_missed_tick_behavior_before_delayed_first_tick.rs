use std::future::ready;

use tokio::sync::watch;
use tokio::time::MissedTickBehavior;
use tokio::time::advance;
use tokio_util::sync::CancellationToken;

use trzcina_tests::expected_tick_contexts::ExpectedTickContexts;
use trzcina_tests::move_clock_off_timer_grid::move_clock_off_timer_grid;
use trzcina_tests::record_tick_contexts::record_tick_contexts;
use trzcina_tests::test_failure::TestFailure;
use trzcina_tests::tick_recorder::TickRecorder;
use trzcina_ticker::first_tick_timing::FirstTickTiming;
use trzcina_ticker::tick_context::TickContext;
use trzcina_ticker::tick_interval::TickInterval;
use trzcina_ticker::ticker::Ticker;

const TICKS_BEFORE_CANCELLATION: usize = 2;

struct ExternallyScheduledTicker {
    missed_tick_behavior: watch::Receiver<MissedTickBehavior>,
    tick_recorder: TickRecorder,
}

impl Ticker for ExternallyScheduledTicker {
    type Error = TestFailure;

    fn first_tick_timing(&self) -> FirstTickTiming {
        FirstTickTiming::AfterInterval
    }

    fn handle_tick(
        &mut self,
        _cancellation_token: CancellationToken,
        tick_context: TickContext,
    ) -> impl Future<Output = Result<(), TestFailure>> + Send {
        ready(self.tick_recorder.record(tick_context))
    }

    fn missed_tick_behavior(&self) -> MissedTickBehavior {
        *self.missed_tick_behavior.borrow()
    }
}

#[tokio::test(start_paused = true)]
async fn reads_missed_tick_behavior_before_delayed_first_tick() {
    let tick_period = TickInterval::default().period();
    let expected_tick_contexts = ExpectedTickContexts { tick_period };
    let (missed_tick_behavior_sender, missed_tick_behavior) =
        watch::channel(MissedTickBehavior::Burst);

    move_clock_off_timer_grid().await;

    assert_eq!(
        record_tick_contexts(
            |tick_recorder| ExternallyScheduledTicker {
                missed_tick_behavior,
                tick_recorder,
            },
            |_cancellation_token| async {
                missed_tick_behavior_sender.send_modify(|missed_tick_behavior| {
                    *missed_tick_behavior = MissedTickBehavior::Skip;
                });
                advance(tick_period * 5 / 2).await;
            },
            TICKS_BEFORE_CANCELLATION,
        )
        .await,
        expected_tick_contexts
            .second_delayed_tick()
            .map(|second_delayed_tick| vec![
                expected_tick_contexts.first_delayed_tick(),
                second_delayed_tick,
            ])
    );
}
