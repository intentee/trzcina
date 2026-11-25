use tokio::sync::oneshot;
use tokio::sync::oneshot::error::RecvError;
use tokio_util::sync::CancellationToken;

use trzcina::service_manager::ServiceManager;
use trzcina_tests::cancellation_ignoring_service::CancellationIgnoringService;

#[tokio::test]
async fn aborts_running_services_when_dropped() {
    let (held_sender, held_receiver) = oneshot::channel::<()>();
    let mut manager = ServiceManager::default();

    manager.register_service(CancellationIgnoringService {
        held_until_abort: held_sender,
    });

    drop(manager.start(CancellationToken::new()));

    assert!(matches!(held_receiver.await, Err(RecvError { .. })));
}
