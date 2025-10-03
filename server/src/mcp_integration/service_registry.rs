//! Service Registry for MCP Integration
//!
//! Provides a registry for services that agents can use to perform operations.
//! Services are different from tools - they are internal capabilities that agents
//! can invoke, while tools are exposed to external MCP clients.

use super::*;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

/// Registry for managing services that agents can use
pub struct ServiceRegistry {
    services: Arc<RwLock<HashMap<String, Arc<dyn Service>>>>,
    metadata: Arc<RwLock<HashMap<String, ServiceMetadata>>>,
}

/// Metadata about a registered service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetadata {
    pub name: String,
    pub description: String,
    pub version: String,
    pub capabilities: Vec<String>,
}

/// Trait for services that can be registered
#[async_trait]
pub trait Service: Send + Sync {
    /// Get service metadata
    fn metadata(&self) -> ServiceMetadata;

    /// Check if service is healthy
    async fn health_check(&self) -> Result<bool, McpError>;

    /// Invoke a service operation
    async fn invoke(&self, operation: &str, params: Value) -> Result<Value, McpError>;
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            metadata: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn register(&self, service: Arc<dyn Service>) {
        let metadata = service.metadata();
        self.services.write().insert(metadata.name.clone(), service);
        self.metadata.write().insert(metadata.name.clone(), metadata);
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn Service>> {
        self.services.read().get(name).cloned()
    }

    pub fn list(&self) -> Vec<ServiceMetadata> {
        self.metadata.read().values().cloned().collect()
    }

    pub fn unregister(&self, name: &str) -> bool {
        let service_removed = self.services.write().remove(name).is_some();
        let metadata_removed = self.metadata.write().remove(name).is_some();
        service_removed && metadata_removed
    }

    pub async fn health_check_all(&self) -> HashMap<String, bool> {
        let services = self.services.read();
        let mut results = HashMap::new();

        for (name, service) in services.iter() {
            let health = service.health_check().await.unwrap_or(false);
            results.insert(name.clone(), health);
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockService {
        name: String,
        healthy: bool,
    }

    #[async_trait]
    impl Service for MockService {
        fn metadata(&self) -> ServiceMetadata {
            ServiceMetadata {
                name: self.name.clone(),
                description: "Mock service for testing".to_string(),
                version: "1.0.0".to_string(),
                capabilities: vec!["test".to_string()],
            }
        }

        async fn health_check(&self) -> Result<bool, McpError> {
            Ok(self.healthy)
        }

        async fn invoke(&self, _operation: &str, _params: Value) -> Result<Value, McpError> {
            Ok(serde_json::json!({"result": "success"}))
        }
    }

    #[test]
    fn test_service_registry_creation() {
        let registry = ServiceRegistry::new();
        assert_eq!(registry.list().len(), 0);
    }

    #[test]
    fn test_service_registration() {
        let registry = ServiceRegistry::new();
        let service = Arc::new(MockService {
            name: "test_service".to_string(),
            healthy: true,
        });

        registry.register(service);
        assert_eq!(registry.list().len(), 1);
        assert!(registry.get("test_service").is_some());
    }

    #[test]
    fn test_service_unregistration() {
        let registry = ServiceRegistry::new();
        let service = Arc::new(MockService {
            name: "test_service".to_string(),
            healthy: true,
        });

        registry.register(service);
        assert!(registry.get("test_service").is_some());

        let removed = registry.unregister("test_service");
        assert!(removed);
        assert!(registry.get("test_service").is_none());
    }

    #[tokio::test]
    async fn test_health_check_all() {
        let registry = ServiceRegistry::new();

        let service1 = Arc::new(MockService {
            name: "healthy_service".to_string(),
            healthy: true,
        });

        let service2 = Arc::new(MockService {
            name: "unhealthy_service".to_string(),
            healthy: false,
        });

        registry.register(service1);
        registry.register(service2);

        let health_status = registry.health_check_all().await;
        assert_eq!(health_status.get("healthy_service"), Some(&true));
        assert_eq!(health_status.get("unhealthy_service"), Some(&false));
    }
}
