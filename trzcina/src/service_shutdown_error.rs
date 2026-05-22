use std::error::Error;
use std::fmt;

use crate::service_shutdown_outcome::ServiceShutdownOutcome;
use crate::service_shutdown_outcome_with_service_name::ServiceShutdownOutcomeWithServiceName;

#[derive(Debug)]
pub struct ServiceShutdownError {
    failed_outcomes: Vec<ServiceShutdownOutcomeWithServiceName>,
}

impl ServiceShutdownError {
    #[must_use]
    pub fn new(failed_outcomes: Vec<ServiceShutdownOutcomeWithServiceName>) -> Self {
        Self { failed_outcomes }
    }

    #[must_use]
    pub fn failed_outcomes(&self) -> &[ServiceShutdownOutcomeWithServiceName] {
        &self.failed_outcomes
    }
}

impl fmt::Display for ServiceShutdownError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "service shutdown failed:")?;

        for ServiceShutdownOutcomeWithServiceName { name, outcome } in &self.failed_outcomes {
            match outcome {
                ServiceShutdownOutcome::Completed => Ok(()),
                ServiceShutdownOutcome::Errored(service_error) => {
                    writeln!(f, "  service {name:?} errored: {service_error:#}")
                }
                ServiceShutdownOutcome::Panicked(panic_message) => {
                    writeln!(f, "  service {name:?} panicked: {panic_message}")
                }
                ServiceShutdownOutcome::AbortedByShutdownDeadline => {
                    writeln!(f, "  service {name:?} aborted after shutdown deadline")
                }
                ServiceShutdownOutcome::LeakedBeyondShutdownDeadline => {
                    writeln!(f, "  service {name:?} leaked beyond shutdown deadline")
                }
            }?;
        }

        Ok(())
    }
}

impl Error for ServiceShutdownError {}

#[cfg(test)]
mod tests {
    use std::fmt;
    use std::fmt::Write;

    use anyhow::anyhow;

    use super::ServiceShutdownError;
    use super::ServiceShutdownOutcome;
    use super::ServiceShutdownOutcomeWithServiceName;

    struct AlwaysFailingWriter;

    impl fmt::Write for AlwaysFailingWriter {
        fn write_str(&mut self, _written: &str) -> fmt::Result {
            Err(fmt::Error)
        }
    }

    struct WriterThatFailsOnSecondWrite {
        has_been_called: bool,
    }

    impl fmt::Write for WriterThatFailsOnSecondWrite {
        fn write_str(&mut self, _written: &str) -> fmt::Result {
            if self.has_been_called {
                return Err(fmt::Error);
            }

            self.has_been_called = true;

            Ok(())
        }
    }

    #[test]
    fn display_formats_all_failure_variants() {
        let outcomes = vec![
            ServiceShutdownOutcomeWithServiceName {
                name: "completed_service",
                outcome: ServiceShutdownOutcome::Completed,
            },
            ServiceShutdownOutcomeWithServiceName {
                name: "errored_service",
                outcome: ServiceShutdownOutcome::Errored(anyhow!("service failed")),
            },
            ServiceShutdownOutcomeWithServiceName {
                name: "panicked_service",
                outcome: ServiceShutdownOutcome::Panicked("service panicked".to_owned()),
            },
            ServiceShutdownOutcomeWithServiceName {
                name: "aborted_service",
                outcome: ServiceShutdownOutcome::AbortedByShutdownDeadline,
            },
            ServiceShutdownOutcomeWithServiceName {
                name: "leaked_service",
                outcome: ServiceShutdownOutcome::LeakedBeyondShutdownDeadline,
            },
        ];

        let error = ServiceShutdownError::new(outcomes);
        let formatted = format!("{error}");

        assert!(!formatted.is_empty());
        assert_eq!(error.failed_outcomes().len(), 5);
    }

    #[test]
    fn display_propagates_writer_error_from_header_line() {
        let shutdown_error =
            ServiceShutdownError::new(vec![ServiceShutdownOutcomeWithServiceName {
                name: "test_service",
                outcome: ServiceShutdownOutcome::AbortedByShutdownDeadline,
            }]);

        let mut writer = AlwaysFailingWriter;
        let write_result = write!(writer, "{shutdown_error}");

        assert!(write_result.is_err());
    }

    #[test]
    fn display_propagates_writer_error_from_outcome_line() {
        let shutdown_error =
            ServiceShutdownError::new(vec![ServiceShutdownOutcomeWithServiceName {
                name: "test_service",
                outcome: ServiceShutdownOutcome::AbortedByShutdownDeadline,
            }]);

        let mut writer = WriterThatFailsOnSecondWrite {
            has_been_called: false,
        };
        let write_result = write!(writer, "{shutdown_error}");

        assert!(write_result.is_err());
    }
}
