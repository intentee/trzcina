use std::time::Duration;

use tokio::sync::oneshot;

use trzcina::service_manager::ServiceManager;
use trzcina::service_shutdown_options::ServiceShutdownOptions;
use trzcina::service_shutdown_outcome::ServiceShutdownOutcome;
use trzcina::service_shutdown_outcome_collection::ServiceShutdownOutcomeCollection;
use trzcina::service_shutdown_outcome_with_service_name::ServiceShutdownOutcomeWithServiceName;
use trzcina_tests::observable_service::ObservableService;
use trzcina_tests::observed_run::ObservedRun;

#[tokio::test(start_paused = true)]
async fn reports_finished_service_as_completed_with_zero_deadlines() {
    let (observation_sender, observation_receiver) = oneshot::channel::<()>();
    let mut manager = ServiceManager::default();

    manager.register_service(ObservableService {
        observation_sender: observation_sender.into(),
    });

    let observed_run = ObservedRun::shutdown_after_observation(
        manager,
        ServiceShutdownOptions {
            abort_deadline: Duration::ZERO,
            cooperative_deadline: Duration::ZERO,
        },
        observation_receiver,
    )
    .await;

    assert!(matches!(
        observed_run
            .into_run_output()
            .as_ref()
            .map(ServiceShutdownOutcomeCollection::outcomes),
        Ok([ServiceShutdownOutcomeWithServiceName {
            outcome: ServiceShutdownOutcome::Completed,
            ..
        }])
    ));
}
