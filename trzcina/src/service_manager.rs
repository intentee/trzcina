use tokio_util::sync::CancellationToken;

use crate::registered_service::RegisteredService;
use crate::running_service_collection::RunningServiceCollection;
use crate::service::Service;
use crate::service_bundle::ServiceBundle;
use crate::startable_service_collection::StartableServiceCollection;

/// Registers services and starts them concurrently, each on its own tokio task.
#[derive(Default)]
pub struct ServiceManager {
    services: Vec<RegisteredService>,
}

impl ServiceManager {
    /// # Errors
    ///
    /// Returns the error of the bundle when it cannot provide its services; no service of the bundle is registered then.
    pub async fn register_bundle<TServiceBundle: ServiceBundle>(
        &mut self,
        bundle: TServiceBundle,
    ) -> Result<(), TServiceBundle::Error> {
        self.services.extend(bundle.services().await?);

        Ok(())
    }

    pub fn register_service(&mut self, service: impl Service) {
        self.services.push(RegisteredService::new(service));
    }

    /// Spawns every registered service on the current tokio runtime.
    ///
    /// # Panics
    ///
    /// Panics when a service is registered and this is called outside a tokio runtime, as `tokio::spawn` does.
    #[must_use]
    pub fn start(self, cancellation_token: CancellationToken) -> RunningServiceCollection {
        RunningServiceCollection::start(self.services, cancellation_token)
    }
}

#[cfg(test)]
mod tests {
    use std::fmt;
    use std::future::ready;

    use futures_util::FutureExt;

    use crate::registered_service::RegisteredService;
    use crate::service_bundle::ServiceBundle;

    use super::ServiceManager;

    impl ServiceBundle for fmt::Error {
        type Error = Self;

        fn services(self) -> impl Future<Output = Result<Vec<RegisteredService>, Self>> + Send {
            ready(Err(self))
        }
    }

    #[test]
    fn propagates_error_from_bundle() {
        assert!(matches!(
            ServiceManager::default()
                .register_bundle(fmt::Error)
                .now_or_never(),
            Some(Err(fmt::Error))
        ));
    }
}
