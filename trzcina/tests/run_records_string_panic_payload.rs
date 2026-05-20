use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;
use trzcina::Service;
use trzcina::ServiceManager;
use trzcina::ServiceShutdownOptions;
use trzcina::ServiceShutdownOutcome;

struct StringPanickingService {
    panic_payload: String,
}

#[async_trait]
impl Service for StringPanickingService {
    async fn run(&mut self, _cancellation_token: CancellationToken) -> Result<()> {
        panic!("dynamic message: {}", self.panic_payload);
    }
}

#[tokio::test]
async fn records_string_panic_payload() {
    let mut manager = ServiceManager::default();
    manager.register_service(StringPanickingService {
        panic_payload: "owned-string-panic-payload".to_owned(),
    });

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
            assert!(panic_message.contains("owned-string-panic-payload"));
        }
        other_outcome => panic!("expected ServiceShutdownOutcome::Panicked, got {other_outcome:?}"),
    }
}
