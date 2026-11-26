# trzcina

Async service lifecycle orchestration for Rust.

```rust
use std::convert::Infallible;

use tokio_util::sync::CancellationToken;
use trzcina::service::Service;
use trzcina::service_manager::ServiceManager;
use trzcina::service_shutdown_error::ServiceShutdownError;
use trzcina::service_shutdown_options::ServiceShutdownOptions;

struct EchoService;

impl Service for EchoService {
    type Error = Infallible;

    async fn run(self: Box<Self>, cancellation_token: CancellationToken) -> Result<(), Infallible> {
        cancellation_token.cancelled().await;

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Vec<ServiceShutdownError>> {
    let mut service_manager = ServiceManager::default();
    service_manager.register_service(EchoService);

    service_manager
        .start(CancellationToken::new())
        .run_to_completion(ServiceShutdownOptions::default())
        .await
        .into_result()
}
```
