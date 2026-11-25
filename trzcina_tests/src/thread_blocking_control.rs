use std::convert::Infallible;
use std::sync::mpsc::Sender;
use std::sync::mpsc::channel;

use tokio::sync::oneshot;

use trzcina::service_manager::ServiceManager;

use crate::thread_blocking_service::ThreadBlockingService;

pub struct ThreadBlockingControl {
    pub release_sender: Sender<Infallible>,
    pub started_receiver: oneshot::Receiver<()>,
}

impl ThreadBlockingControl {
    pub fn register(service_manager: &mut ServiceManager) -> Self {
        let (release_sender, release_receiver) = channel::<Infallible>();
        let (started_sender, started_receiver) = oneshot::channel::<()>();

        service_manager.register_service(ThreadBlockingService {
            release_receiver,
            started_sender: started_sender.into(),
        });

        Self {
            release_sender,
            started_receiver,
        }
    }
}
