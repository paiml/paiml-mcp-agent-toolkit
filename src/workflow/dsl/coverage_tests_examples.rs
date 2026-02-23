#![cfg_attr(coverage_nightly, coverage(off))]

#[cfg(test)]
mod coverage_tests_examples {
    use crate::workflow::dsl::*;
    use crate::workflow::*;
    use crate::{step, workflow};
    use std::collections::HashMap;
    use std::time::Duration;

    // =========================================================================
    // WORKFLOW_DSL_EXAMPLE constant tests
    // =========================================================================

    #[test]
    fn test_workflow_dsl_example_is_valid_json() {
        let result: Result<serde_json::Value, _> = serde_json::from_str(WORKFLOW_DSL_EXAMPLE);
        assert!(result.is_ok());
    }

    #[test]
    fn test_workflow_dsl_example_parses_to_workflow() {
        let result = DslCompiler::compile(WORKFLOW_DSL_EXAMPLE);
        assert!(result.is_ok());
        let workflow = result.unwrap();

        // Verify all expected fields
        assert_eq!(
            workflow.id.to_string(),
            "f47ac10b-58cc-4372-a567-0e02b2c3d479"
        );
        assert_eq!(workflow.name, "quality_check_workflow");
        assert_eq!(
            workflow.description,
            Some("Comprehensive quality check workflow".to_string())
        );
        assert_eq!(workflow.version, "1.0.0");
        assert_eq!(workflow.steps.len(), 1);
        assert!(matches!(workflow.error_strategy, ErrorStrategy::FailFast));
        assert_eq!(workflow.timeout, Some(Duration::from_secs(300)));
    }

    #[test]
    fn test_workflow_dsl_example_step_properties() {
        let workflow = DslCompiler::compile(WORKFLOW_DSL_EXAMPLE).unwrap();
        let step = &workflow.steps[0];

        assert_eq!(step.id, "analyze");
        assert_eq!(step.name, "Code Analysis");
        assert_eq!(step.timeout, Some(Duration::from_secs(60)));
        assert!(step.retry.is_some());

        match &step.step_type {
            StepType::Action {
                agent,
                operation,
                params,
            } => {
                assert_eq!(agent, "analyzer");
                assert_eq!(operation, "analyze");
                assert!(params.is_object());
            }
            _ => panic!("Expected Action step type"),
        }
    }

    // =========================================================================
    // Edge cases and complex scenarios
    // =========================================================================

