use std::time::Duration;

/// The 30 years tokio caps sleeps and interval periods at; `tokio::time::timeout` applies no such cap.
pub const TOKIO_SAFE_DELAY_CAP: Duration = Duration::from_hours(24 * 365 * 30);
