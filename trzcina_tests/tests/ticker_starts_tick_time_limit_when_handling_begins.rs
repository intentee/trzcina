use std::time::Duration;

use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::time::MissedTickBehavior;
use tokio::time::advance;
use tokio_util::sync::CancellationToken;

use trzcina_tests::expected_tick_contexts::ExpectedTickContexts;
use trzcina_tests::first_tick_stalling_ticker::FirstTickStallingTicker;
use trzcina_tests::record_tick_contexts::record_tick_contexts;
use trzcina_tests::test_failure::TestFailure;
use trzcina_tests::tick_recorder::TickRecorder;
use trzcina_ticker::first_tick_timing::FirstTickTiming;
use trzcina_ticker::tick_context::TickContext;
use trzcina_ticker::tick_interval::TickInterval;
use trzcina_ticker::tick_time_limit::TickTimeLimit;
use trzcina_ticker::ticker::Ticker;

const TICKS_BEFORE_CANCELLATION: usize = 1;

async fn record_ticks_with_later_tick_released<TBeforeRelease>(
    tick_time_limit: Duration,
    before_release: TBeforeRelease,
) -> Result<Vec<TickContext>, TestFailure>
where
    TBeforeRelease: AsyncFnOnce(&mut UnboundedReceiver<TickContext>),
{
    let later_tick_release = CancellationToken::new();
    let (later_tick_began_sender, mut later_tick_began) = mpsc::unbounded_channel();

    record_tick_contexts(
        |tick_recorder| LaterTickAwaitingTicker {
            first_tick_stalling_ticker: FirstTickStallingTicker {
                first_tick_began: CancellationToken::new(),
                first_tick_release: CancellationToken::new(),
                first_tick_timing: FirstTickTiming::Immediate,
                missed_tick_behavior: MissedTickBehavior::Delay,
                tick_recorder,
                tick_time_limit: TickTimeLimit::Limited(tick_time_limit),
            },
            later_tick_began: later_tick_began_sender.into(),
            later_tick_release: later_tick_release.clone(),
        },
        |_cancellation_token| async {
            before_release(&mut later_tick_began).await;
            later_tick_release.cancel();
        },
        TICKS_BEFORE_CANCELLATION,
    )
    .await
}

struct LaterTickAwaitingTicker {
    first_tick_stalling_ticker: FirstTickStallingTicker,
    later_tick_began: TickRecorder,
    later_tick_release: CancellationToken,
}

impl Ticker for LaterTickAwaitingTicker {
    type Error = TestFailure;

    fn first_tick_timing(&self) -> FirstTickTiming {
        self.first_tick_stalling_ticker.first_tick_timing()
    }

    async fn handle_tick(
        &mut self,
        cancellation_token: CancellationToken,
        tick_context @ TickContext {
            ticks_since_start, ..
        }: TickContext,
    ) -> Result<(), TestFailure> {
        if ticks_since_start > 0 {
            self.later_tick_began.record(tick_context)?;
            self.later_tick_release.cancelled().await;
        }

        self.first_tick_stalling_ticker
            .handle_tick(cancellation_token, tick_context)
            .await
    }

    fn missed_tick_behavior(&self) -> MissedTickBehavior {
        self.first_tick_stalling_ticker.missed_tick_behavior()
    }

    fn tick_time_limit(&self) -> TickTimeLimit {
        self.first_tick_stalling_ticker.tick_time_limit()
    }
}

#[tokio::test(start_paused = true)]
async fn starts_tick_time_limit_when_handling_begins() {
    let tick_period = TickInterval::default().period();
    let tick_time_limit = tick_period * 2;
    let second_tick = ExpectedTickContexts { tick_period }.second_immediate_tick();
    let third_tick_handled_late = TickContext {
        elapsed_since_start: tick_period * 3,
        ticks_since_start: 2,
        since_last_tick: tick_period * 2,
    };

    assert_eq!(
        record_ticks_with_later_tick_released(tick_time_limit, async |later_tick_began| {
            later_tick_began.recv().await;
            advance(tick_period * 3 / 2).await;
        })
        .await,
        Ok(vec![second_tick, third_tick_handled_late])
    );
    assert_eq!(
        record_ticks_with_later_tick_released(tick_time_limit, async |later_tick_began| {
            later_tick_began.recv().await;
            later_tick_began.recv().await;
        })
        .await,
        Ok(vec![third_tick_handled_late])
    );
    assert_eq!(
        record_ticks_with_later_tick_released(tick_period / 2, async |later_tick_began| {
            later_tick_began.recv().await;
            advance(tick_period / 4).await;
        })
        .await,
        Ok(vec![second_tick])
    );
}
