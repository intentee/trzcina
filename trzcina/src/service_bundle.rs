use std::error::Error;

use crate::registered_service::RegisteredService;

pub trait ServiceBundle {
    type Error: Error;

    fn services(self) -> impl Future<Output = Result<Vec<RegisteredService>, Self::Error>> + Send;
}
