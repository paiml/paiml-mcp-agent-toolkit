#![cfg_attr(coverage_nightly, coverage(off))]
use super::*;

// DSL compiler
pub struct DslCompiler;

impl DslCompiler {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn compile(source: &str) -> Result<Workflow, WorkflowError> {
        debug_assert!(!source.is_empty(), "source must not be empty");
        // For now, use YAML/JSON parsing
        serde_yaml_ng::from_str(source)
            .or_else(|_| serde_json::from_str(source))
            .map_err(|e| WorkflowError::InvalidDefinition(e.to_string()))
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn compile_step(source: &str) -> Result<WorkflowStep, WorkflowError> {
        debug_assert!(!source.is_empty(), "source must not be empty");
        serde_yaml_ng::from_str(source)
            .or_else(|_| serde_json::from_str(source))
            .map_err(|e| WorkflowError::InvalidDefinition(e.to_string()))
    }
}
