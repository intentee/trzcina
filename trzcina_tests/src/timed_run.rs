use std::time::Duration;

use tokio::time::Instant;

pub struct TimedRun<TOutput> {
    pub elapsed: Duration,
    pub output: TOutput,
}

impl<TOutput> TimedRun<TOutput> {
    pub async fn measure<TRun: Future<Output = TOutput>>(run: TRun) -> Self {
        let run_started_at = Instant::now();
        let output = run.await;

        Self {
            elapsed: run_started_at.elapsed(),
            output,
        }
    }
}
