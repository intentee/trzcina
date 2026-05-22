use std::time::Duration;

use trzcina::ServiceShutdownOptions;

#[test]
fn can_be_cloned() {
    let original = ServiceShutdownOptions {
        cooperative_deadline: Duration::from_millis(7),
        abort_deadline: Duration::from_millis(11),
    };
    let cloned = original.clone();
    assert_eq!(original.cooperative_deadline, cloned.cooperative_deadline);
    assert_eq!(original.abort_deadline, cloned.abort_deadline);
}
