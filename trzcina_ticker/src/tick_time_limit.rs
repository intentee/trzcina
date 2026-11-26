use std::time::Duration;

/// How long a tick may run; a tick still running at its limit is dropped and skipped with a warning. Tickers are
/// unlimited by default, and a limit beyond `TOKIO_SAFE_DELAY_CAP` never elapses.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TickTimeLimit {
    Limited(Duration),
    Unlimited,
}
