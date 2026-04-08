//! Service Registry for MCP Integration
//!
//! Provides a registry for services that agents can use to perform operations.
//! Services are different from tools - they are internal capabilities that agents
//! can invoke, while tools are exposed to external MCP clients.

use super::*;
use async_trait::async_trait;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            metadata: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Register a new item.
    pub fn register(&self, service: Arc<dyn Service>) {
        let metadata = service.metadata();
        self.services.write().insert(metadata.name.clone(), service);
        self.metadata
            .write()
            .insert(metadata.name.clone(), metadata);
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Retrieve a value.
    pub fn get(&self, name: &str) -> Option<Arc<dyn Service>> {
        self.services.read().get(name).cloned()
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// List.
    pub fn list(&self) -> Vec<ServiceMetadata> {
        self.metadata.read().values().cloned().collect()
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Unregister an item.
    pub fn unregister(&self, name: &str) -> bool {
        let service_removed = self.services.write().remove(name).is_some();
        let metadata_removed = self.metadata.write().remove(name).is_some();
        service_removed && metadata_removed
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn health_check_all(&self) -> HashMap<String, bool> {
        // Collect services while holding the lock, then drop it before async work
        let services_snapshot: Vec<_> = {
            let services = self.services.read();
            services
                .iter()
                .map(|(name, service)| (name.clone(), service.clone()))
                .collect()
        };
        // Lock is dropped here

        let mut results = HashMap::new();
        for (name, service) in services_snapshot {
            let health = service.health_check().await.unwrap_or(false);
            results.insert(name, health);
        }

        results
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
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

    struct FailingHealthService;

    #[async_trait]
    impl Service for FailingHealthService {
        fn metadata(&self) -> ServiceMetadata {
            ServiceMetadata {
                name: "failing_health".to_string(),
                description: "Service with failing health check".to_string(),
                version: "1.0.0".to_string(),
                capabilities: vec![],
            }
        }

        async fn health_check(&self) -> Result<bool, McpError> {
            Err(McpError {
                code: -1,
                message: "Health check failed".to_string(),
                data: None,
            })
        }

        async fn invoke(&self, _operation: &str, _params: Value) -> Result<Value, McpError> {
            Ok(serde_json::json!({}))
        }
    }

    #[test]
    fn test_service_registry_creation() {
        let registry = ServiceRegistry::new();
        assert_eq!(registry.list().len(), 0);
    }

    #[test]
    fn test_service_registry_default() {
        let registry = ServiceRegistry::default();
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

    #[test]
    fn test_service_unregistration_nonexistent() {
        let registry = ServiceRegistry::new();
        let removed = registry.unregister("nonexistent");
        assert!(!removed);
    }

    #[test]
    fn test_get_nonexistent_service() {
        let registry = ServiceRegistry::new();
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_list_empty() {
        let registry = ServiceRegistry::new();
        let services = registry.list();
        assert!(services.is_empty());
    }

    #[test]
    fn test_list_multiple_services() {
        let registry = ServiceRegistry::new();

        for i in 0..5 {
            let service = Arc::new(MockService {
                name: format!("service_{}", i),
                healthy: true,
            });
            registry.register(service);
        }

        let services = registry.list();
        assert_eq!(services.len(), 5);
    }

    #[test]
    fn test_service_metadata() {
        let service = MockService {
            name: "meta_test".to_string(),
            healthy: true,
        };

        let meta = service.metadata();
        assert_eq!(meta.name, "meta_test");
        assert_eq!(meta.description, "Mock service for testing");
        assert_eq!(meta.version, "1.0.0");
        assert_eq!(meta.capabilities, vec!["test".to_string()]);
    }

    #[test]
    fn test_service_metadata_clone() {
        let meta = ServiceMetadata {
            name: "clone_test".to_string(),
            description: "Testing clone".to_string(),
            version: "2.0.0".to_string(),
            capabilities: vec!["a".to_string(), "b".to_string()],
        };

        let cloned = meta.clone();
        assert_eq!(meta.name, cloned.name);
        assert_eq!(meta.version, cloned.version);
    }

    #[test]
    fn test_service_metadata_debug() {
        let meta = ServiceMetadata {
            name: "debug_test".to_string(),
            description: "Testing debug".to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec![],
        };

        let debug_str = format!("{:?}", meta);
        assert!(debug_str.contains("debug_test"));
        assert!(debug_str.contains("ServiceMetadata"));
    }

    #[test]
    fn test_service_metadata_serialization() {
        let meta = ServiceMetadata {
            name: "serialize_test".to_string(),
            description: "Testing serialization".to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec!["cap1".to_string()],
        };

        let json = serde_json::to_string(&meta).unwrap();
        assert!(json.contains("serialize_test"));

        let deserialized: ServiceMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, meta.name);
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

    #[tokio::test]
    async fn test_health_check_all_empty() {
        let registry = ServiceRegistry::new();
        let health_status = registry.health_check_all().await;
        assert!(health_status.is_empty());
    }

    #[tokio::test]
    async fn test_health_check_all_with_failing_service() {
        let registry = ServiceRegistry::new();

        let failing_service = Arc::new(FailingHealthService);
        registry.register(failing_service);

        let health_status = registry.health_check_all().await;
        // Failing health check returns false (not error)
        assert_eq!(health_status.get("failing_health"), Some(&false));
    }

    #[tokio::test]
    async fn test_service_invoke() {
        let service = MockService {
            name: "invoke_test".to_string(),
            healthy: true,
        };

        let result = service.invoke("operation", serde_json::json!({})).await;

        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value["result"], "success");
    }

    #[tokio::test]
    async fn test_service_health_check() {
        let healthy = MockService {
            name: "healthy".to_string(),
            healthy: true,
        };

        let unhealthy = MockService {
            name: "unhealthy".to_string(),
            healthy: false,
        };

        assert!(healthy.health_check().await.unwrap());
        assert!(!unhealthy.health_check().await.unwrap());
    }

    #[test]
    fn test_register_overwrite() {
        let registry = ServiceRegistry::new();

        let service1 = Arc::new(MockService {
            name: "same_name".to_string(),
            healthy: true,
        });

        let service2 = Arc::new(MockService {
            name: "same_name".to_string(),
            healthy: false,
        });

        registry.register(service1);
        registry.register(service2);

        // Should still have one service (overwritten)
        assert_eq!(registry.list().len(), 1);
    }
}
