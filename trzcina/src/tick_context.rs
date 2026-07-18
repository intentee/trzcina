use std::time::Duration;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TickContext {
    pub elapsed_since_start: Duration,
    pub ticks_since_start: u64,
    pub since_last_tick: Duration,
}
