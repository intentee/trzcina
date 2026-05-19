# trzcina

Async service lifecycle orchestration for Rust. Run a set of long-lived async services concurrently, cancel siblings when one finishes, surface errors and panics through a typed outcome collection, and enforce an absolute shutdown deadline.

Cancellation is cooperative: every service must `.await` on the `CancellationToken` passed to `run` (typically inside a `tokio::select!`) and return when it fires. Services that ignore the token are aborted when the shutdown deadline expires.

If your service spawns child tasks via `tokio::spawn`, clone the cancellation token into them — trzcina only bounds the service's own task, not detached children. (Inside a `LocalService`, `tokio::task::spawn_local` is scoped to trzcina's `LocalSet` and is cancelled automatically; `tokio::spawn` still escapes.)

## Usage

A service that simply waits for shutdown:

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

A service that does periodic work and yields to cancellation on every tick:

```rust
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use tokio_util::sync::CancellationToken;
use trzcina::Service;

struct TickerService;

#[async_trait]
impl Service for TickerService {
    async fn run(&mut self, cancellation_token: CancellationToken) -> Result<()> {
        let mut ticker = tokio::time::interval(Duration::from_secs(1));
        loop {
            tokio::select! {
                biased;
                () = cancellation_token.cancelled() => return Ok(()),
                _ = ticker.tick() => {
                    // do periodic work here
                }
            }
        }
    }
}
```
