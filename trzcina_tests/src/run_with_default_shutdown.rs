use tokio_util::sync::CancellationToken;

use trzcina::service_manager::ServiceManager;
use trzcina::service_shutdown_options::ServiceShutdownOptions;
use trzcina::service_shutdown_outcome_collection::ServiceShutdownOutcomeCollection;

pub async fn run_with_default_shutdown(
    service_manager: ServiceManager,
) -> ServiceShutdownOutcomeCollection {
    service_manager
        .start(CancellationToken::new())
        .run_to_completion(ServiceShutdownOptions::default())
        .await
}
