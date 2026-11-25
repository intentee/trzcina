use trzcina::service_manager::ServiceManager;
use trzcina::service_shutdown_options::ServiceShutdownOptions;
use trzcina::service_shutdown_outcome_collection::ServiceShutdownOutcomeCollection;

use crate::cancellation_ignoring_service::CancellationIgnoringService;
use crate::observed_run::ObservedRun;
use crate::timed_run::TimedRun;
use crate::waiting_observers::WaitingObservers;

const COOPERATIVE_SERVICES: usize = 1;

pub struct CooperativeShutdown {
    pub cooperative_observers: WaitingObservers,
    pub timed_shutdown: TimedRun<ServiceShutdownOutcomeCollection>,
}

impl CooperativeShutdown {
    pub async fn measure() -> Self {
        Self::measure_after_registering(|_service_manager| (), ServiceShutdownOptions::default())
            .await
    }

    pub async fn measure_alongside_aborted_service(
        shutdown_options: ServiceShutdownOptions,
    ) -> Self {
        Self::measure_after_registering(
            |service_manager| {
                service_manager.register_service(CancellationIgnoringService {
                    held_until_abort: (),
                });
            },
            shutdown_options,
        )
        .await
    }

    async fn measure_after_registering<TRegisterServices: FnOnce(&mut ServiceManager)>(
        register_services: TRegisterServices,
        shutdown_options: ServiceShutdownOptions,
    ) -> Self {
        let mut service_manager = ServiceManager::default();
        let cooperative_observers =
            WaitingObservers::register(&mut service_manager, COOPERATIVE_SERVICES);

        register_services(&mut service_manager);

        Self {
            cooperative_observers,
            timed_shutdown: TimedRun::measure(async {
                ObservedRun::shutdown_after_observation(service_manager, shutdown_options, async {})
                    .await
                    .run_output
            })
            .await,
        }
    }
}
