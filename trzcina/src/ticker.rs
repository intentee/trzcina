use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use tokio::time::Instant;
use tokio::time::MissedTickBehavior;
use tokio::time::interval_at;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;

use crate::first_tick_timing::FirstTickTiming;
use crate::service::Service;
use crate::tick_context::TickContext;

#[async_trait]
pub trait Ticker: Send + 'static {
    fn tick_interval(&self) -> Duration {
        Duration::from_secs(1)
    }

    fn first_tick_timing(&self) -> FirstTickTiming {
        FirstTickTiming::Immediate
    }

    fn missed_tick_behavior(&self) -> MissedTickBehavior {
        MissedTickBehavior::Delay
    }

    fn tick_time_limit(&self) -> Option<Duration> {
        None
    }

    async fn handle_tick(
        &mut self,
        cancellation_token: CancellationToken,
        tick_context: TickContext,
    ) -> Result<()>;
}

#[async_trait]
impl<TTicker: Ticker> Service for TTicker {
    async fn run(self: Box<Self>, cancellation_token: CancellationToken) -> Result<()> {
        let mut ticker = self;

        let tick_interval = ticker.tick_interval();
        let started_at = Instant::now();
        let first_tick_at = match ticker.first_tick_timing() {
            FirstTickTiming::Immediate => started_at,
            FirstTickTiming::AfterInterval => started_at + tick_interval,
        };
        let mut ticker_interval = interval_at(first_tick_at, tick_interval);
        ticker_interval.set_missed_tick_behavior(ticker.missed_tick_behavior());

        let mut last_tick_at = started_at;
        let mut ticks_since_start: u64 = 0;

        loop {
            let Some(tick_started_at) = cancellation_token
                .run_until_cancelled(ticker_interval.tick())
                .await
            else {
                return Ok(());
            };

            let tick_context = TickContext {
                elapsed_since_start: tick_started_at.duration_since(started_at),
                ticks_since_start,
                since_last_tick: tick_started_at.duration_since(last_tick_at),
            };
            last_tick_at = tick_started_at;
            ticks_since_start += 1;

            match ticker.tick_time_limit() {
                None => {
                    ticker
                        .handle_tick(cancellation_token.clone(), tick_context)
                        .await?;
                }
                Some(tick_time_limit) => {
                    match timeout(
                        tick_time_limit,
                        ticker.handle_tick(cancellation_token.clone(), tick_context),
                    )
                    .await
                    {
                        Ok(handle_tick_result) => handle_tick_result?,
                        Err(_elapsed) => continue,
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use anyhow::Result;
    use anyhow::anyhow;
    use async_trait::async_trait;
    use tokio::sync::mpsc;
    use tokio::time::MissedTickBehavior;
    use tokio::time::sleep;
    use tokio_util::sync::CancellationToken;

    use super::FirstTickTiming;
    use super::Service;
    use super::TickContext;
    use super::Ticker;

    struct RecordingTicker {
        first_tick_timing: FirstTickTiming,
        contexts_sender: mpsc::UnboundedSender<TickContext>,
    }

    #[async_trait]
    impl Ticker for RecordingTicker {
        fn first_tick_timing(&self) -> FirstTickTiming {
            self.first_tick_timing
        }

        async fn handle_tick(
            &mut self,
            _cancellation_token: CancellationToken,
            tick_context: TickContext,
        ) -> Result<()> {
            self.contexts_sender.send(tick_context).unwrap();

            Ok(())
        }
    }

    struct FailingTicker {
        tick_interval: Duration,
        time_limit: Option<Duration>,
    }

    #[async_trait]
    impl Ticker for FailingTicker {
        fn tick_interval(&self) -> Duration {
            self.tick_interval
        }

        fn tick_time_limit(&self) -> Option<Duration> {
            self.time_limit
        }

        async fn handle_tick(
            &mut self,
            _cancellation_token: CancellationToken,
            _tick_context: TickContext,
        ) -> Result<()> {
            Err(anyhow!("handle_tick failed"))
        }
    }

    struct SkippingTicker {
        tick_interval: Duration,
        time_limit: Duration,
        contexts_sender: mpsc::UnboundedSender<TickContext>,
    }

    #[async_trait]
    impl Ticker for SkippingTicker {
        fn tick_interval(&self) -> Duration {
            self.tick_interval
        }

        fn tick_time_limit(&self) -> Option<Duration> {
            Some(self.time_limit)
        }

        async fn handle_tick(
            &mut self,
            _cancellation_token: CancellationToken,
            tick_context: TickContext,
        ) -> Result<()> {
            if tick_context.ticks_since_start == 0 {
                sleep(self.time_limit * 2).await;
            } else {
                self.contexts_sender.send(tick_context).unwrap();
            }

            Ok(())
        }
    }

    #[tokio::test(start_paused = true)]
    async fn fires_ticks_with_monotonic_context_until_cancelled() {
        let tick_interval = Duration::from_secs(1);
        let (contexts_sender, mut contexts_receiver) = mpsc::unbounded_channel();
        let cancellation_token = CancellationToken::new();
        let run_token = cancellation_token.clone();

        let run_handle = tokio::spawn(async move {
            Box::new(RecordingTicker {
                first_tick_timing: FirstTickTiming::Immediate,
                contexts_sender,
            })
            .run(run_token)
            .await
        });

        let first_tick_context = contexts_receiver.recv().await.unwrap();
        let second_tick_context = contexts_receiver.recv().await.unwrap();
        let third_tick_context = contexts_receiver.recv().await.unwrap();

        cancellation_token.cancel();
        run_handle.await.unwrap().unwrap();

        assert_eq!(
            first_tick_context,
            TickContext {
                elapsed_since_start: Duration::ZERO,
                ticks_since_start: 0,
                since_last_tick: Duration::ZERO,
            }
        );
        assert_eq!(
            second_tick_context,
            TickContext {
                elapsed_since_start: tick_interval,
                ticks_since_start: 1,
                since_last_tick: tick_interval,
            }
        );
        assert_eq!(
            third_tick_context,
            TickContext {
                elapsed_since_start: tick_interval * 2,
                ticks_since_start: 2,
                since_last_tick: tick_interval,
            }
        );
    }

    #[tokio::test(start_paused = true)]
    async fn delays_first_tick_by_one_interval_when_configured() {
        let tick_interval = Duration::from_secs(1);
        let (contexts_sender, mut contexts_receiver) = mpsc::unbounded_channel();
        let cancellation_token = CancellationToken::new();
        let run_token = cancellation_token.clone();

        let run_handle = tokio::spawn(async move {
            Box::new(RecordingTicker {
                first_tick_timing: FirstTickTiming::AfterInterval,
                contexts_sender,
            })
            .run(run_token)
            .await
        });

        let first_tick_context = contexts_receiver.recv().await.unwrap();

        cancellation_token.cancel();
        run_handle.await.unwrap().unwrap();

        assert_eq!(
            first_tick_context,
            TickContext {
                elapsed_since_start: tick_interval,
                ticks_since_start: 0,
                since_last_tick: tick_interval,
            }
        );
    }

    #[tokio::test(start_paused = true)]
    async fn propagates_handle_tick_error() {
        let run_result = Box::new(FailingTicker {
            tick_interval: Duration::from_millis(10),
            time_limit: None,
        })
        .run(CancellationToken::new())
        .await;

        assert!(run_result.is_err());
    }

    #[tokio::test(start_paused = true)]
    async fn propagates_handle_tick_error_within_time_limit() {
        let run_result = Box::new(FailingTicker {
            tick_interval: Duration::from_millis(10),
            time_limit: Some(Duration::from_millis(5)),
        })
        .run(CancellationToken::new())
        .await;

        assert!(run_result.is_err());
    }

    #[tokio::test(start_paused = true)]
    async fn skips_tick_that_exceeds_time_limit_and_continues() {
        let (contexts_sender, mut contexts_receiver) = mpsc::unbounded_channel();
        let cancellation_token = CancellationToken::new();
        let run_token = cancellation_token.clone();

        let run_handle = tokio::spawn(async move {
            Box::new(SkippingTicker {
                tick_interval: Duration::from_millis(10),
                time_limit: Duration::from_millis(5),
                contexts_sender,
            })
            .run(run_token)
            .await
        });

        let first_recorded_tick = contexts_receiver.recv().await.unwrap();

        cancellation_token.cancel();
        run_handle.await.unwrap().unwrap();

        assert_eq!(first_recorded_tick.ticks_since_start, 1);
    }

    #[test]
    fn defaults_to_delay_missed_tick_behavior() {
        let (contexts_sender, _contexts_receiver) = mpsc::unbounded_channel();
        let ticker = RecordingTicker {
            first_tick_timing: FirstTickTiming::Immediate,
            contexts_sender,
        };

        assert_eq!(ticker.missed_tick_behavior(), MissedTickBehavior::Delay);
    }
}
