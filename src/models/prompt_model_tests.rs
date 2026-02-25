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
