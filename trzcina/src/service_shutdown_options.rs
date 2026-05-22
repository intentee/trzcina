use std::time::Duration;

#[derive(Clone)]
pub struct ServiceShutdownOptions {
    pub cooperative_deadline: Duration,
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

    #[test]
    fn can_be_cloned() {
        let original = ServiceShutdownOptions {
            cooperative_deadline: Duration::from_millis(7),
            abort_deadline: Duration::from_millis(11),
        };
        let cloned = original.clone();

        assert_eq!(original.cooperative_deadline, cloned.cooperative_deadline);
        assert_eq!(original.abort_deadline, cloned.abort_deadline);
    }
}
