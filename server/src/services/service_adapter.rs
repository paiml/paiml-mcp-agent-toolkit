//! Adapters to help existing services implement the Service trait
//!
//! This module provides adapter patterns to integrate legacy services
//! with the new unified service architecture.

use super::service_base::{Service, ServiceMetrics};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Generic adapter for converting existing services to Service trait
pub struct ServiceAdapter<T, I, O> {
    inner: Arc<T>,
    metrics: Arc<RwLock<ServiceMetrics>>,
    _phantom: PhantomData<(I, O)>,
}

impl<T, I, O> ServiceAdapter<T, I, O> {
    pub fn new(inner: T) -> Self {
        Self {
            inner: Arc::new(inner),
            metrics: Arc::new(RwLock::new(ServiceMetrics::default())),
            _phantom: PhantomData,
        }
    }
    
    pub fn inner(&self) -> &T {
        &self.inner
    }
}

/// Macro to quickly implement Service trait for adapted services
#[macro_export]
macro_rules! impl_service_adapter {
    ($adapter:ty, $input:ty, $output:ty, $process_fn:expr) => {
        #[async_trait::async_trait]
        impl Service for $adapter {
            type Input = $input;
            type Output = $output;
            type Error = anyhow::Error;
            
            async fn process(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
                let start = std::time::Instant::now();
                let result = $process_fn(&self.inner, input).await;
                let duration = start.elapsed();
                
                let mut metrics = self.metrics.write().await;
                metrics.record_request(duration, result.is_ok());
                
                result
            }
            
            fn metrics(&self) -> ServiceMetrics {
                self.metrics.blocking_read().clone()
            }
        }
    };
}

/// Example: ComplexityService adapter
pub mod complexity_adapter {
    use super::*;
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
        pub fn new_complexity_service() -> Self {
            ServiceAdapter::new(())
        }
    }
    
    async fn process_complexity(
        _inner: &(),
        _input: ComplexityInput,
    ) -> Result<ComplexityOutput> {
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

/// Example: RefactorService adapter  
pub mod refactor_adapter {
    use super::*;
    use std::path::PathBuf;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RefactorInput {
        pub file_path: PathBuf,
        pub refactor_type: RefactorType,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum RefactorType {
        ExtractFunction,
        SimplifyCondition,
        RemoveDeadCode,
        Auto,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RefactorOutput {
        pub success: bool,
        pub changes: Vec<Change>,
        pub message: String,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Change {
        pub file: String,
        pub line: usize,
        pub before: String,
        pub after: String,
    }
    
    pub type RefactorServiceAdapter = ServiceAdapter<(), RefactorInput, RefactorOutput>;
    
    impl RefactorServiceAdapter {
        pub fn new_refactor_service() -> Self {
            ServiceAdapter::new(())
        }
    }
    
    async fn process_refactor(
        _inner: &(),
        _input: RefactorInput,
    ) -> Result<RefactorOutput> {
        // Would call actual refactor engine here
        Ok(RefactorOutput {
            success: true,
            changes: vec![],
            message: "Refactoring completed".to_string(),
        })
    }
    
    impl_service_adapter!(
        RefactorServiceAdapter,
        RefactorInput,
        RefactorOutput,
        process_refactor
    );
}

/// Registry builder with fluent API for registering services
pub struct ServiceRegistryBuilder {
    registry: super::service_base::ServiceRegistry,
}

impl ServiceRegistryBuilder {
    pub fn new() -> Self {
        Self {
            registry: super::service_base::ServiceRegistry::new(),
        }
    }
    
    /// Register an analysis service
    pub fn with_analysis_service(self) -> Self {
        let service = super::analysis_service::AnalysisService::new();
        self.registry.register(service);
        self
    }
    
    /// Register a quality gate service
    pub fn with_quality_gate_service(self) -> Self {
        let service = super::quality_gate_service::QualityGateService::new();
        self.registry.register(service);
        self
    }
    
    /// Register a complexity service adapter
    pub fn with_complexity_service(self) -> Self {
        let service = complexity_adapter::ComplexityServiceAdapter::new_complexity_service();
        self.registry.register(service);
        self
    }
    
    /// Register a refactor service adapter
    pub fn with_refactor_service(self) -> Self {
        let service = refactor_adapter::RefactorServiceAdapter::new_refactor_service();
        self.registry.register(service);
        self
    }
    
    /// Build the registry
    pub fn build(self) -> super::service_base::ServiceRegistry {
        self.registry
    }
}

impl Default for ServiceRegistryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_service_registry_builder() {
        let registry = ServiceRegistryBuilder::new()
            .with_analysis_service()
            .with_quality_gate_service()
            .build();
        
        // Check that services are registered
        let services = registry.list_services();
        assert!(services.len() >= 2);
    }
}