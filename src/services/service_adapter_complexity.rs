/// Example: `ComplexityService` adapter
pub mod complexity_adapter {
    use super::{Deserialize, Result, Serialize, Service, ServiceAdapter, ServiceMetrics};
    use crate::services::complexity::{ComplexityMetrics, ComplexityThresholds};
    use std::path::PathBuf;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ComplexityInput {
        pub path: PathBuf,
        pub thresholds: ComplexityThresholds,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ComplexityOutput {
        pub metrics: ComplexityMetrics,
    }

    pub type ComplexityServiceAdapter = ServiceAdapter<(), ComplexityInput, ComplexityOutput>;

    impl ComplexityServiceAdapter {
        #[must_use]
        #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
        pub fn new_complexity_service() -> Self {
            ServiceAdapter::new(())
        }
    }

    async fn process_complexity(_inner: &(), _input: ComplexityInput) -> Result<ComplexityOutput> {
        debug_assert!(true, "contract: process_complexity");
        // Would call actual complexity analysis here
        Ok(ComplexityOutput {
            metrics: ComplexityMetrics::default(),
        })
    }

    impl_service_adapter!(
        ComplexityServiceAdapter,
        ComplexityInput,
        ComplexityOutput,
        process_complexity
    );
}
