use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;
use trzcina::Service;
use trzcina::ServiceManager;
use trzcina::ServiceShutdownOptions;
use trzcina::ServiceShutdownOutcome;

const PANIC_LITERAL: &str = "deliberately panicking with a string literal";

struct LiteralPanickingService;

#[async_trait]
impl Service for LiteralPanickingService {
    async fn run(self: Box<Self>, _cancellation_token: CancellationToken) -> Result<()> {
        panic!("deliberately panicking with a string literal");
    }
}

#[tokio::test]
async fn records_string_literal_panic_payload() {
    let mut manager = ServiceManager::default();
    manager.register_service(LiteralPanickingService);

    let report = timeout(
        Duration::from_secs(5),
        manager
            .start(CancellationToken::new())
            .run_to_completion(ServiceShutdownOptions::default()),
    )
    .await
    .unwrap();

    assert_eq!(report.outcomes().len(), 1);
    match &report.outcomes()[0].outcome {
        ServiceShutdownOutcome::Panicked(panic_message) => {
            assert!(panic_message.contains(PANIC_LITERAL));
        }
        other_outcome => panic!("expected ServiceShutdownOutcome::Panicked, got {other_outcome:?}"),
    }
}
