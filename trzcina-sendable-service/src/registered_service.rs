use crate::service::Service;

pub struct RegisteredService {
    pub name: &'static str,
    pub service: Box<dyn Service>,
}
