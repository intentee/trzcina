use futures_util::FutureExt;
use futures_util::future::BoxFuture;
use log::error;
use log::info;
use tokio_util::sync::CancellationToken;

use crate::service::Service;
use crate::service_shutdown_outcome::ServiceShutdownOutcome;

pub trait RunnableService: Send {
    fn run_to_outcome(
        self: Box<Self>,
        service_name: &'static str,
        cancellation_token: CancellationToken,
    ) -> BoxFuture<'static, ServiceShutdownOutcome>;
}

impl<TService: Service> RunnableService for TService {
    fn run_to_outcome(
        self: Box<Self>,
        service_name: &'static str,
        cancellation_token: CancellationToken,
    ) -> BoxFuture<'static, ServiceShutdownOutcome> {
        async move {
            match self.run(cancellation_token).await {
                Ok(()) => {
                    info!("Service {service_name:?} stopped");
                    ServiceShutdownOutcome::Completed
                }
                Err(service_error) => {
                    error!("Service {service_name:?} error: {service_error:#?}");
                    ServiceShutdownOutcome::Errored(Box::new(service_error))
                }
            }
        }
        .boxed()
    }
}
