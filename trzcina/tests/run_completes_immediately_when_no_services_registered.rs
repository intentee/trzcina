use std::time::Duration;

use trzcina::ServiceManager;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn completes_immediately_when_no_services_registered() {
    let manager = ServiceManager::default();
    timeout(
        Duration::from_secs(5),
        manager
            .start(CancellationToken::new())
            .run_to_completion(Duration::from_secs(1)),
    )
    .await
    .unwrap()
    .into_result()
    .unwrap();
}
