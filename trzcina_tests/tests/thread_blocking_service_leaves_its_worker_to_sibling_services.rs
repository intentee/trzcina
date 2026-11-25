use tokio::sync::oneshot;

use trzcina::service_manager::ServiceManager;
use trzcina::service_shutdown_outcome_collection::ServiceShutdownOutcomeCollection;
use trzcina_tests::observable_service::ObservableService;
use trzcina_tests::observed_run::ObservedRun;
use trzcina_tests::run_with_default_shutdown::run_with_default_shutdown;
use trzcina_tests::thread_blocking_control::ThreadBlockingControl;

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn leaves_its_worker_to_sibling_services() {
    let mut manager = ServiceManager::default();
    let ThreadBlockingControl {
        release_sender,
        started_receiver,
    } = ThreadBlockingControl::register(&mut manager);
    let (sibling_observation_sender, sibling_observation_receiver) = oneshot::channel::<()>();

    manager.register_service(ObservableService {
        observation_sender: sibling_observation_sender.into(),
    });

    let observed_run = ObservedRun::concurrently(run_with_default_shutdown(manager), async move {
        let sibling_observation = started_receiver
            .await
            .and(sibling_observation_receiver.await);

        drop(release_sender);

        sibling_observation
    })
    .await;

    assert!(matches!(
        observed_run
            .into_run_output()
            .map(ServiceShutdownOutcomeCollection::into_result),
        Ok(Ok(()))
    ));
}
