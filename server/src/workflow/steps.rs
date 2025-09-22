use super::*;

// Step implementations
pub struct StepRegistry {
    steps: std::collections::HashMap<String, Box<dyn StepHandler>>,
}

pub trait StepHandler: Send + Sync {
    fn execute(&self, params: &Value, context: &WorkflowContext) -> Result<Value, WorkflowError>;
}