use std::time::Duration;

pub struct RunToCompletionOptions {
    pub shutdown_deadline: Duration,
}

impl Default for RunToCompletionOptions {
    fn default() -> Self {
        Self {
            shutdown_deadline: Duration::from_secs(10),
        }
    }
}
