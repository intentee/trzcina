use tokio::sync::oneshot;

use trzcina::service_manager::ServiceManager;

use crate::waiting_observer_service::WaitingObserverService;

pub struct WaitingObservers {
    observation_receivers: Vec<oneshot::Receiver<()>>,
}

impl WaitingObservers {
    pub fn register(service_manager: &mut ServiceManager, observer_count: usize) -> Self {
        Self {
            observation_receivers: (0..observer_count)
                .map(|_observer_index| {
                    let (observation_sender, observation_receiver) = oneshot::channel::<()>();

                    service_manager.register_service(WaitingObserverService {
                        observation_sender: observation_sender.into(),
                    });

                    observation_receiver
                })
                .collect(),
        }
    }

    #[must_use]
    pub fn all_observed(self) -> bool {
        self.observation_receivers
            .into_iter()
            .all(|mut observation_receiver| observation_receiver.try_recv() == Ok(()))
    }
}
