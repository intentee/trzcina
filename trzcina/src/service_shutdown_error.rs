use std::error::Error;

#[derive(Debug, thiserror::Error)]
pub enum ServiceShutdownError {
    #[error("service {service_name:?} aborted after shutdown deadline")]
    AbortedByShutdownDeadline { service_name: &'static str },
    #[error("service {service_name:?} was cancelled by its runtime shutting down")]
    CancelledByRuntimeShutdown { service_name: &'static str },
    #[error("service {service_name:?} errored")]
    Errored {
        service_name: &'static str,
        source: Box<dyn Error + Send + Sync>,
    },
    #[error("service {service_name:?} leaked beyond shutdown deadline")]
    LeakedBeyondShutdownDeadline { service_name: &'static str },
    #[error("service {service_name:?} panicked: {panic_message}")]
    Panicked {
        service_name: &'static str,
        panic_message: String,
    },
    #[error("service {service_name:?} panicked with a non-string payload")]
    PanickedWithNonStringPayload { service_name: &'static str },
}
