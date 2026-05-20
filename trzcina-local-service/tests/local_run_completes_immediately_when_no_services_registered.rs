use std::time::Duration;

use tokio::time::timeout;
use tokio_util::sync::CancellationToken;
use trzcina_local_service::LocalServiceManager;
use trzcina_service::Manager;
use trzcina_service::RunToCompletionOptions;
use trzcina_service::RunningCollection;

#[tokio::test]
async fn local_completes_immediately_when_no_services_registered() {
    let manager = LocalServiceManager::default();
    timeout(
        Duration::from_secs(5),
        manager
            .start(CancellationToken::new())
            .run_to_completion(RunToCompletionOptions {
                shutdown_deadline: Duration::from_secs(1),
            }),
    )
    .await
    .unwrap()
    .into_result()
    .unwrap();
}
