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
    use std::panic::AssertUnwindSafe;
    use std::panic::catch_unwind;

    use futures_util::FutureExt;
    use tokio_util::sync::CancellationToken;

    use crate::registered_service::RegisteredService;
    use crate::service::Service;
    use crate::service_bundle::ServiceBundle;

    use super::ServiceManager;

    impl Service for fmt::Error {
        type Error = Self;

        fn run(
            self: Box<Self>,
            _cancellation_token: CancellationToken,
        ) -> impl Future<Output = Result<(), Self>> + Send {
            ready(Err(*self))
        }
    }

    impl ServiceBundle for fmt::Error {
        type Error = Self;

        fn services(self) -> impl Future<Output = Result<Vec<RegisteredService>, Self>> + Send {
            ready(Err(self))
        }
    }

    #[test]
    fn panics_outside_a_tokio_runtime() {
        let mut service_manager = ServiceManager::default();

        service_manager.register_service(fmt::Error);

        assert!(
            catch_unwind(AssertUnwindSafe(move || {
                service_manager.start(CancellationToken::new())
            }))
            .is_err()
        );
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
