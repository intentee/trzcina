use std::time::Duration;

#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum TickerError {
    #[error(
        "tick interval of {period:?} is longer than the longest supported period of {longest_period:?}"
    )]
    TickIntervalTooLong {
        longest_period: Duration,
        period: Duration,
    },
    #[error("tick interval must be longer than zero")]
    ZeroTickInterval,
}
