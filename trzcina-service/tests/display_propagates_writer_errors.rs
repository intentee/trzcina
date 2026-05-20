use std::fmt;
use std::fmt::Write;

use anyhow::anyhow;
use trzcina_service::ServiceShutdownError;
use trzcina_service::ServiceShutdownOutcome;
use trzcina_service::ServiceShutdownOutcomeWithServiceName;

struct WriterFailingAfter {
    remaining_successful_calls: usize,
}

impl WriterFailingAfter {
    fn new(successful_calls_before_failure: usize) -> Self {
        Self {
            remaining_successful_calls: successful_calls_before_failure,
        }
    }
}

impl Write for WriterFailingAfter {
    fn write_str(&mut self, _payload: &str) -> fmt::Result {
        if self.remaining_successful_calls == 0 {
            return Err(fmt::Error);
        }
        self.remaining_successful_calls -= 1;
        Ok(())
    }
}

fn build_error_with_one_errored_service() -> ServiceShutdownError {
    ServiceShutdownError::new(vec![ServiceShutdownOutcomeWithServiceName {
        name: "errored_service",
        outcome: ServiceShutdownOutcome::Errored(anyhow!("service failed")),
    }])
}

#[test]
fn display_propagates_header_write_failure() {
    let error = build_error_with_one_errored_service();
    let mut writer = WriterFailingAfter::new(0);
    let format_result = write!(writer, "{error}");
    assert!(format_result.is_err());
}

#[test]
fn display_propagates_body_write_failure() {
    let error = build_error_with_one_errored_service();
    let mut writer = WriterFailingAfter::new(1);
    let format_result = write!(writer, "{error}");
    assert!(format_result.is_err());
}
