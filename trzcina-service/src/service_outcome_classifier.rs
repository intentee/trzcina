use std::any::Any;
use std::future::Future;
use std::panic::AssertUnwindSafe;

use futures_util::FutureExt;
use log::error;
use log::info;

use crate::service_shutdown_outcome::ServiceShutdownOutcome;

fn panic_payload_to_string(panic_payload: Box<dyn Any + Send>) -> String {
    if let Some(static_str_message) = panic_payload.downcast_ref::<&'static str>() {
        return (*static_str_message).to_owned();
    }
    if let Ok(boxed_message) = panic_payload.downcast::<String>() {
        return *boxed_message;
    }
    "non-string panic payload".to_owned()
}

pub async fn classify_future_outcome<TServiceFuture>(
    service_name: &'static str,
    run_future: TServiceFuture,
) -> ServiceShutdownOutcome
where
    TServiceFuture: Future<Output = anyhow::Result<()>>,
{
    info!("Service {service_name:?} starting");
    let panic_caught_outcome = AssertUnwindSafe(run_future).catch_unwind().await;

    match panic_caught_outcome {
        Ok(Ok(())) => {
            info!("Service {service_name:?} stopped");
            ServiceShutdownOutcome::Completed
        }
        Ok(Err(service_error)) => {
            error!("Service {service_name:?} error: {service_error:#?}");
            ServiceShutdownOutcome::Errored(service_error)
        }
        Err(panic_payload) => {
            let panic_message = panic_payload_to_string(panic_payload);
            error!("Service {service_name:?} panicked: {panic_message}");
            ServiceShutdownOutcome::Panicked(panic_message)
        }
    }
}
