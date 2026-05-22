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
