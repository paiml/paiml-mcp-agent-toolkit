//! Service composition pattern implementation per SPECIFICATION.md Section 2.2
//! 
//! This module provides service composition capabilities allowing complex
//! operations to be built from simpler services.

use super::service_base::{Service, ServiceMetrics, ValidationError};
use anyhow::Result;
use std::sync::Arc;
use std::time::Instant;

/// Simple composite service for compatible types
pub struct SimpleCompositeService<A, B> 
where
    A: Service,
    B: Service<Input = A::Output>,
{
    first: Arc<A>,
    second: Arc<B>,
    metrics: ServiceMetrics,
}

impl<A, B> SimpleCompositeService<A, B>
where
    A: Service,
    B: Service<Input = A::Output>,
{
    /// Create a new simple composite service
    pub fn new(first: A, second: B) -> Self {
        Self {
            first: Arc::new(first),
            second: Arc::new(second),
            metrics: ServiceMetrics::default(),
        }
    }
}

#[async_trait::async_trait]
impl<A, B> Service for SimpleCompositeService<A, B>
where
    A: Service + Send + Sync,
    B: Service<Input = A::Output> + Send + Sync,
    A::Input: Send + Sync + 'static,
    A::Output: Send + Sync + 'static,
    B::Output: Send + Sync + 'static,
    A::Error: Into<anyhow::Error> + Send + Sync + 'static,
    B::Error: Into<anyhow::Error> + Send + Sync + 'static,
{
    type Input = A::Input;
    type Output = B::Output;
    type Error = anyhow::Error;
    
    async fn process(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        let start = Instant::now();
        
        // Process through first service
        let intermediate = self.first
            .process(input)
            .await
            .map_err(Into::into)?;
        
        // Process through second service
        let output = self.second
            .process(intermediate)
            .await
            .map_err(Into::into)?;
        
        // Record metrics
        let mut metrics = self.metrics.clone();
        metrics.record_request(start.elapsed(), true);
        
        Ok(output)
    }
    
    fn validate_input(&self, input: &Self::Input) -> Result<(), ValidationError> {
        self.first.validate_input(input)
    }
    
    fn metrics(&self) -> ServiceMetrics {
        self.metrics.clone()
    }
    
    fn name(&self) -> &str {
        "SimpleCompositeService"
    }
}

/// Service composition builder for complex workflows
pub struct ServiceComposer {
    registry: Arc<super::service_base::ServiceRegistry>,
}

impl ServiceComposer {
    /// Create a new service composer
    pub fn new(registry: Arc<super::service_base::ServiceRegistry>) -> Self {
        Self { registry }
    }
    
    /// Compose two services sequentially
    pub fn compose<A, B>(&self, first: A, second: B) -> SimpleCompositeService<A, B>
    where
        A: Service,
        B: Service<Input = A::Output>,
    {
        SimpleCompositeService::new(first, second)
    }
    
    /// Get the service registry
    pub fn registry(&self) -> &super::service_base::ServiceRegistry {
        &self.registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Mock service for testing
    #[derive(Clone)]
    struct AddService {
        value: i32,
    }
    
    #[async_trait::async_trait]
    impl Service for AddService {
        type Input = i32;
        type Output = i32;
        type Error = anyhow::Error;
        
        async fn process(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
            Ok(input + self.value)
        }
    }
    
    #[derive(Clone)]
    struct MultiplyService {
        factor: i32,
    }
    
    #[async_trait::async_trait]
    impl Service for MultiplyService {
        type Input = i32;
        type Output = i32;
        type Error = anyhow::Error;
        
        async fn process(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
            Ok(input * self.factor)
        }
    }
    
    #[tokio::test]
    async fn test_simple_composite_service() {
        let add = AddService { value: 5 };
        let multiply = MultiplyService { factor: 2 };
        
        let composite = SimpleCompositeService::new(add, multiply);
        let result = composite.process(10).await.unwrap();
        
        // (10 + 5) * 2 = 30
        assert_eq!(result, 30);
    }
    
    #[tokio::test]
    async fn test_service_composer() {
        let registry = Arc::new(super::super::service_base::ServiceRegistry::new());
        let composer = ServiceComposer::new(registry);
        
        let add = AddService { value: 7 };
        let multiply = MultiplyService { factor: 4 };
        
        let composed = composer.compose(add, multiply);
        let result = composed.process(3).await.unwrap();
        
        // (3 + 7) * 4 = 40
        assert_eq!(result, 40);
    }
}