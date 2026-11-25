use tokio::sync::mpsc::UnboundedSender;

use trzcina_ticker::tick_context::TickContext;

use crate::test_failure::TestFailure;

pub struct TickRecorder {
    tick_context_sender: UnboundedSender<TickContext>,
}

impl TickRecorder {
    /// # Errors
    ///
    /// Returns `TestFailure::TickContextNotDelivered` when the recording side was dropped.
    pub fn record(&self, tick_context: TickContext) -> Result<(), TestFailure> {
        Ok(self.tick_context_sender.send(tick_context)?)
    }
}

impl From<UnboundedSender<TickContext>> for TickRecorder {
    fn from(tick_context_sender: UnboundedSender<TickContext>) -> Self {
        Self {
            tick_context_sender,
        }
    }
}
