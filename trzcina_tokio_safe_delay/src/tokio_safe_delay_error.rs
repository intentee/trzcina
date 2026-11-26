use std::time::Duration;

#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum TokioSafeDelayError {
    #[error(
        "duration of {duration:?} is longer than the longest delay of {longest_duration:?} tokio timers schedule"
    )]
    DurationBeyondCap {
        duration: Duration,
        longest_duration: Duration,
    },
}
