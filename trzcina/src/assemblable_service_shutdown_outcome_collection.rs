use crate::service_shutdown_outcome_with_service_name::ServiceShutdownOutcomeWithServiceName;

pub trait AssemblableServiceShutdownOutcomeCollection {
    #[must_use]
    fn assemble(outcomes: Vec<ServiceShutdownOutcomeWithServiceName>) -> Self;
}
