use std::time::Duration;

use trzcina_tests::cooperative_shutdown::CooperativeShutdown;
use trzcina_tests::timed_run::TimedRun;

#[tokio::test(start_paused = true)]
async fn ends_cooperative_phase_once_services_stop() {
    let CooperativeShutdown {
        cooperative_observers,
        timed_shutdown: TimedRun { elapsed, .. },
    } = CooperativeShutdown::measure().await;

    assert_eq!(elapsed, Duration::ZERO);
    assert!(cooperative_observers.all_observed());
}
