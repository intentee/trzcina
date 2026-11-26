use tokio::time::advance;
use tokio_util::sync::CancellationToken;

use trzcina::service::Service;
use trzcina_tests::expected_tick_contexts::ExpectedTickContexts;
use trzcina_tests::failing_ticker::FailingTicker;
use trzcina_tests::recording_ticker::RecordingTicker;
use trzcina_tests::test_failure::TestFailure;
use trzcina_ticker::first_tick_timing::FirstTickTiming;
use trzcina_ticker::tick_context::TickContext;
use trzcina_ticker::tick_interval::TickInterval;
use trzcina_ticker::tick_time_limit::TickTimeLimit;
use trzcina_ticker::ticker_service::TickerService;

const TICKS_BEFORE_CANCELLATION: usize = 0;

async fn record_ticks_due_after_cancellation(
    first_tick_timing: FirstTickTiming,
    tick_time_limit: TickTimeLimit,
) -> Result<Vec<TickContext>, TestFailure> {
    let tick_interval = TickInterval::default();

    RecordingTicker::record(
        first_tick_timing,
        tick_interval,
        tick_time_limit,
        |cancellation_token| async move {
            cancellation_token.cancel();
            advance(tick_interval.period()).await;
        },
        TICKS_BEFORE_CANCELLATION,
    )
    .await
}

#[tokio::test(start_paused = true)]
async fn handles_no_tick_once_cancelled() {
    let cancelled_before_start = CancellationToken::new();

    cancelled_before_start.cancel();

    assert_eq!(
        Box::new(TickerService {
            ticker: FailingTicker {
                failing_tick: 0,
                first_tick_timing: FirstTickTiming::Immediate,
                tick_time_limit: TickTimeLimit::Unlimited,
            },
        })
        .run(cancelled_before_start)
        .await,
        Ok(())
    );

    for tick_time_limit in [
        TickTimeLimit::Unlimited,
        TickTimeLimit::Limited(TickInterval::default().period() * 2),
    ] {
        assert_eq!(
            record_ticks_due_after_cancellation(FirstTickTiming::Immediate, tick_time_limit).await,
            Ok(vec![ExpectedTickContexts::first_immediate_tick()])
        );
    }
    assert_eq!(
        record_ticks_due_after_cancellation(
            FirstTickTiming::AfterInterval,
            TickTimeLimit::Unlimited
        )
        .await,
        Ok(vec![])
    );
}
