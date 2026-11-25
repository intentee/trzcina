use std::any::type_name;

use log::warn;
use tokio::time::Instant;
use tokio::time::interval_at;
use tokio_util::sync::CancellationToken;

use trzcina::service::Service;
use trzcina_tokio_safe_delay::timeout_within_tokio_safe_delay_cap::timeout_within_tokio_safe_delay_cap;

use crate::first_tick_timing::FirstTickTiming;
use crate::tick_clock::TickClock;
use crate::tick_context::TickContext;
use crate::tick_time_limit::TickTimeLimit;
use crate::ticker::Ticker;

/// Runs a `Ticker` as a `Service`.
///
/// Once its cancellation token is cancelled, it starts no new tick; a tick already running receives the same
/// token, and the service stops once that tick returns or exceeds its time limit.
pub struct TickerService<TTicker> {
    pub ticker: TTicker,
}

impl<TTicker: Ticker> TickerService<TTicker> {
    async fn handle_tick_within_time_limit(
        &mut self,
        cancellation_token: CancellationToken,
        tick_context @ TickContext {
            ticks_since_start, ..
        }: TickContext,
    ) -> Result<(), TTicker::Error> {
        let tick_time_limit = self.ticker.tick_time_limit();
        let tick_handling = self.ticker.handle_tick(cancellation_token, tick_context);

        match tick_time_limit {
            TickTimeLimit::Limited(tick_time_limit) => {
                match timeout_within_tokio_safe_delay_cap(tick_time_limit, tick_handling).await {
                    Ok(handle_tick_result) => handle_tick_result,
                    Err(tick_time_limit_elapsed) => {
                        warn!(
                            "Service {:?} skipped tick {}: {tick_time_limit_elapsed}",
                            self.name(),
                            ticks_since_start
                        );

                        Ok(())
                    }
                }
            }
            TickTimeLimit::Unlimited => tick_handling.await,
        }
    }
}

impl<TTicker: Ticker> Service for TickerService<TTicker> {
    type Error = TTicker::Error;

    fn name(&self) -> &'static str {
        type_name::<TTicker>()
    }

    async fn run(
        mut self: Box<Self>,
        cancellation_token: CancellationToken,
    ) -> Result<(), TTicker::Error> {
        let started_at = Instant::now();
        let missed_tick_behavior = self.ticker.missed_tick_behavior();
        let mut ticker_interval = interval_at(started_at, self.ticker.tick_interval().period());

        match self.ticker.first_tick_timing() {
            FirstTickTiming::AfterInterval => {
                let _tick_due_at_start = cancellation_token
                    .run_until_cancelled(ticker_interval.tick())
                    .await;
            }
            FirstTickTiming::Immediate => {}
        }

        ticker_interval.set_missed_tick_behavior(missed_tick_behavior);

        let mut tick_clock = TickClock::new(started_at);

        while let Some(tick_started_at) = cancellation_token
            .run_until_cancelled(ticker_interval.tick())
            .await
            && !cancellation_token.is_cancelled()
        {
            self.handle_tick_within_time_limit(
                cancellation_token.clone(),
                tick_clock.record_tick(tick_started_at),
            )
            .await?;
        }

        Ok(())
    }
}
