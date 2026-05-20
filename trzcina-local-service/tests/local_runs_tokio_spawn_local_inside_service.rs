use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::oneshot;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;
use trzcina_local_service::LocalService;
use trzcina_local_service::LocalServiceManager;
use trzcina_service::Manager;
use trzcina_service::RunToCompletionOptions;
use trzcina_service::RunningCollection;
use trzcina_service::ServiceShutdownOutcome;

const CHILD_TASK_RESULT: u32 = 42;

struct SpawnLocalUsingService {
    observation_tx: Option<oneshot::Sender<u32>>,
}

#[async_trait(?Send)]
impl LocalService for SpawnLocalUsingService {
    async fn run(&mut self, _cancellation_token: CancellationToken) -> Result<()> {
        let child_join_handle = tokio::task::spawn_local(async { CHILD_TASK_RESULT });
        let observed = child_join_handle.await?;
        if let Some(observation_tx) = self.observation_tx.take() {
            observation_tx.send(observed).unwrap();
        }
        Ok(())
    }
}

#[tokio::test]
async fn local_runs_tokio_spawn_local_inside_service() {
    let (observation_tx, mut observation_rx) = oneshot::channel::<u32>();

    let mut manager = LocalServiceManager::default();
    manager.register_service(SpawnLocalUsingService {
        observation_tx: Some(observation_tx),
    });

    let report = timeout(
        Duration::from_secs(5),
        manager
            .start(CancellationToken::new())
            .run_to_completion(RunToCompletionOptions {
                shutdown_deadline: Duration::from_secs(1),
            }),
    )
    .await
    .unwrap();

    assert_eq!(report.outcomes().len(), 1);
    assert!(matches!(
        report.outcomes()[0].outcome,
        ServiceShutdownOutcome::Completed
    ));
    assert_eq!(observation_rx.try_recv().unwrap(), CHILD_TASK_RESULT);
}
