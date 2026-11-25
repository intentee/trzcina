use std::time::Duration;

use tokio::time::MissedTickBehavior;
use tokio::time::advance;
use tokio_util::sync::CancellationToken;

use trzcina_tests::expected_tick_contexts::ExpectedTickContexts;
use trzcina_tests::first_tick_stalling_ticker::FirstTickStallingTicker;
use trzcina_tests::record_tick_contexts::record_tick_contexts;
use trzcina_tests::test_failure::TestFailure;
use trzcina_ticker::first_tick_timing::FirstTickTiming;
use trzcina_ticker::tick_context::TickContext;
use trzcina_ticker::tick_interval::TickInterval;
use trzcina_ticker::tick_time_limit::TickTimeLimit;
use trzcina_ticker::ticker::Ticker;

const TICKS_BEFORE_CANCELLATION: usize = 4;

async fn record_ticks_after_tick_schedule_switches(
    first_tick_stall: Duration,
    switched_tick_period: Duration,
) -> Result<Vec<TickContext>, TestFailure> {
    let first_tick_release = CancellationToken::new();
    let switched_tick_interval = TickInterval::new(switched_tick_period)?;

    record_tick_contexts(
        |tick_recorder| TickScheduleSwitchingTicker {
            first_tick_stalling_ticker: FirstTickStallingTicker {
                first_tick_release: first_tick_release.clone(),
                first_tick_timing: FirstTickTiming::Immediate,
                missed_tick_behavior: MissedTickBehavior::Burst,
                tick_recorder,
                tick_time_limit: TickTimeLimit::Unlimited,
            },
            switched_missed_tick_behavior: MissedTickBehavior::Skip,
            switched_tick_interval,
            tick_interval: TickInterval::default(),
        },
        |_cancellation_token| async {
            advance(first_tick_stall).await;
            first_tick_release.cancel();
        },
        TICKS_BEFORE_CANCELLATION,
    )
    .await
}

struct TickScheduleSwitchingTicker {
    first_tick_stalling_ticker: FirstTickStallingTicker,
    switched_missed_tick_behavior: MissedTickBehavior,
    switched_tick_interval: TickInterval,
    tick_interval: TickInterval,
}

impl Ticker for TickScheduleSwitchingTicker {
    type Error = TestFailure;

    fn first_tick_timing(&self) -> FirstTickTiming {
        self.first_tick_stalling_ticker.first_tick_timing()
    }

    fn handle_tick(
        &mut self,
        cancellation_token: CancellationToken,
        tick_context: TickContext,
    ) -> impl Future<Output = Result<(), TestFailure>> + Send {
        self.first_tick_stalling_ticker.missed_tick_behavior = self.switched_missed_tick_behavior;
        self.tick_interval = self.switched_tick_interval;
        self.first_tick_stalling_ticker
            .handle_tick(cancellation_token, tick_context)
    }

    fn missed_tick_behavior(&self) -> MissedTickBehavior {
        self.first_tick_stalling_ticker.missed_tick_behavior()
    }

    fn tick_interval(&self) -> TickInterval {
        self.tick_interval
    }

    fn tick_time_limit(&self) -> TickTimeLimit {
        self.first_tick_stalling_ticker.tick_time_limit()
    }
}

#[tokio::test(start_paused = true)]
async fn keeps_tick_schedule_read_at_start() {
    let tick_period = TickInterval::default().period();
    let expected_tick_contexts = ExpectedTickContexts { tick_period };

    assert_eq!(
        record_ticks_after_tick_schedule_switches(tick_period * 7 / 2, tick_period * 2).await,
        expected_tick_contexts
            .third_immediate_tick()
            .and_then(|third_immediate_tick| {
                expected_tick_contexts
                    .fourth_immediate_tick()
                    .map(|fourth_immediate_tick| {
                        vec![
                            ExpectedTickContexts::first_immediate_tick(),
                            expected_tick_contexts.second_immediate_tick(),
                            third_immediate_tick,
                            fourth_immediate_tick,
                        ]
                    })
            })
    );
}
