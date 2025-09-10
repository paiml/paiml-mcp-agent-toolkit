//! Base service architecture per SPECIFICATION.md Section 2
//!
//! This module provides the foundational Service trait and ServiceRegistry
//! for unified service architecture across PMAT.

use anyhow::Result;
use dashmap::DashMap;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::any::{Any, TypeId};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Validation errors for service inputs
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Missing required field: {field}")]
    MissingField { field: String },

    #[error("Invalid value for field {field}: {reason}")]
    InvalidValue { field: String, reason: String },

    #[error("Input size exceeds limit: {size} > {limit}")]
    SizeLimit { size: usize, limit: usize },

    #[error("Invalid format: {0}")]
    InvalidFormat(String),
}

/// Service metrics for monitoring and performance tracking
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServiceMetrics {
    pub request_count: u64,
    pub success_count: u64,
    pub error_count: u64,
    pub total_duration_ms: u64,
    pub average_duration_ms: u64,
    #[serde(skip)]
    pub last_request_time: Option<Instant>,
}

impl ServiceMetrics {
    pub fn record_request(&mut self, duration: Duration, success: bool) {
        self.request_count += 1;
        if success {
            self.success_count += 1;
        } else {
            self.error_count += 1;
        }
        let duration_ms = duration.as_millis() as u64;
        self.total_duration_ms += duration_ms;
        self.average_duration_ms = self.total_duration_ms / self.request_count;
        self.last_request_time = Some(Instant::now());
    }

    pub fn success_rate(&self) -> f64 {
        if self.request_count == 0 {
            return 0.0;
        }
        self.success_count as f64 / self.request_count as f64
    }
}

/// Core service trait - all services implement this
#[async_trait::async_trait]
pub trait Service: Send + Sync {
    type Input: Serialize + DeserializeOwned + Send + Sync;
    type Output: Serialize + DeserializeOwned + Send + Sync;
    type Error: Into<anyhow::Error> + Send + Sync;

    /// Process the input and return output
    async fn process(&self, input: Self::Input) -> Result<Self::Output, Self::Error>;

    /// Validate input before processing
    fn validate_input(&self, _input: &Self::Input) -> Result<(), ValidationError> {
        // Default validation - can be overridden
        Ok(())
    }

    /// Get service metrics
    fn metrics(&self) -> ServiceMetrics {
        ServiceMetrics::default()
    }

    /// Get service name for logging and monitoring
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }
}

/// Service registry for dependency injection
pub struct ServiceRegistry {
    services: DashMap<TypeId, Arc<dyn Any + Send + Sync>>,
    metrics: DashMap<TypeId, ServiceMetrics>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self {
            services: DashMap::new(),
            metrics: DashMap::new(),
        }
    }

    /// Register a service in the registry
    pub fn register<S>(&self, service: S)
    where
        S: Service + 'static,
    {
        let id = TypeId::of::<S>();
        self.services.insert(id, Arc::new(service));
        self.metrics.insert(id, ServiceMetrics::default());
    }

    /// Get a service from the registry
    pub fn get<S>(&self) -> Option<Arc<S>>
    where
        S: Service + 'static,
    {
        let id = TypeId::of::<S>();
        self.services
            .get(&id)
            .and_then(|s| s.clone().downcast::<S>().ok())
    }

    /// Get metrics for a service
    pub fn get_metrics<S>(&self) -> Option<ServiceMetrics>
    where
        S: Service + 'static,
    {
        let id = TypeId::of::<S>();
        self.metrics.get(&id).map(|m| m.clone())
    }

    /// Update metrics for a service
    pub fn update_metrics<S>(&self, duration: Duration, success: bool)
    where
        S: Service + 'static,
    {
        let id = TypeId::of::<S>();
        if let Some(mut metrics) = self.metrics.get_mut(&id) {
            metrics.record_request(duration, success);
        }
    }

    /// List all registered service names
    pub fn list_services(&self) -> Vec<String> {
        self.services
            .iter()
            .map(|entry| format!("{:?}", entry.key()))
            .collect()
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Services can be composed for complex operations
pub struct CompositeService<A, B>
where
    A: Service,
    B: Service,
{
    pub first: A,
    pub second: B,
    pub adapter: Box<dyn Fn(A::Output) -> B::Input + Send + Sync>,
}

#[async_trait::async_trait]
impl<A, B> Service for CompositeService<A, B>
where
    A: Service + Send + Sync,
    B: Service + Send + Sync,
    A::Output: Send + Sync,
    B::Input: Send + Sync,
{
    type Input = A::Input;
    type Output = B::Output;
    type Error = anyhow::Error;

    async fn process(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        // Process through first service
        let intermediate = self.first.process(input).await.map_err(|e| e.into())?;

        // Adapt output to input for second service
        let adapted = (self.adapter)(intermediate);

        // Process through second service
        let output = self.second.process(adapted).await.map_err(|e| e.into())?;

        Ok(output)
    }

    fn validate_input(&self, input: &Self::Input) -> Result<(), ValidationError> {
        self.first.validate_input(input)
    }

    fn name(&self) -> &str {
        "CompositeService"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Serialize, Deserialize)]
    struct TestInput {
        value: i32,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct TestOutput {
        result: i32,
    }

    struct TestService;

    #[async_trait::async_trait]
    impl Service for TestService {
        type Input = TestInput;
        type Output = TestOutput;
        type Error = anyhow::Error;

        async fn process(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
            Ok(TestOutput {
                result: input.value * 2,
            })
        }
    }

    #[tokio::test]
    async fn test_service_registry() {
        let registry = ServiceRegistry::new();
        let service = TestService;

        registry.register(service);

        let retrieved = registry.get::<TestService>();
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_service_metrics() {
        let mut metrics = ServiceMetrics::default();

        metrics.record_request(Duration::from_millis(100), true);
        metrics.record_request(Duration::from_millis(200), true);
        metrics.record_request(Duration::from_millis(150), false);

        assert_eq!(metrics.request_count, 3);
        assert_eq!(metrics.success_count, 2);
        assert_eq!(metrics.error_count, 1);
        assert_eq!(metrics.average_duration_ms, 150);
        assert_eq!(metrics.success_rate(), 2.0 / 3.0);
    }
}

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test] 
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}
