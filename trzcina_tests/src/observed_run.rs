use futures_util::future::join;
use tokio_util::sync::CancellationToken;

use trzcina::service::Service;
use trzcina::service_manager::ServiceManager;
use trzcina::service_shutdown_options::ServiceShutdownOptions;
use trzcina::service_shutdown_outcome_collection::ServiceShutdownOutcomeCollection;
use trzcina_ticker::ticker::Ticker;
use trzcina_ticker::ticker_service::TickerService;

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
        let (run_output, observation) = join(run, observed).await;

        Self {
            observation,
            run_output,
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
