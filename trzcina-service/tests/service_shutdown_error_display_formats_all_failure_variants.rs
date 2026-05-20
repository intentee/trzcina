use anyhow::anyhow;
use trzcina_service::ServiceShutdownError;
use trzcina_service::ServiceShutdownOutcome;
use trzcina_service::ServiceShutdownOutcomeWithServiceName;

#[test]
fn display_formats_all_failure_variants() {
    let outcomes = vec![
        ServiceShutdownOutcomeWithServiceName {
            name: "completed_service",
            outcome: ServiceShutdownOutcome::Completed,
        },
        ServiceShutdownOutcomeWithServiceName {
            name: "errored_service",
            outcome: ServiceShutdownOutcome::Errored(anyhow!("service failed")),
        },
        ServiceShutdownOutcomeWithServiceName {
            name: "panicked_service",
            outcome: ServiceShutdownOutcome::Panicked("service panicked".to_owned()),
        },
        ServiceShutdownOutcomeWithServiceName {
            name: "aborted_service",
            outcome: ServiceShutdownOutcome::AbortedByShutdownDeadline,
        },
        ServiceShutdownOutcomeWithServiceName {
            name: "leaked_service",
            outcome: ServiceShutdownOutcome::LeakedBeyondAbortDeadline,
        },
    ];

    let error = ServiceShutdownError::new(outcomes);
    let formatted = format!("{error}");

    assert!(!formatted.is_empty());
    assert_eq!(error.failed_outcomes().len(), 5);
}
