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

#[cfg(test)]
mod tests {
    use std::panic::catch_unwind;

    use tokio::time::Instant;

    use super::TickClock;

    #[test]
    fn crashes_instead_of_wrapping_when_tick_count_overflows() {
        let started_at = Instant::now();
        let tick_clock = TickClock {
            last_tick_at: started_at,
            started_at,
            ticks_since_start: u64::MAX,
        };

        assert!(
            catch_unwind(move || {
                let mut overflowing_tick_clock = tick_clock;

                overflowing_tick_clock.record_tick(started_at)
            })
            .is_err()
        );
    }
}
