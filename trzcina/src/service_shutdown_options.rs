use std::time::Duration;

/// Deadlines that apply once the cancellation token fires; both default to 10 seconds, and a deadline beyond
/// `TOKIO_SAFE_DELAY_CAP` never elapses.
#[derive(Clone)]
pub struct ServiceShutdownOptions {
    /// How long services have to stop on their own.
    pub cooperative_deadline: Duration,
    /// How long services still running after the cooperative deadline have to stop once aborted.
    pub abort_deadline: Duration,
}

impl Default for ServiceShutdownOptions {
    fn default() -> Self {
        Self {
            cooperative_deadline: Duration::from_secs(10),
            abort_deadline: Duration::from_secs(10),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::ServiceShutdownOptions;

    #[test]
    fn default_uses_ten_second_deadlines_for_both_phases() {
        let ServiceShutdownOptions {
            cooperative_deadline,
            abort_deadline,
        } = ServiceShutdownOptions::default();

        assert_eq!(cooperative_deadline, Duration::from_secs(10));
        assert_eq!(abort_deadline, Duration::from_secs(10));
    }
}
