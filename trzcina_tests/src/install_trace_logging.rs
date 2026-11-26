use std::sync::OnceLock;

use env_logger::Builder;
use log::LevelFilter;
use log::SetLoggerError;

static TRACE_LOGGING_INSTALLATION: OnceLock<Result<(), SetLoggerError>> = OnceLock::new();

/// # Errors
///
/// Returns the `SetLoggerError` when another logger was installed first.
pub fn install_trace_logging() -> Result<(), &'static SetLoggerError> {
    TRACE_LOGGING_INSTALLATION
        .get_or_init(|| {
            Builder::new()
                .filter_level(LevelFilter::Trace)
                .is_test(true)
                .try_init()
        })
        .as_ref()
        .copied()
}
