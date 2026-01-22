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
    pub toyota_way_principles: Option<HashMap<String, serde_yaml::Value>>,

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
    pub zero_tolerance: Option<HashMap<String, serde_yaml::Value>>,

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
    pub fn from_yaml(yaml_str: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(yaml_str)
    }

    /// Render the prompt with variable substitution
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
    pub fn extract_variables(&self) -> Vec<String> {
        let re = regex::Regex::new(r"\$\{([^}]+)\}").expect("internal error");
        re.captures_iter(&self.prompt)
            .map(|cap| cap[1].to_string())
            .collect()
    }

    /// Serialize to YAML format
    pub fn to_yaml(&self) -> Result<String, serde_yaml::Error> {
        serde_yaml::to_string(self)
    }

    /// Serialize to JSON format
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Render to plain text format (just the prompt text)
    pub fn to_text(&self, variables: &HashMap<String, String>) -> String {
        self.render(variables)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create a basic prompt
    fn create_test_prompt() -> WorkflowPrompt {
        WorkflowPrompt {
            name: "test".to_string(),
            description: "Test prompt".to_string(),
            category: "testing".to_string(),
            priority: "high".to_string(),
            prompt: "Test ${VAR}".to_string(),
            methodology: None,
            constraints: None,
            heuristics: None,
            toyota_way_principles: None,
            quality_gates: None,
            validation_tools: None,
            testing_approaches: None,
            coverage_target: None,
            mutation_score_target: None,
            zero_tolerance: None,
            validation: None,
            mutation_targets: None,
            improvement_goals: None,
            refactoring_targets: None,
            optimization_targets: None,
            tools: None,
            vulnerability_tolerance: None,
            security_tools: None,
        }
    }

    #[test]
    fn test_load_prompt_from_yaml() {
        let yaml = r#"
name: test-prompt
description: Test prompt
category: testing
priority: high
prompt: |
  This is a test prompt with ${VAR1} and ${VAR2}.
methodology: EXTREME TDD
"#;

        let prompt = WorkflowPrompt::from_yaml(yaml).expect("internal error");
        assert_eq!(prompt.name, "test-prompt");
        assert_eq!(prompt.description, "Test prompt");
        assert_eq!(prompt.category, "testing");
        assert_eq!(prompt.priority, "high");
        assert!(prompt.prompt.contains("${VAR1}"));
        assert_eq!(prompt.methodology, Some("EXTREME TDD".to_string()));
    }

    #[test]
    fn test_load_prompt_with_all_fields() {
        let yaml = r#"
name: full-prompt
description: Full prompt test
category: quality
priority: critical
prompt: "Full ${TEST}"
methodology: Toyota Way
constraints:
  - "No flaky tests"
  - "All tests must pass"
heuristics:
  - "Start simple"
quality_gates:
  - "coverage > 90%"
validation_tools:
  - "cargo test"
  - "cargo clippy"
testing_approaches:
  - "unit"
  - "integration"
coverage_target: 95
mutation_score_target: 80
tools:
  - "llvm-cov"
vulnerability_tolerance: 0
security_tools:
  - "cargo-audit"
"#;

        let prompt = WorkflowPrompt::from_yaml(yaml).expect("internal error");
        assert_eq!(prompt.name, "full-prompt");
        assert_eq!(prompt.methodology, Some("Toyota Way".to_string()));
        assert_eq!(prompt.constraints.as_ref().unwrap().len(), 2);
        assert_eq!(prompt.heuristics.as_ref().unwrap().len(), 1);
        assert_eq!(prompt.quality_gates.as_ref().unwrap().len(), 1);
        assert_eq!(prompt.validation_tools.as_ref().unwrap().len(), 2);
        assert_eq!(prompt.testing_approaches.as_ref().unwrap().len(), 2);
        assert_eq!(prompt.coverage_target, Some(95));
        assert_eq!(prompt.mutation_score_target, Some(80));
        assert_eq!(prompt.tools.as_ref().unwrap().len(), 1);
        assert_eq!(prompt.vulnerability_tolerance, Some(0));
        assert_eq!(prompt.security_tools.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_variable_substitution() {
        let yaml = r#"
name: test
description: Test
category: test
priority: high
prompt: "Test ${VAR1} and ${VAR2}"
"#;

        let prompt = WorkflowPrompt::from_yaml(yaml).expect("internal error");
        let mut vars = HashMap::new();
        vars.insert("VAR1".to_string(), "value1".to_string());
        vars.insert("VAR2".to_string(), "value2".to_string());

        let rendered = prompt.render(&vars);
        assert_eq!(rendered, "Test value1 and value2");
    }

    #[test]
    fn test_variable_substitution_empty_vars() {
        let prompt = create_test_prompt();
        let vars = HashMap::new();

        let rendered = prompt.render(&vars);
        assert_eq!(rendered, "Test ${VAR}");
    }

    #[test]
    fn test_variable_substitution_partial() {
        let yaml = r#"
name: test
description: Test
category: test
priority: high
prompt: "Test ${VAR1} and ${VAR2}"
"#;

        let prompt = WorkflowPrompt::from_yaml(yaml).expect("internal error");
        let mut vars = HashMap::new();
        vars.insert("VAR1".to_string(), "value1".to_string());
        // VAR2 not provided

        let rendered = prompt.render(&vars);
        assert_eq!(rendered, "Test value1 and ${VAR2}");
    }

    #[test]
    fn test_variable_substitution_no_vars_in_prompt() {
        let yaml = r#"
name: test
description: Test
category: test
priority: high
prompt: "No variables here"
"#;

        let prompt = WorkflowPrompt::from_yaml(yaml).expect("internal error");
        let mut vars = HashMap::new();
        vars.insert("UNUSED".to_string(), "value".to_string());

        let rendered = prompt.render(&vars);
        assert_eq!(rendered, "No variables here");
    }

    #[test]
    fn test_extract_variables() {
        let yaml = r#"
name: test
description: Test
category: test
priority: high
prompt: "Use ${TEST_CMD} and ${COVERAGE_CMD} here"
"#;

        let prompt = WorkflowPrompt::from_yaml(yaml).expect("internal error");
        let vars = prompt.extract_variables();
        assert_eq!(vars.len(), 2);
        assert!(vars.contains(&"TEST_CMD".to_string()));
        assert!(vars.contains(&"COVERAGE_CMD".to_string()));
    }

    #[test]
    fn test_extract_variables_empty() {
        let yaml = r#"
name: test
description: Test
category: test
priority: high
prompt: "No variables"
"#;

        let prompt = WorkflowPrompt::from_yaml(yaml).expect("internal error");
        let vars = prompt.extract_variables();
        assert!(vars.is_empty());
    }

    #[test]
    fn test_extract_variables_duplicates() {
        let yaml = r#"
name: test
description: Test
category: test
priority: high
prompt: "Use ${VAR} and ${VAR} again"
"#;

        let prompt = WorkflowPrompt::from_yaml(yaml).expect("internal error");
        let vars = prompt.extract_variables();
        // Both occurrences are extracted
        assert_eq!(vars.len(), 2);
    }

    #[test]
    fn test_to_json() {
        let yaml = r#"
name: test
description: Test prompt
category: testing
priority: high
prompt: "Test prompt"
coverage_target: 85
"#;

        let prompt = WorkflowPrompt::from_yaml(yaml).expect("internal error");
        let json = prompt.to_json().expect("internal error");
        assert!(json.contains("\"name\": \"test\""));
        assert!(json.contains("\"coverage_target\": 85"));
    }

    #[test]
    fn test_to_json_skips_none() {
        let prompt = create_test_prompt();
        let json = prompt.to_json().expect("internal error");

        assert!(!json.contains("methodology"));
        assert!(!json.contains("constraints"));
        assert!(!json.contains("coverage_target"));
    }

    #[test]
    fn test_to_text() {
        let yaml = r#"
name: test
description: Test
category: test
priority: high
prompt: "Run ${CMD} now"
"#;

        let prompt = WorkflowPrompt::from_yaml(yaml).expect("internal error");
        let mut vars = HashMap::new();
        vars.insert("CMD".to_string(), "cargo test".to_string());

        let text = prompt.to_text(&vars);
        assert_eq!(text, "Run cargo test now");
    }

    #[test]
    fn test_to_yaml() {
        let prompt = create_test_prompt();
        let yaml = prompt.to_yaml().expect("internal error");

        assert!(yaml.contains("name: test"));
        assert!(yaml.contains("description: Test prompt"));
        assert!(yaml.contains("category: testing"));
    }

    #[test]
    fn test_to_yaml_skips_none() {
        let prompt = create_test_prompt();
        let yaml = prompt.to_yaml().expect("internal error");

        assert!(!yaml.contains("methodology"));
        assert!(!yaml.contains("constraints"));
        assert!(!yaml.contains("coverage_target"));
    }

    #[test]
    fn test_from_yaml_invalid() {
        let invalid_yaml = "this is: [not: valid";
        let result = WorkflowPrompt::from_yaml(invalid_yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_from_yaml_missing_required_field() {
        let yaml = r#"
name: test
description: Test
# missing category and priority
prompt: "Test"
"#;
        let result = WorkflowPrompt::from_yaml(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_prompt_clone() {
        let prompt = create_test_prompt();
        let cloned = prompt.clone();

        assert_eq!(cloned.name, prompt.name);
        assert_eq!(cloned.description, prompt.description);
        assert_eq!(cloned.prompt, prompt.prompt);
    }

    #[test]
    fn test_prompt_debug() {
        let prompt = create_test_prompt();
        let debug = format!("{:?}", prompt);

        assert!(debug.contains("WorkflowPrompt"));
        assert!(debug.contains("test"));
    }

    #[test]
    fn test_prompt_with_toyota_principles() {
        let yaml = r#"
name: test
description: Test
category: test
priority: high
prompt: "Test"
toyota_way_principles:
  jidoka: "Stop on defects"
  kaizen: "Continuous improvement"
"#;

        let prompt = WorkflowPrompt::from_yaml(yaml).expect("internal error");
        assert!(prompt.toyota_way_principles.is_some());
        let principles = prompt.toyota_way_principles.unwrap();
        assert_eq!(principles.len(), 2);
    }

    #[test]
    fn test_prompt_with_targets() {
        let yaml = r#"
name: test
description: Test
category: test
priority: high
prompt: "Test"
mutation_targets:
  coverage: "90%"
improvement_goals:
  speed: "10x faster"
refactoring_targets:
  complexity: "reduce by 50%"
optimization_targets:
  memory: "50% less"
"#;

        let prompt = WorkflowPrompt::from_yaml(yaml).expect("internal error");
        assert!(prompt.mutation_targets.is_some());
        assert!(prompt.improvement_goals.is_some());
        assert!(prompt.refactoring_targets.is_some());
        assert!(prompt.optimization_targets.is_some());
    }

    #[test]
    fn test_prompt_with_validation() {
        let yaml = r#"
name: test
description: Test
category: test
priority: high
prompt: "Test"
validation:
  - "check syntax"
  - "verify output"
"#;

        let prompt = WorkflowPrompt::from_yaml(yaml).expect("internal error");
        assert!(prompt.validation.is_some());
        assert_eq!(prompt.validation.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_prompt_with_zero_tolerance() {
        let yaml = r#"
name: test
description: Test
category: test
priority: high
prompt: "Test"
zero_tolerance:
  security_vulnerabilities: true
  test_failures: true
"#;

        let prompt = WorkflowPrompt::from_yaml(yaml).expect("internal error");
        assert!(prompt.zero_tolerance.is_some());
    }

    #[test]
    fn test_variable_substitution_special_chars() {
        let mut prompt = create_test_prompt();
        prompt.prompt = "Test ${VAR} with special".to_string();

        let mut vars = HashMap::new();
        vars.insert(
            "VAR".to_string(),
            "value with $pecial & <chars>".to_string(),
        );

        let rendered = prompt.render(&vars);
        assert!(rendered.contains("value with $pecial & <chars>"));
    }

    #[test]
    fn test_multiline_prompt() {
        let yaml = r#"
name: test
description: Test
category: test
priority: high
prompt: |
  Line 1 with ${VAR1}
  Line 2 with ${VAR2}
  Line 3
"#;

        let prompt = WorkflowPrompt::from_yaml(yaml).expect("internal error");
        let mut vars = HashMap::new();
        vars.insert("VAR1".to_string(), "value1".to_string());
        vars.insert("VAR2".to_string(), "value2".to_string());

        let rendered = prompt.render(&vars);
        assert!(rendered.contains("Line 1 with value1"));
        assert!(rendered.contains("Line 2 with value2"));
        assert!(rendered.contains("Line 3"));
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_render_idempotent(var_value in "[a-zA-Z0-9_-]+") {
            let yaml = r#"
name: test
description: Test
category: test
priority: high
prompt: "Test ${VAR}"
"#;
            let prompt = WorkflowPrompt::from_yaml(yaml).expect("internal error");
            let mut vars = HashMap::new();
            vars.insert("VAR".to_string(), var_value);

            let rendered1 = prompt.render(&vars);
            let rendered2 = prompt.render(&vars);
            prop_assert_eq!(rendered1, rendered2);
        }

        #[test]
        fn test_to_yaml_roundtrip(name in "[a-z-]+", desc in ".*") {
            let prompt = WorkflowPrompt {
                name: name.clone(),
                description: desc.clone(),
                category: "test".to_string(),
                priority: "high".to_string(),
                prompt: "test".to_string(),
                methodology: None,
                constraints: None,
                heuristics: None,
                toyota_way_principles: None,
                quality_gates: None,
                validation_tools: None,
                testing_approaches: None,
                coverage_target: None,
                mutation_score_target: None,
                zero_tolerance: None,
                validation: None,
                mutation_targets: None,
                improvement_goals: None,
                refactoring_targets: None,
                optimization_targets: None,
                tools: None,
                vulnerability_tolerance: None,
                security_tools: None,
            };

            let yaml = prompt.to_yaml().expect("internal error");
            let parsed = WorkflowPrompt::from_yaml(&yaml).expect("internal error");
            prop_assert_eq!(parsed.name, name);
            prop_assert_eq!(parsed.description, desc);
        }
    }
}
