#![cfg_attr(coverage_nightly, coverage(off))]

#[cfg(test)]
mod coverage_tests_compiler {
    use crate::workflow::dsl::*;
    use crate::workflow::*;

    // =========================================================================
    // DslCompiler::compile tests
    // =========================================================================

    #[test]
    fn test_dsl_compiler_compile_json_success() {
        // Test compilation from valid JSON
        let json_workflow = r#"{
            "id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
            "name": "json_workflow",
            "description": "A test workflow",
            "version": "1.0.0",
            "steps": [],
            "error_strategy": "FailFast",
            "metadata": {}
        }"#;

        let result = DslCompiler::compile(json_workflow);
        assert!(result.is_ok());
        let workflow = result.unwrap();
        assert_eq!(workflow.name, "json_workflow");
        assert_eq!(workflow.version, "1.0.0");
        assert!(workflow.steps.is_empty());
    }

    #[test]
    fn test_dsl_compiler_compile_yaml_success() {
        // Test compilation from valid YAML
        let yaml_workflow = r#"
id: f47ac10b-58cc-4372-a567-0e02b2c3d479
name: yaml_workflow
description: A YAML test workflow
version: "2.0.0"
steps: []
error_strategy: Continue
metadata: {}
"#;

        let result = DslCompiler::compile(yaml_workflow);
        assert!(result.is_ok());
        let workflow = result.unwrap();
        assert_eq!(workflow.name, "yaml_workflow");
        assert_eq!(workflow.version, "2.0.0");
    }

    #[test]
    fn test_dsl_compiler_compile_invalid_input() {
        // Test compilation with invalid input that fails both YAML and JSON parsing
        let invalid_input = "this is not valid { yaml or json }}}";

        let result = DslCompiler::compile(invalid_input);
        assert!(result.is_err());
        match result {
            Err(WorkflowError::InvalidDefinition(msg)) => {
                assert!(!msg.is_empty());
            }
            _ => panic!("Expected InvalidDefinition error"),
        }
    }

    #[test]
    fn test_dsl_compiler_compile_empty_input() {
        // Test compilation with empty input
        let result = DslCompiler::compile("");
        assert!(result.is_err());
    }

    #[test]
    fn test_dsl_compiler_compile_partial_json() {
        // Test compilation with JSON missing required fields
        let partial_json = r#"{"name": "incomplete"}"#;
        let result = DslCompiler::compile(partial_json);
        assert!(result.is_err());
    }

    // =========================================================================
    // DslCompiler::compile_step tests
    // =========================================================================

    #[test]
    fn test_dsl_compiler_compile_step_json_success() {
        let json_step = r#"{
            "id": "step1",
            "name": "Test Step",
            "step_type": {
                "type": "action",
                "agent": "test_agent",
                "operation": "test_op",
                "params": {}
            },
            "metadata": {}
        }"#;

        let result = DslCompiler::compile_step(json_step);
        assert!(result.is_ok());
        let step = result.unwrap();
        assert_eq!(step.id, "step1");
        assert_eq!(step.name, "Test Step");
    }

    #[test]
    fn test_dsl_compiler_compile_step_yaml_success() {
        let yaml_step = r#"
id: step2
name: YAML Step
step_type:
  type: wait
  duration:
    secs: 10
    nanos: 0
metadata: {}
"#;

        let result = DslCompiler::compile_step(yaml_step);
        assert!(result.is_ok());
        let step = result.unwrap();
        assert_eq!(step.id, "step2");
        assert_eq!(step.name, "YAML Step");
    }

    #[test]
    fn test_dsl_compiler_compile_step_invalid_input() {
        let invalid_input = "not a valid step {{{";
        let result = DslCompiler::compile_step(invalid_input);
        assert!(result.is_err());
        match result {
            Err(WorkflowError::InvalidDefinition(_)) => {}
            _ => panic!("Expected InvalidDefinition error"),
        }
    }

    #[test]
    fn test_dsl_compiler_compile_step_empty_input() {
        let result = DslCompiler::compile_step("");
        assert!(result.is_err());
    }
}
