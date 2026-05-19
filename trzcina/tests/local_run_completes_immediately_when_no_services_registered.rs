use std::time::Duration;

use tokio::time::timeout;
use tokio_util::sync::CancellationToken;
use trzcina::LocalServiceManager;

#[tokio::test]
async fn local_completes_immediately_when_no_services_registered() {
    let manager = LocalServiceManager::default();
    timeout(
        Duration::from_secs(5),
        manager
            .start_local(CancellationToken::new())
            .run_to_completion(Duration::from_secs(1)),
    )
    .await
    .unwrap()
    .into_result()
    .unwrap();
}
