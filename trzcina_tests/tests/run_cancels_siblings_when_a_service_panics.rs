use trzcina::service_manager::ServiceManager;
use trzcina_tests::panicking_service::PanickingService;
use trzcina_tests::run_with_default_shutdown::run_with_default_shutdown;
use trzcina_tests::waiting_observers::WaitingObservers;

const PANIC_MESSAGE: &str = "deliberately panicking to cancel siblings";
const SIBLINGS_TO_SHOW_ALL_ARE_CANCELLED: usize = 2;

#[tokio::test]
async fn cancels_siblings_when_a_service_panics() {
    let mut manager = ServiceManager::default();

    manager.register_service(PanickingService {
        panic_payload: Box::new(PANIC_MESSAGE),
    });

    let sibling_observers =
        WaitingObservers::register(&mut manager, SIBLINGS_TO_SHOW_ALL_ARE_CANCELLED);

    run_with_default_shutdown(manager).await;

    assert!(sibling_observers.all_observed());
}
