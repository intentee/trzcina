use std::time::Duration;

use tokio_util::sync::CancellationToken;
use trzcina::LocalServiceManager;
use trzcina::Manager;
use trzcina::RunToCompletionOptions;
use trzcina::RunningCollection;
use trzcina::ServiceManager;

async fn drive<TManager: Manager>(manager: TManager) {
    let running = Manager::start(manager, CancellationToken::new());
    let _ = RunningCollection::run_to_completion(
        running,
        RunToCompletionOptions {
            shutdown_deadline: Duration::from_secs(1),
        },
    )
    .await;
}

#[tokio::test]
async fn sendable_service_manager_implements_manager() {
    drive(ServiceManager::default()).await;
}

#[tokio::test]
async fn local_service_manager_implements_manager() {
    drive(LocalServiceManager::default()).await;
}
