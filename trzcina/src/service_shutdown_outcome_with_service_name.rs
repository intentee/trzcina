use crate::service_shutdown_error::ServiceShutdownError;
use crate::service_shutdown_failure_source::ServiceShutdownFailureSource;
use crate::service_shutdown_outcome::ServiceShutdownOutcome;

#[derive(Debug)]
pub struct ServiceShutdownOutcomeWithServiceName {
    pub name: &'static str,
    pub outcome: ServiceShutdownOutcome,
}

impl ServiceShutdownFailureSource for ServiceShutdownOutcomeWithServiceName {
    fn into_failure(self) -> Option<ServiceShutdownError> {
        let Self { name, outcome } = self;

        match outcome {
            ServiceShutdownOutcome::Completed => None,
            ServiceShutdownOutcome::Errored(source) => Some(ServiceShutdownError::Errored {
                service_name: name,
                source,
            }),
            ServiceShutdownOutcome::Panicked(panic_message) => {
                Some(ServiceShutdownError::Panicked {
                    service_name: name,
                    panic_message,
                })
            }
            ServiceShutdownOutcome::PanickedWithNonStringPayload => {
                Some(ServiceShutdownError::PanickedWithNonStringPayload { service_name: name })
            }
            ServiceShutdownOutcome::AbortedByShutdownDeadline => {
                Some(ServiceShutdownError::AbortedByShutdownDeadline { service_name: name })
            }
            ServiceShutdownOutcome::CancelledByRuntimeShutdown => {
                Some(ServiceShutdownError::CancelledByRuntimeShutdown { service_name: name })
            }
            ServiceShutdownOutcome::LeakedBeyondShutdownDeadline => {
                Some(ServiceShutdownError::LeakedBeyondShutdownDeadline { service_name: name })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::service_shutdown_error::ServiceShutdownError;
    use crate::service_shutdown_failure_source::ServiceShutdownFailureSource;
    use crate::service_shutdown_outcome::ServiceShutdownOutcome;

    use super::ServiceShutdownOutcomeWithServiceName;

    #[test]
    fn reports_aborted_service_under_its_name() {
        assert!(matches!(
            ServiceShutdownOutcomeWithServiceName {
                name: "aborted_service",
                outcome: ServiceShutdownOutcome::AbortedByShutdownDeadline,
            }
            .into_failure(),
            Some(ServiceShutdownError::AbortedByShutdownDeadline {
                service_name: "aborted_service"
            })
        ));
    }

    #[test]
    fn reports_non_string_panic_payload_as_its_own_failure() {
        assert!(matches!(
            ServiceShutdownOutcomeWithServiceName {
                name: "non_string_payload_panicking_service",
                outcome: ServiceShutdownOutcome::PanickedWithNonStringPayload,
            }
            .into_failure(),
            Some(ServiceShutdownError::PanickedWithNonStringPayload {
                service_name: "non_string_payload_panicking_service"
            })
        ));
    }
}
