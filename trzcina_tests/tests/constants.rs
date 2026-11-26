use std::time::Duration;

pub const SMALLEST_DURATION_STEP: Duration = Duration::from_nanos(1);
pub const TOKIO_TIMER_GRANULARITY: Duration = Duration::from_millis(1);
