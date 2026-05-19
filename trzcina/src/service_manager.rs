use anyhow::Result;
use tokio_util::sync::CancellationToken;

use crate::ServiceBundle;
use crate::registered_service::RegisteredService;
use crate::running_service_collection::RunningServiceCollection;
use crate::service::Service;

#[derive(Default)]
pub struct ServiceManager {
    services: Vec<RegisteredService>,
}

impl ServiceManager {
    pub async fn register_bundle<TServiceBundle: ServiceBundle>(
        &mut self,
        bundle: TServiceBundle,
    ) -> Result<()> {
        for service in bundle.services().await? {
            let name = service.name();
            self.services.push(RegisteredService { name, service });
        }

        Ok(())
    }

    pub fn register_service(&mut self, service: impl Service) {
        let name = service.name();
        self.services.push(RegisteredService {
            name,
            service: Box::new(service),
        });
    }

    #[must_use]
    pub fn start(self, cancellation_token: CancellationToken) -> RunningServiceCollection {
        RunningServiceCollection::start(self.services, cancellation_token)
    }
}
