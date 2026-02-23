#![cfg_attr(coverage_nightly, coverage(off))]
use super::*;

// DSL compiler
pub struct DslCompiler;

impl DslCompiler {
    pub fn compile(source: &str) -> Result<Workflow, WorkflowError> {
        // For now, use YAML/JSON parsing
        serde_yaml_ng::from_str(source)
            .or_else(|_| serde_json::from_str(source))
            .map_err(|e| WorkflowError::InvalidDefinition(e.to_string()))
    }

    pub fn compile_step(source: &str) -> Result<WorkflowStep, WorkflowError> {
        serde_yaml_ng::from_str(source)
            .or_else(|_| serde_json::from_str(source))
            .map_err(|e| WorkflowError::InvalidDefinition(e.to_string()))
    }
}
