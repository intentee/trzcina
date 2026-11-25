use crate::service_shutdown_error::ServiceShutdownError;

pub trait ServiceShutdownFailureSource {
    #[must_use]
    fn into_failure(self) -> Option<ServiceShutdownError>;
}
