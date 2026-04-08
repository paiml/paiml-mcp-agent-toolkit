/// Registry builder with fluent API for registering services
pub struct ServiceRegistryBuilder {
    registry: super::service_base::ServiceRegistry,
}

impl ServiceRegistryBuilder {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self {
            registry: super::service_base::ServiceRegistry::new(),
        }
    }

    /// Register an analysis service
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_analysis_service(self) -> Self {
        let service = super::analysis_service::AnalysisService::new();
        self.registry.register(service);
        self
    }

    /// Register a quality gate service
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_quality_gate_service(self) -> Self {
        let service = super::quality_gate_service::QualityGateService::new();
        self.registry.register(service);
        self
    }

    /// Register a complexity service adapter
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_complexity_service(self) -> Self {
        let service = complexity_adapter::ComplexityServiceAdapter::new_complexity_service();
        self.registry.register(service);
        self
    }

    /// Register a refactor service adapter
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_refactor_service(self) -> Self {
        let service = refactor_adapter::RefactorServiceAdapter::new_refactor_service();
        self.registry.register(service);
        self
    }

    /// Build the registry
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn build(self) -> super::service_base::ServiceRegistry {
        self.registry
    }
}

impl Default for ServiceRegistryBuilder {
    fn default() -> Self {
        Self::new()
    }
}
