use std::convert::Infallible;
use std::future::ready;

use trzcina::registered_service::RegisteredService;
use trzcina::service_bundle::ServiceBundle;

pub struct PrebuiltServicesBundle {
    pub registered_services: Vec<RegisteredService>,
}

impl ServiceBundle for PrebuiltServicesBundle {
    type Error = Infallible;

    fn services(self) -> impl Future<Output = Result<Vec<RegisteredService>, Infallible>> + Send {
        ready(Ok(self.registered_services))
    }
}
