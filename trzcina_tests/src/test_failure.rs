use std::error::Error;
use std::time::Duration;

use tokio::sync::mpsc::error::SendError;

use trzcina_ticker::tick_context::TickContext;
use trzcina_ticker::ticker_error::TickerError;

#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum TestFailure {
    #[error("service failed deliberately")]
    DeliberateServiceFailure,
    #[error("tick failed deliberately")]
    DeliberateTickFailure,
    #[error("observation was not delivered")]
    ObservationNotDelivered,
    #[error("tick interval was rejected")]
    TickIntervalRejected {
        #[from]
        source: TickerError,
    },
    #[error("tick context was not delivered")]
    TickContextNotDelivered {
        #[from]
        source: SendError<TickContext>,
    },
    #[error("{periods} tick periods of {tick_period:?} cannot be represented")]
    TickPeriodsOverflow { periods: u32, tick_period: Duration },
}

impl TestFailure {
    #[must_use]
    pub fn is_erased_as(&self, erased_error: &(dyn Error + Send + Sync + 'static)) -> bool {
        erased_error.downcast_ref::<Self>() == Some(self)
    }
}
