use std::panic::panic_any;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use trzcina::Service;
use trzcina::ServiceManager;
use trzcina::ServiceShutdownOutcome;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;

struct NonStringPanickingService;

#[async_trait]
impl Service for NonStringPanickingService {
    async fn run(&mut self, _cancellation_token: CancellationToken) -> Result<()> {
        panic_any(42_u32);
    }
}

#[tokio::test]
async fn records_non_string_panic_payload_as_generic_message() {
    let mut manager = ServiceManager::default();
    manager.register_service(NonStringPanickingService);

    let report = timeout(
        Duration::from_secs(5),
        manager
            .start(CancellationToken::new())
            .run_to_completion(Duration::from_secs(1)),
    )
    .await
    .unwrap();

    assert_eq!(report.outcomes().len(), 1);
    assert!(matches!(
        report.outcomes()[0].outcome,
        ServiceShutdownOutcome::Panicked(_)
    ));
}
