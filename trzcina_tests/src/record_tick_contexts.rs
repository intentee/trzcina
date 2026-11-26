use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio_util::sync::CancellationToken;

use trzcina_ticker::tick_context::TickContext;
use trzcina_ticker::ticker::Ticker;

use crate::observed_run::ObservedRun;
use crate::tick_recorder::TickRecorder;

async fn receive_tick_contexts(
    tick_context_receiver: &mut UnboundedReceiver<TickContext>,
    tick_count: usize,
) -> Vec<TickContext> {
    let mut tick_contexts = Vec::with_capacity(tick_count);

    for _received_tick in 0..tick_count {
        tick_contexts.extend(tick_context_receiver.recv().await);
    }

    tick_contexts
}

async fn receive_tick_contexts_recorded_until_ticker_dropped(
    mut tick_context_receiver: UnboundedReceiver<TickContext>,
    mut tick_contexts: Vec<TickContext>,
) -> Vec<TickContext> {
    while let Some(tick_context) = tick_context_receiver.recv().await {
        tick_contexts.push(tick_context);
    }

    tick_contexts
}

/// # Errors
///
/// Returns the error that stopped the ticker.
pub async fn record_tick_contexts<TTicker, TBuildTicker, TBeforeReceiving, TBeforeReceivingStep>(
    build_ticker: TBuildTicker,
    before_receiving: TBeforeReceiving,
    ticks_before_cancellation: usize,
) -> Result<Vec<TickContext>, TTicker::Error>
where
    TTicker: Ticker,
    TBuildTicker: FnOnce(TickRecorder) -> TTicker,
    TBeforeReceiving: FnOnce(CancellationToken) -> TBeforeReceivingStep,
    TBeforeReceivingStep: Future<Output = ()>,
{
    let (tick_context_sender, mut tick_context_receiver) = mpsc::unbounded_channel();
    let receiving_end = &mut tick_context_receiver;
    let observed_run = ObservedRun::tick_until_observation(
        build_ticker(tick_context_sender.into()),
        move |cancellation_token| async move {
            before_receiving(cancellation_token).await;

            receive_tick_contexts(receiving_end, ticks_before_cancellation).await
        },
    )
    .await;
    let recorded_tick_contexts = receive_tick_contexts_recorded_until_ticker_dropped(
        tick_context_receiver,
        observed_run.observation,
    )
    .await;

    observed_run.run_output.map(|()| recorded_tick_contexts)
}
