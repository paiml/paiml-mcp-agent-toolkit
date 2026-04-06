#![cfg_attr(coverage_nightly, coverage(off))]
//! Prompt data model for CLI workflow prompts
//!
//! This module defines the structure for storing and rendering workflow prompts
//! that enforce EXTREME TDD and Toyota Way quality principles.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Workflow prompt that can be loaded from YAML and rendered with variable substitution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowPrompt {
    /// Unique name identifier
    pub name: String,

    /// Human-readable description
    pub description: String,

    /// Category (quality, maintenance, workflow, etc.)
    pub category: String,

    /// Priority level (critical, high, medium, low)
    pub priority: String,

    /// The main prompt text (supports variable substitution)
    pub prompt: String,

    /// Optional methodology (e.g., "EXTREME TDD", "Five Whys")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub methodology: Option<String>,

    /// Optional constraints (e.g., time limits, resource bounds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraints: Option<Vec<String>>,

    /// Optional heuristics for decision making
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heuristics: Option<Vec<String>>,

    /// Optional Toyota Way principles
    #[serde(skip_serializing_if = "Option::is_none")]
    pub toyota_way_principles: Option<HashMap<String, serde_yaml_ng::Value>>,

    /// Optional quality gates
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality_gates: Option<Vec<String>>,

    /// Optional validation tools/commands
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_tools: Option<Vec<String>>,

    /// Optional testing approaches
    #[serde(skip_serializing_if = "Option::is_none")]
    pub testing_approaches: Option<Vec<String>>,

    /// Optional coverage target percentage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coverage_target: Option<u8>,

    /// Optional mutation score target percentage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mutation_score_target: Option<u8>,

    /// Optional zero tolerance settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zero_tolerance: Option<HashMap<String, serde_yaml_ng::Value>>,

    /// Optional validation rules
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation: Option<Vec<String>>,

    /// Optional mutation targets
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mutation_targets: Option<HashMap<String, String>>,

    /// Optional improvement goals
    #[serde(skip_serializing_if = "Option::is_none")]
    pub improvement_goals: Option<HashMap<String, String>>,

    /// Optional refactoring targets
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refactoring_targets: Option<HashMap<String, String>>,

    /// Optional optimization targets
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optimization_targets: Option<HashMap<String, String>>,

    /// Optional tools list
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<String>>,

    /// Optional vulnerability tolerance
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vulnerability_tolerance: Option<u8>,

    /// Optional security tools
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_tools: Option<Vec<String>>,
}

impl WorkflowPrompt {
    /// Load a prompt from YAML string
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn from_yaml(yaml_str: &str) -> Result<Self, serde_yaml_ng::Error> {
        debug_assert!(!yaml_str.is_empty(), "yaml_str must not be empty");
        serde_yaml_ng::from_str(yaml_str)
    }

    /// Render the prompt with variable substitution
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn render(&self, variables: &HashMap<String, String>) -> String {
        let mut rendered = self.prompt.clone();

        // Simple ${VAR} substitution
        for (key, value) in variables {
            let placeholder = format!("${{{key}}}");
            rendered = rendered.replace(&placeholder, value);
        }

        rendered
    }

    /// Extract variable names from the prompt text
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn extract_variables(&self) -> Vec<String> {
        let re = regex::Regex::new(r"\$\{([^}]+)\}").expect("internal error");
        re.captures_iter(&self.prompt)
            .map(|cap| cap[1].to_string())
            .collect()
    }

    /// Serialize to YAML format
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn to_yaml(&self) -> Result<String, serde_yaml_ng::Error> {
        serde_yaml_ng::to_string(self)
    }

    /// Serialize to JSON format
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Render to plain text format (just the prompt text)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn to_text(&self, variables: &HashMap<String, String>) -> String {
        self.render(variables)
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    include!("prompt_model_tests.rs");
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    include!("prompt_model_property_tests.rs");
}
