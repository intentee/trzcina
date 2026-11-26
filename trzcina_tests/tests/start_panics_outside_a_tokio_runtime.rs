use std::panic::AssertUnwindSafe;
use std::panic::catch_unwind;

use tokio_util::sync::CancellationToken;

use trzcina::service_manager::ServiceManager;
use trzcina_tests::completing_service::CompletingService;

#[test]
fn panics_outside_a_tokio_runtime() {
    let mut manager = ServiceManager::default();

    manager.register_service(CompletingService);

    assert!(
        catch_unwind(AssertUnwindSafe(move || {
            manager.start(CancellationToken::new())
        }))
        .is_err()
    );
}
