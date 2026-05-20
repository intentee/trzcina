use anyhow::Result;
use tokio_util::sync::CancellationToken;
use trzcina_service::Manager;

use crate::local_registered_service::LocalRegisteredService;
use crate::local_running_service_collection::LocalRunningServiceCollection;
use crate::local_service::LocalService;
use crate::local_service_bundle::LocalServiceBundle;

#[derive(Default)]
pub struct LocalServiceManager {
    services: Vec<LocalRegisteredService>,
}

impl LocalServiceManager {
    pub async fn register_bundle<TLocalServiceBundle: LocalServiceBundle>(
        &mut self,
        bundle: TLocalServiceBundle,
    ) -> Result<()> {
        for service in bundle.services().await? {
            let name = service.name();
            self.services.push(LocalRegisteredService { name, service });
        }

        Ok(())
    }

    pub fn register_service(&mut self, service: impl LocalService) {
        let name = service.name();
        self.services.push(LocalRegisteredService {
            name,
            service: Box::new(service),
        });
    }
}

impl Manager for LocalServiceManager {
    type Running = LocalRunningServiceCollection;

    fn start(self, cancellation_token: CancellationToken) -> LocalRunningServiceCollection {
        LocalRunningServiceCollection::start(self.services, cancellation_token)
    }
}
