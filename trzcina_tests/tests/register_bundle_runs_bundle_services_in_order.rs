use std::any::type_name;

use tokio::sync::oneshot;

use trzcina::registered_service::RegisteredService;
use trzcina::service_manager::ServiceManager;
use trzcina::service_shutdown_outcome::ServiceShutdownOutcome;
use trzcina::service_shutdown_outcome_with_service_name::ServiceShutdownOutcomeWithServiceName;
use trzcina_tests::observable_service::ObservableService;
use trzcina_tests::prebuilt_services_bundle::PrebuiltServicesBundle;
use trzcina_tests::run_with_default_shutdown::run_with_default_shutdown;
use trzcina_tests::waiting_observer_service::WaitingObserverService;

#[tokio::test]
async fn runs_bundle_services_in_order() {
    let (first_observation_sender, _first_observation_receiver) = oneshot::channel::<()>();
    let (second_observation_sender, _second_observation_receiver) = oneshot::channel::<()>();
    let mut manager = ServiceManager::default();

    let Ok(()) = manager
        .register_bundle(PrebuiltServicesBundle {
            registered_services: vec![
                RegisteredService::new(ObservableService {
                    observation_sender: first_observation_sender.into(),
                }),
                RegisteredService::new(WaitingObserverService {
                    observation_sender: second_observation_sender.into(),
                }),
            ],
        })
        .await;

    let report = run_with_default_shutdown(manager).await;

    assert!(matches!(
        report.outcomes(),
        [
            ServiceShutdownOutcomeWithServiceName {
                name: first_name,
                outcome: ServiceShutdownOutcome::Completed,
            },
            ServiceShutdownOutcomeWithServiceName {
                name: second_name,
                outcome: ServiceShutdownOutcome::Completed,
            },
        ] if *first_name == type_name::<ObservableService>()
            && *second_name == type_name::<WaitingObserverService>()
    ));
}
