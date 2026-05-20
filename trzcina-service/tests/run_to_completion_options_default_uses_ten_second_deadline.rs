use std::time::Duration;

use trzcina_service::RunToCompletionOptions;

#[test]
fn default_uses_ten_second_deadline() {
    let options = RunToCompletionOptions::default();
    assert_eq!(options.shutdown_deadline, Duration::from_secs(10));
}
