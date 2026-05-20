use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;
use trzcina_local_service::LocalService;
use trzcina_local_service::LocalServiceManager;
use trzcina_service::Manager;
use trzcina_service::RunToCompletionOptions;
use trzcina_service::RunningCollection;
use trzcina_service::ServiceShutdownOutcome;

const PANIC_LITERAL: &str = "deliberately panicking with a string literal";

struct LiteralPanickingService;

#[async_trait(?Send)]
impl LocalService for LiteralPanickingService {
    async fn run(&mut self, _cancellation_token: CancellationToken) -> Result<()> {
        panic!("deliberately panicking with a string literal");
    }
}

#[tokio::test]
async fn local_records_string_literal_panic_payload() {
    let mut manager = LocalServiceManager::default();
    manager.register_service(LiteralPanickingService);

    let report = timeout(
        Duration::from_secs(5),
        manager
            .start(CancellationToken::new())
            .run_to_completion(RunToCompletionOptions {
                shutdown_deadline: Duration::from_secs(1),
            }),
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
