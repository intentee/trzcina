use std::fmt;
use std::fmt::Write;

use trzcina::ServiceShutdownError;
use trzcina::ServiceShutdownOutcome;
use trzcina::ServiceShutdownOutcomeWithServiceName;

struct WriterThatFailsAfterFirstLine {
    saw_newline: bool,
}

impl fmt::Write for WriterThatFailsAfterFirstLine {
    fn write_str(&mut self, written: &str) -> fmt::Result {
        if self.saw_newline {
            return Err(fmt::Error);
        }

        if written.contains('\n') {
            self.saw_newline = true;
        }

        Ok(())
    }
}

#[test]
fn display_propagates_writer_error_from_outcome_line() {
    let shutdown_error = ServiceShutdownError::new(vec![ServiceShutdownOutcomeWithServiceName {
        name: "test_service",
        outcome: ServiceShutdownOutcome::AbortedByShutdownDeadline,
    }]);

    let mut writer = WriterThatFailsAfterFirstLine { saw_newline: false };
    let write_result = write!(writer, "{shutdown_error}");

    assert!(write_result.is_err());
}
