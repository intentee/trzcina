use anyhow::Error;

#[derive(Debug)]
pub enum ServiceShutdownOutcome {
    Completed,
    Errored(Error),
    Panicked(String),
    AbortedByShutdownDeadline,
    LeakedBeyondShutdownDeadline,
}
