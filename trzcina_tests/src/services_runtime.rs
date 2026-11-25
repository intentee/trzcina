use std::io;

use tokio::runtime::Builder;
use tokio::runtime::Runtime;
use tokio_util::sync::CancellationToken;

use trzcina::running_service_collection::RunningServiceCollection;
use trzcina::service_manager::ServiceManager;

use crate::block_on_paused_runtime::block_on_paused_runtime;

pub struct ServicesRuntime {
    pub running_services: RunningServiceCollection,
    pub runtime: Runtime,
}

impl ServicesRuntime {
    /// # Errors
    ///
    /// Returns the `io::Error` of building the runtime the services are spawned on.
    pub fn start(
        service_manager: ServiceManager,
        cancellation_token: CancellationToken,
    ) -> io::Result<Self> {
        let runtime = Builder::new_current_thread().enable_all().build()?;
        let runtime_context = runtime.enter();
        let running_services = service_manager.start(cancellation_token);

        drop(runtime_context);

        Ok(Self {
            running_services,
            runtime,
        })
    }

    /// # Errors
    ///
    /// Returns the `io::Error` of building the paused runtime the shutdown runs on.
    pub fn shut_down_while_undriven<TStartShutdown, TShutdown>(
        self,
        start_shutdown: TStartShutdown,
    ) -> io::Result<TShutdown::Output>
    where
        TStartShutdown: FnOnce(RunningServiceCollection) -> TShutdown,
        TShutdown: Future,
    {
        let Self {
            running_services,
            runtime,
        } = self;
        let shutdown_output = block_on_paused_runtime(start_shutdown(running_services));

        drop(runtime);

        shutdown_output
    }
}
