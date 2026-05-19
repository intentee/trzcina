# trzcina

Async service lifecycle orchestration for Rust. Run a set of long-lived async services concurrently, cancel siblings when one finishes, surface errors and panics through a typed outcome collection, and enforce an absolute shutdown deadline.

## Usage

```rust
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use tokio_util::sync::CancellationToken;
use trzcina::{Service, ServiceManager};

struct EchoService;

#[async_trait]
impl Service for EchoService {
    async fn run(&mut self, cancellation_token: CancellationToken) -> Result<()> {
        cancellation_token.cancelled().await;
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut service_manager = ServiceManager::default();
    service_manager.register_service(EchoService);

    let running = service_manager.start(CancellationToken::new());
    running
        .run_to_completion(Duration::from_secs(10))
        .await
        .into_result()?;
    Ok(())
}
```
