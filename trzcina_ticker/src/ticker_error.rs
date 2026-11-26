use trzcina_tokio_safe_delay::tokio_safe_delay_error::TokioSafeDelayError;

#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum TickerError {
    #[error("tick interval is longer than tokio intervals can schedule")]
    TickIntervalTooLong { source: TokioSafeDelayError },
    #[error("tick interval must be longer than zero")]
    ZeroTickInterval,
}
