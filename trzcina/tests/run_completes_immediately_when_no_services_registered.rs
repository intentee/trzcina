use std::time::Duration;

use tokio::time::timeout;
use tokio_util::sync::CancellationToken;
use trzcina::ServiceManager;
use trzcina::ServiceShutdownOptions;

#[tokio::test]
async fn completes_immediately_when_no_services_registered() {
    let manager = ServiceManager::default();
    timeout(
        Duration::from_secs(5),
        manager
            .start(CancellationToken::new())
            .run_to_completion(ServiceShutdownOptions::default()),
    )
    .await
    .unwrap()
    .into_result()
    .unwrap();
}