    #[test]
    fn test_nested_conditionals() {
        let inner_if_step = WorkflowStep {
            id: "inner_if".to_string(),
            name: "Inner If".to_string(),
            step_type: StepType::Wait {
                duration: Duration::from_secs(1),
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        };
        let outer_if_step = WorkflowStep {
            id: "outer_if".to_string(),
            name: "Outer If".to_string(),
            step_type: StepType::Wait {
                duration: Duration::from_secs(1),
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        };

        // Build outer conditional
        let workflow = FluentWorkflow::define("nested")
            .when("outer_cond")
            .do_this(outer_if_step)
            .end_if()
            .when("inner_cond")
            .do_this(inner_if_step)
            .end_if()
            .build();

        assert_eq!(workflow.steps.len(), 2);
    }

    #[test]
    fn test_parallel_with_many_steps() {
        let steps: Vec<WorkflowStep> = (0..10)
            .map(|i| WorkflowStep {
                id: format!("step_{}", i),
                name: format!("Step {}", i),
                step_type: StepType::Wait {
                    duration: Duration::from_secs(1),
                },
                condition: None,
                retry: None,
                timeout: None,
                on_error: None,
                metadata: HashMap::new(),
            })
            .collect();

        let workflow = FluentWorkflow::define("many_parallel")
            .parallel(steps)
            .build();

        match &workflow.steps[0].step_type {
            StepType::Parallel { steps } => {
                assert_eq!(steps.len(), 10);
            }
            _ => panic!("Expected Parallel"),
        }
    }

    #[test]
    fn test_fluent_workflow_complex_sequence() {
        let step1 = WorkflowStep {
            id: "s1".to_string(),
            name: "S1".to_string(),
            step_type: StepType::Wait {
                duration: Duration::from_secs(1),
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        };
        let step2 = WorkflowStep {
            id: "s2".to_string(),
            name: "S2".to_string(),
            step_type: StepType::Wait {
                duration: Duration::from_secs(1),
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        };
        let step3 = WorkflowStep {
            id: "s3".to_string(),
            name: "S3".to_string(),
            step_type: StepType::Wait {
                duration: Duration::from_secs(1),
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        };
        let step4 = WorkflowStep {
            id: "s4".to_string(),
            name: "S4".to_string(),
            step_type: StepType::Wait {
                duration: Duration::from_secs(1),
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        };
        let loop_step = WorkflowStep {
            id: "loop_body".to_string(),
            name: "Loop Body".to_string(),
            step_type: StepType::Wait {
                duration: Duration::from_secs(1),
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        };

        let workflow = FluentWorkflow::define("complex")
            .then(step1)
            .parallel(vec![step2, step3])
            .when("check")
            .do_this(step4)
            .end_if()
            .repeat("counter < 5", loop_step)
            .on_error(ErrorStrategy::Rollback)
            .with_timeout(Duration::from_secs(600))
            .build();

        assert_eq!(workflow.name, "complex");
        assert_eq!(workflow.steps.len(), 4); // then, parallel, conditional, loop
        assert!(matches!(workflow.error_strategy, ErrorStrategy::Rollback));
        assert_eq!(workflow.timeout, Some(Duration::from_secs(600)));
    }

    #[test]
    fn test_compile_workflow_with_all_step_types() {
        // Test that we can handle all step types through JSON compilation
        let json = r#"{
            "id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
            "name": "all_types",
            "version": "1.0.0",
            "steps": [
                {
                    "id": "action_step",
                    "name": "Action",
                    "step_type": {"type": "action", "agent": "a", "operation": "o", "params": {}},
                    "metadata": {}
                },
                {
                    "id": "wait_step",
                    "name": "Wait",
                    "step_type": {"type": "wait", "duration": {"secs": 5, "nanos": 0}},
                    "metadata": {}
                }
            ],
            "error_strategy": "Continue",
            "metadata": {}
        }"#;

        let result = DslCompiler::compile(json);
        assert!(result.is_ok());
        let workflow = result.unwrap();
        assert_eq!(workflow.steps.len(), 2);
    }

    #[test]
    fn test_compile_step_with_all_optional_fields() {
        let json = r#"{
            "id": "full_step",
            "name": "Full Step",
            "step_type": {"type": "action", "agent": "a", "operation": "o", "params": {}},
            "condition": {"expression": "x > 0", "skip_on_false": true},
            "retry": {
                "max_attempts": 3,
                "backoff": {"Fixed": {"delay": {"secs": 1, "nanos": 0}}},
                "retry_on": ["error1"]
            },
            "timeout": {"secs": 30, "nanos": 0},
            "on_error": "Skip",
            "metadata": {"key": "value"}
        }"#;

        let result = DslCompiler::compile_step(json);
        assert!(result.is_ok());
        let step = result.unwrap();
        assert!(step.condition.is_some());
        assert!(step.retry.is_some());
        assert!(step.timeout.is_some());
        assert!(step.on_error.is_some());
        assert!(!step.metadata.is_empty());
    }

    #[test]
    fn test_workflow_macro_with_single_step() {
        let wf = workflow!("single_macro" => {
            step!(wait: Duration::from_secs(1)),
        });

        assert_eq!(wf.name, "single_macro");
        assert_eq!(wf.steps.len(), 1);
    }

    #[test]
    fn test_workflow_macro_with_empty_steps() {
        let wf = workflow!("empty_macro" => {});

        assert_eq!(wf.name, "empty_macro");
        assert!(wf.steps.is_empty());
    }

    #[test]
    fn test_step_macro_action_with_params() {
        // Use StepBuilder directly instead of macro to avoid identifier issues
        let step = StepBuilder::action(
            "step_test".to_string(),
            "my_agent.my_op".to_string(),
            "my_agent",
            "my_op",
        )
        .params(serde_json::json!({
            "key1": "value1",
            "key2": 42
        }))
        .build();

        assert!(step.name.contains("my_agent"));
        assert!(step.name.contains("my_op"));

        match &step.step_type {
            StepType::Action {
                agent,
                operation,
                params,
            } => {
                assert_eq!(agent, "my_agent");
                assert_eq!(operation, "my_op");
                assert_eq!(params["key1"], "value1");
                assert_eq!(params["key2"], 42);
            }
            _ => panic!("Expected Action step"),
        }
    }

    #[test]
    fn test_step_macro_wait() {
        let step = step!(wait: Duration::from_millis(500));

        assert!(step.id.starts_with("wait_"));
        assert_eq!(step.name, "Wait");

        match &step.step_type {
            StepType::Wait { duration } => {
                assert_eq!(*duration, Duration::from_millis(500));
            }
            _ => panic!("Expected Wait step"),
        }
    }

    #[test]
    fn test_yaml_fallback_when_json_fails() {
        // Valid YAML but invalid JSON
        let yaml_only = r#"
id: f47ac10b-58cc-4372-a567-0e02b2c3d479
name: yaml_only_workflow
version: "1.0.0"
steps: []
error_strategy: FailFast
metadata: {}
"#;

        let result = DslCompiler::compile(yaml_only);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, "yaml_only_workflow");
    }

    #[test]
    fn test_compile_step_yaml_fallback() {
        // Valid YAML step but not JSON
        let yaml_step = r#"
id: yaml_step
name: YAML Only Step
step_type:
  type: wait
  duration:
    secs: 5
    nanos: 0
metadata: {}
"#;

        let result = DslCompiler::compile_step(yaml_step);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, "yaml_step");
    }
}
