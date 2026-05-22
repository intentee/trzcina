use std::fmt;
use std::fmt::Write;

use trzcina::ServiceShutdownError;
use trzcina::ServiceShutdownOutcome;
use trzcina::ServiceShutdownOutcomeWithServiceName;

struct AlwaysFailingWriter;

impl fmt::Write for AlwaysFailingWriter {
    fn write_str(&mut self, _written: &str) -> fmt::Result {
        Err(fmt::Error)
    }
}

#[test]
fn display_propagates_writer_error_from_header_line() {
    let shutdown_error = ServiceShutdownError::new(vec![ServiceShutdownOutcomeWithServiceName {
        name: "test_service",
        outcome: ServiceShutdownOutcome::AbortedByShutdownDeadline,
    }]);

    let mut writer = AlwaysFailingWriter;
    let write_result = write!(writer, "{shutdown_error}");

    assert!(write_result.is_err());
}
