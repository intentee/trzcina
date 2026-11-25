use tokio::time::Instant;

use crate::tick_context::TickContext;

pub struct TickClock {
    last_tick_at: Instant,
    started_at: Instant,
    ticks_since_start: u64,
}

impl TickClock {
    #[must_use]
    pub const fn new(started_at: Instant) -> Self {
        Self {
            last_tick_at: started_at,
            started_at,
            ticks_since_start: 0,
        }
    }

    pub fn record_tick(&mut self, tick_started_at: Instant) -> TickContext {
        let tick_context = TickContext {
            elapsed_since_start: tick_started_at.duration_since(self.started_at),
            ticks_since_start: self.ticks_since_start,
            since_last_tick: tick_started_at.duration_since(self.last_tick_at),
        };

        self.last_tick_at = tick_started_at;
        self.ticks_since_start = self.ticks_since_start.strict_add(1);

        tick_context
    }
}
