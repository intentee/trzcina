use std::time::Duration;

use trzcina_ticker::tick_context::TickContext;

use crate::test_failure::TestFailure;

pub struct ExpectedTickContexts {
    pub tick_period: Duration,
}

impl ExpectedTickContexts {
    #[must_use]
    pub const fn first_delayed_tick(&self) -> TickContext {
        TickContext {
            elapsed_since_start: self.tick_period,
            ticks_since_start: 0,
            since_last_tick: self.tick_period,
        }
    }

    #[must_use]
    pub const fn first_immediate_tick() -> TickContext {
        TickContext {
            elapsed_since_start: Duration::ZERO,
            ticks_since_start: 0,
            since_last_tick: Duration::ZERO,
        }
    }

    /// # Errors
    ///
    /// Returns `TestFailure::TickPeriodsOverflow` when three tick periods cannot be represented.
    pub fn fourth_immediate_tick(&self) -> Result<TickContext, TestFailure> {
        self.tick_periods(3).map(|three_tick_periods| TickContext {
            elapsed_since_start: three_tick_periods,
            ticks_since_start: 3,
            since_last_tick: self.tick_period,
        })
    }

    /// # Errors
    ///
    /// Returns `TestFailure::TickPeriodsOverflow` when two tick periods cannot be represented.
    pub fn second_delayed_tick(&self) -> Result<TickContext, TestFailure> {
        self.tick_periods(2).map(|two_tick_periods| TickContext {
            elapsed_since_start: two_tick_periods,
            ticks_since_start: 1,
            since_last_tick: self.tick_period,
        })
    }

    #[must_use]
    pub const fn second_immediate_tick(&self) -> TickContext {
        TickContext {
            elapsed_since_start: self.tick_period,
            ticks_since_start: 1,
            since_last_tick: self.tick_period,
        }
    }

    /// # Errors
    ///
    /// Returns `TestFailure::TickPeriodsOverflow` when two tick periods cannot be represented.
    pub fn third_immediate_tick(&self) -> Result<TickContext, TestFailure> {
        self.tick_periods(2).map(|two_tick_periods| TickContext {
            elapsed_since_start: two_tick_periods,
            ticks_since_start: 2,
            since_last_tick: self.tick_period,
        })
    }

    fn tick_periods(&self, periods: u32) -> Result<Duration, TestFailure> {
        self.tick_period
            .checked_mul(periods)
            .ok_or(TestFailure::TickPeriodsOverflow {
                periods,
                tick_period: self.tick_period,
            })
    }
}
