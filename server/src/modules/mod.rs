// Modular monolith with compiler-enforced boundaries
// Each module has a public trait interface and private implementation

pub mod analyzer;
pub mod orchestrator;
pub mod transformer;
pub mod validator;

use async_trait::async_trait;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ModuleError {
    #[error("Module not found: {0}")]
    NotFound(String),
    #[error("Module initialization failed: {0}")]
    InitFailed(String),
    #[error("Module execution failed: {0}")]
    ExecutionFailed(String),
}

// Base trait that all modules must implement
#[async_trait]
pub trait PmatModule: Send + Sync + 'static {
    type Input: Send + Sync;
    type Output: Send + Sync;

    fn name(&self) -> &'static str;

    async fn initialize(&mut self) -> Result<(), ModuleError>;

    async fn process(&self, input: Self::Input) -> Result<Self::Output, ModuleError>;

    async fn shutdown(&mut self) -> Result<(), ModuleError>;
}

// Module registry for dependency injection
pub struct ModuleRegistry {
    modules: dashmap::DashMap<String, Arc<dyn std::any::Any + Send + Sync>>,
}

impl ModuleRegistry {
    pub fn new() -> Self {
        Self {
            modules: dashmap::DashMap::new(),
        }
    }

    pub fn register<T>(&self, name: String, module: Arc<T>)
    where
        T: std::any::Any + Send + Sync + 'static,
    {
        self.modules
            .insert(name, module as Arc<dyn std::any::Any + Send + Sync>);
    }

    pub fn get<T>(&self, name: &str) -> Option<Arc<T>>
    where
        T: std::any::Any + Send + Sync + 'static,
    {
        self.modules
            .get(name)
            .and_then(|module| module.clone().downcast::<T>().ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_registry() {
        let registry = ModuleRegistry::new();

        #[derive(Debug)]
        struct TestModule {
            value: i32,
        }

        let module = Arc::new(TestModule { value: 42 });
        registry.register("test".to_string(), module.clone());

        let retrieved = registry.get::<TestModule>("test");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().value, 42);
    }
}
