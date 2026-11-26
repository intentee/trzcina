use std::pin::pin;
use std::task::Poll;

use futures_util::future::Either;
use futures_util::future::poll_fn;
use tokio::sync::oneshot;
use tokio::sync::oneshot::error::RecvError;
use tokio_util::sync::CancellationToken;

use trzcina::service::Service;
use trzcina::service_manager::ServiceManager;
use trzcina::service_shutdown_error::ServiceShutdownError;
use trzcina::service_shutdown_options::ServiceShutdownOptions;
use trzcina::service_shutdown_outcome_collection::ServiceShutdownOutcomeCollection;
use trzcina_ticker::ticker::Ticker;
use trzcina_ticker::ticker_service::TickerService;

use crate::run_with_default_shutdown::run_with_default_shutdown;

pub struct ObservedRun<TRunOutput, TObservation> {
    pub observation: TObservation,
    pub run_output: TRunOutput,
}

impl<TRunOutput, TObservation> ObservedRun<TRunOutput, TObservation> {
    pub async fn concurrently<TRun, TObserved>(run: TRun, observed: TObserved) -> Self
    where
        TRun: Future<Output = TRunOutput>,
        TObserved: Future<Output = TObservation>,
    {
        let mut run = pin!(run);
        let mut observed = pin!(observed);
        let first_finished = poll_fn(|context| match run.as_mut().poll(context) {
            Poll::Ready(run_output) => Poll::Ready(Either::Left(run_output)),
            Poll::Pending => observed.as_mut().poll(context).map(Either::Right),
        })
        .await;

        match first_finished {
            Either::Left(run_output) => Self {
                observation: observed.await,
                run_output,
            },
            Either::Right(observation) => Self {
                observation,
                run_output: run.await,
            },
        }
    }

    async fn cancel_after_observation<TStartRun, TRun, TObserve, TObserved>(
        start_run: TStartRun,
        observe: TObserve,
    ) -> Self
    where
        TStartRun: FnOnce(CancellationToken) -> TRun,
        TRun: Future<Output = TRunOutput>,
        TObserve: FnOnce(CancellationToken) -> TObserved,
        TObserved: Future<Output = TObservation>,
    {
        let cancellation_token = CancellationToken::new();
        let observed = observe(cancellation_token.clone());

        Self::concurrently(start_run(cancellation_token.clone()), async move {
            let _cancellation_on_observation = cancellation_token.drop_guard();

            observed.await
        })
        .await
    }
}

impl<TRunOutput, TObservationError> ObservedRun<TRunOutput, Result<(), TObservationError>> {
    /// # Errors
    ///
    /// Returns the error of the observation the run was synchronized on.
    pub fn into_run_output(self) -> Result<TRunOutput, TObservationError> {
        let Self {
            observation,
            run_output,
        } = self;

        observation.map(|()| run_output)
    }
}

impl<TObservation> ObservedRun<ServiceShutdownOutcomeCollection, TObservation> {
    pub async fn shutdown_after_observation<TObserved: Future<Output = TObservation>>(
        service_manager: ServiceManager,
        shutdown_options: ServiceShutdownOptions,
        observed: TObserved,
    ) -> Self {
        Self::cancel_after_observation(
            |cancellation_token| {
                service_manager
                    .start(cancellation_token)
                    .run_to_completion(shutdown_options)
            },
            |_cancellation_token| observed,
        )
        .await
    }
}

impl ObservedRun<ServiceShutdownOutcomeCollection, Result<(), RecvError>> {
    /// # Errors
    ///
    /// Returns the `RecvError` when the observed service stopped without sending its observation.
    pub async fn shutdown_result_releasing_after_observation(
        service_manager: ServiceManager,
        observation_receiver: oneshot::Receiver<()>,
        release: CancellationToken,
    ) -> Result<Result<(), Vec<ServiceShutdownError>>, RecvError> {
        Self::concurrently(run_with_default_shutdown(service_manager), async move {
            let observation = observation_receiver.await;

            release.cancel();

            observation
        })
        .await
        .into_run_output()
        .map(ServiceShutdownOutcomeCollection::into_result)
    }
}

impl<TTickerError, TObservation> ObservedRun<Result<(), TTickerError>, TObservation> {
    pub async fn tick_until_observation<TTicker, TObserve, TObserved>(
        ticker: TTicker,
        observe: TObserve,
    ) -> Self
    where
        TTicker: Ticker<Error = TTickerError>,
        TObserve: FnOnce(CancellationToken) -> TObserved,
        TObserved: Future<Output = TObservation>,
    {
        Self::cancel_after_observation(
            |cancellation_token| Box::new(TickerService { ticker }).run(cancellation_token),
            observe,
        )
        .await
    }
}
