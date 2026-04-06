#![cfg_attr(coverage_nightly, coverage(off))]
use super::*;

// Step implementations
pub struct StepRegistry {
    _steps: std::collections::HashMap<String, Box<dyn StepHandler>>,
}

impl Default for StepRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl StepRegistry {
    pub fn new() -> Self {
        Self {
            _steps: std::collections::HashMap::new(),
        }
    }

    pub fn register(&mut self, name: impl Into<String>, handler: Box<dyn StepHandler>) {
        self._steps.insert(name.into(), handler);
    }

    pub fn get(&self, name: &str) -> Option<&dyn StepHandler> {
        debug_assert!(!name.is_empty(), "name must not be empty");
        self._steps.get(name).map(|b| b.as_ref())
    }

    pub fn remove(&mut self, name: &str) -> Option<Box<dyn StepHandler>> {
        debug_assert!(!name.is_empty(), "name must not be empty");
        self._steps.remove(name)
    }

    pub fn list(&self) -> Vec<&String> {
        self._steps.keys().collect()
    }

    pub fn is_empty(&self) -> bool {
        self._steps.is_empty()
    }

    pub fn len(&self) -> usize {
        self._steps.len()
    }
}

pub trait StepHandler: Send + Sync {
    fn execute(&self, params: &Value, context: &WorkflowContext) -> Result<Value, WorkflowError>;
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    struct MockStepHandler {
        name: String,
    }

    impl StepHandler for MockStepHandler {
        fn execute(
            &self,
            _params: &Value,
            _context: &WorkflowContext,
        ) -> Result<Value, WorkflowError> {
            Ok(serde_json::json!({"handler": self.name}))
        }
    }

    #[test]
    fn test_step_registry_new() {
        let registry = StepRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_step_registry_default() {
        let registry = StepRegistry::default();
        assert!(registry.is_empty());
    }

    #[test]
    fn test_step_registry_register() {
        let mut registry = StepRegistry::new();
        let handler = Box::new(MockStepHandler {
            name: "test_handler".to_string(),
        });

        registry.register("test_step", handler);

        assert!(!registry.is_empty());
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_step_registry_get() {
        let mut registry = StepRegistry::new();
        let handler = Box::new(MockStepHandler {
            name: "my_handler".to_string(),
        });

        registry.register("my_step", handler);

        assert!(registry.get("my_step").is_some());
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_step_registry_remove() {
        let mut registry = StepRegistry::new();
        let handler = Box::new(MockStepHandler {
            name: "removable".to_string(),
        });

        registry.register("step_to_remove", handler);
        assert_eq!(registry.len(), 1);

        let removed = registry.remove("step_to_remove");
        assert!(removed.is_some());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_step_registry_remove_nonexistent() {
        let mut registry = StepRegistry::new();
        let removed = registry.remove("nonexistent");
        assert!(removed.is_none());
    }

    #[test]
    fn test_step_registry_list() {
        let mut registry = StepRegistry::new();
        registry.register(
            "step1",
            Box::new(MockStepHandler {
                name: "h1".to_string(),
            }),
        );
        registry.register(
            "step2",
            Box::new(MockStepHandler {
                name: "h2".to_string(),
            }),
        );

        let list = registry.list();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_step_registry_multiple_operations() {
        let mut registry = StepRegistry::new();

        // Register multiple handlers
        for i in 0..5 {
            registry.register(
                format!("step_{}", i),
                Box::new(MockStepHandler {
                    name: format!("handler_{}", i),
                }),
            );
        }
        assert_eq!(registry.len(), 5);

        // Remove some
        registry.remove("step_0");
        registry.remove("step_2");
        assert_eq!(registry.len(), 3);

        // Check remaining
        assert!(registry.get("step_1").is_some());
        assert!(registry.get("step_3").is_some());
        assert!(registry.get("step_4").is_some());
        assert!(registry.get("step_0").is_none());
    }

    #[test]
    fn test_mock_step_handler_execute() {
        let handler = MockStepHandler {
            name: "test".to_string(),
        };

        let registry = Arc::new(crate::agents::registry::AgentRegistry::new());
        let context = WorkflowContext::new(uuid::Uuid::new_v4(), registry);
        let params = serde_json::json!({});

        let result = handler.execute(&params, &context);
        assert!(result.is_ok());

        let value = result.unwrap();
        assert_eq!(value["handler"], "test");
    }

    #[test]
    fn test_step_handler_trait_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<MockStepHandler>();
    }

    #[test]
    fn test_step_registry_overwrite() {
        let mut registry = StepRegistry::new();

        registry.register(
            "step",
            Box::new(MockStepHandler {
                name: "first".to_string(),
            }),
        );
        registry.register(
            "step",
            Box::new(MockStepHandler {
                name: "second".to_string(),
            }),
        );

        // Should still have only one entry (overwritten)
        assert_eq!(registry.len(), 1);
    }
}
