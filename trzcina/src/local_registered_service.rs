use crate::local_service::LocalService;

pub struct LocalRegisteredService {
    pub name: &'static str,
    pub service: Box<dyn LocalService>,
}
