use std::io;

use tokio::runtime::Builder;

/// # Errors
///
/// Returns the `io::Error` of building the paused runtime.
pub fn block_on_paused_runtime<TRun: Future>(run: TRun) -> io::Result<TRun::Output> {
    Ok(Builder::new_current_thread()
        .enable_all()
        .start_paused(true)
        .build()?
        .block_on(run))
}
