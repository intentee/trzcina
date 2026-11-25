use std::error::Error;

use crate::panic_payload::PanicPayload;

#[derive(Debug)]
pub enum ServiceShutdownOutcome {
    Completed,
    Errored(Box<dyn Error + Send + Sync>),
    Panicked(String),
    PanickedWithNonStringPayload,
    AbortedByShutdownDeadline,
    CancelledByRuntimeShutdown,
    LeakedBeyondShutdownDeadline,
}

impl From<PanicPayload> for ServiceShutdownOutcome {
    fn from(panic_payload: PanicPayload) -> Self {
        panic_payload
            .message()
            .map_or(Self::PanickedWithNonStringPayload, |panic_message| {
                Self::Panicked(panic_message.to_owned())
            })
    }
}

#[cfg(test)]
mod tests {
    use crate::panic_payload::PanicPayload;

    use super::ServiceShutdownOutcome;

    #[test]
    fn keeps_owned_string_payload() {
        let panic_payload = PanicPayload {
            payload: Box::new(String::from("owned panic message")),
        };

        assert!(matches!(
            ServiceShutdownOutcome::from(panic_payload),
            ServiceShutdownOutcome::Panicked(panic_message) if panic_message == "owned panic message"
        ));
    }

    #[test]
    fn classifies_non_string_payload_as_non_string_panic() {
        let panic_payload = PanicPayload {
            payload: Box::new(()),
        };

        assert!(matches!(
            ServiceShutdownOutcome::from(panic_payload),
            ServiceShutdownOutcome::PanickedWithNonStringPayload
        ));
    }
}
