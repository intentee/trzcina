use std::time::Duration;

use trzcina::ServiceShutdownOptions;

#[test]
fn default_uses_ten_second_deadlines_for_both_phases() {
    let ServiceShutdownOptions {
        cooperative_deadline,
        abort_deadline,
    } = ServiceShutdownOptions::default();

    assert_eq!(cooperative_deadline, Duration::from_secs(10));
    assert_eq!(abort_deadline, Duration::from_secs(10));
}
