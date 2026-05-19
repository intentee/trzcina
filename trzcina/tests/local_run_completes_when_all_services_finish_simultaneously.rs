use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;
use trzcina::LocalService;
use trzcina::LocalServiceManager;
use trzcina::ServiceShutdownOutcome;

struct InstantOkService;

#[async_trait(?Send)]
impl LocalService for InstantOkService {
    async fn run(&mut self, _cancellation_token: CancellationToken) -> Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn local_completes_when_all_services_finish_simultaneously() {
    let mut manager = LocalServiceManager::default();
    for _ in 0..5 {
        manager.register_service(InstantOkService);
    }

    let report = timeout(
        Duration::from_secs(5),
        manager
            .start_local(CancellationToken::new())
            .run_to_completion(Duration::from_secs(1)),
    )
    .await
    .unwrap();

    assert_eq!(report.outcomes().len(), 5);
    for named_outcome in report.outcomes() {
        assert!(matches!(
            named_outcome.outcome,
            ServiceShutdownOutcome::Completed
        ));
    }
}
