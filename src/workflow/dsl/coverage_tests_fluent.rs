#![cfg_attr(coverage_nightly, coverage(off))]

#[cfg(test)]
mod coverage_tests_fluent {
    use crate::workflow::dsl::*;
    use crate::workflow::*;
    use std::collections::HashMap;
    use std::time::Duration;

    // =========================================================================
    // FluentWorkflow tests
    // =========================================================================

    #[test]
    fn test_fluent_workflow_define() {
        let fluent = FluentWorkflow::define("my_workflow");
        let workflow = fluent.build();
        assert_eq!(workflow.name, "my_workflow");
        assert!(workflow.steps.is_empty());
    }

    #[test]
    fn test_fluent_workflow_define_string_type() {
        let name = String::from("string_workflow");
        let fluent = FluentWorkflow::define(name);
        let workflow = fluent.build();
        assert_eq!(workflow.name, "string_workflow");
    }

    #[test]
    fn test_fluent_workflow_then_single_step() {
        let step = WorkflowStep {
            id: "test_step".to_string(),
            name: "Test".to_string(),
            step_type: StepType::Wait {
                duration: Duration::from_secs(1),
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        };

        let workflow = FluentWorkflow::define("test").then(step).build();
        assert_eq!(workflow.steps.len(), 1);
        assert_eq!(workflow.steps[0].id, "test_step");
    }

    #[test]
    fn test_fluent_workflow_then_multiple_steps() {
        let step1 = WorkflowStep {
            id: "step1".to_string(),
            name: "Step 1".to_string(),
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
            id: "step2".to_string(),
            name: "Step 2".to_string(),
            step_type: StepType::Wait {
                duration: Duration::from_secs(2),
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        };
        let step3 = WorkflowStep {
            id: "step3".to_string(),
            name: "Step 3".to_string(),
            step_type: StepType::Wait {
                duration: Duration::from_secs(3),
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        };

        let workflow = FluentWorkflow::define("multi_step")
            .then(step1)
            .then(step2)
            .then(step3)
            .build();

        assert_eq!(workflow.steps.len(), 3);
        assert_eq!(workflow.steps[0].id, "step1");
        assert_eq!(workflow.steps[1].id, "step2");
        assert_eq!(workflow.steps[2].id, "step3");
    }

    #[test]
    fn test_fluent_workflow_parallel() {
        let step1 = WorkflowStep {
            id: "parallel_step1".to_string(),
            name: "Parallel 1".to_string(),
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
            id: "parallel_step2".to_string(),
            name: "Parallel 2".to_string(),
            step_type: StepType::Wait {
                duration: Duration::from_secs(1),
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        };

        let workflow = FluentWorkflow::define("parallel_workflow")
            .parallel(vec![step1, step2])
            .build();

        assert_eq!(workflow.steps.len(), 1);
        match &workflow.steps[0].step_type {
            StepType::Parallel { steps } => {
                assert_eq!(steps.len(), 2);
            }
            _ => panic!("Expected Parallel step type"),
        }
        assert!(workflow.steps[0].id.starts_with("parallel_"));
        assert_eq!(workflow.steps[0].name, "Parallel Execution");
    }

    #[test]
    fn test_fluent_workflow_parallel_empty_steps() {
        let workflow = FluentWorkflow::define("empty_parallel")
            .parallel(vec![])
            .build();

        assert_eq!(workflow.steps.len(), 1);
        match &workflow.steps[0].step_type {
            StepType::Parallel { steps } => {
                assert!(steps.is_empty());
            }
            _ => panic!("Expected Parallel step type"),
        }
    }

    #[test]
    fn test_fluent_workflow_repeat() {
        let step = WorkflowStep {
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

        let workflow = FluentWorkflow::define("loop_workflow")
            .repeat("counter < 10", step)
            .build();

        assert_eq!(workflow.steps.len(), 1);
        match &workflow.steps[0].step_type {
            StepType::Loop {
                condition,
                step,
                max_iterations,
            } => {
                assert_eq!(condition, "counter < 10");
                assert_eq!(step.id, "loop_body");
                assert!(max_iterations.is_none());
            }
            _ => panic!("Expected Loop step type"),
        }
        assert!(workflow.steps[0].id.starts_with("loop_"));
        assert_eq!(workflow.steps[0].name, "Loop");
    }

    #[test]
    fn test_fluent_workflow_repeat_with_string_condition() {
        let step = WorkflowStep {
            id: "body".to_string(),
            name: "Body".to_string(),
            step_type: StepType::Wait {
                duration: Duration::from_secs(1),
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        };

        let condition = String::from("status == 'pending'");
        let workflow = FluentWorkflow::define("test")
            .repeat(condition, step)
            .build();

        match &workflow.steps[0].step_type {
            StepType::Loop { condition, .. } => {
                assert_eq!(condition, "status == 'pending'");
            }
            _ => panic!("Expected Loop step type"),
        }
    }

    #[test]
    fn test_fluent_workflow_on_error_failfast() {
        let workflow = FluentWorkflow::define("error_workflow")
            .on_error(ErrorStrategy::FailFast)
            .build();

        assert!(matches!(workflow.error_strategy, ErrorStrategy::FailFast));
    }

    #[test]
    fn test_fluent_workflow_on_error_continue() {
        let workflow = FluentWorkflow::define("error_workflow")
            .on_error(ErrorStrategy::Continue)
            .build();

        assert!(matches!(workflow.error_strategy, ErrorStrategy::Continue));
    }

    #[test]
    fn test_fluent_workflow_on_error_rollback() {
        let workflow = FluentWorkflow::define("error_workflow")
            .on_error(ErrorStrategy::Rollback)
            .build();

        assert!(matches!(workflow.error_strategy, ErrorStrategy::Rollback));
    }

    #[test]
    fn test_fluent_workflow_on_error_compensate() {
        let workflow = FluentWorkflow::define("error_workflow")
            .on_error(ErrorStrategy::Compensate)
            .build();

        assert!(matches!(workflow.error_strategy, ErrorStrategy::Compensate));
    }

    #[test]
    fn test_fluent_workflow_with_timeout() {
        let workflow = FluentWorkflow::define("timeout_workflow")
            .with_timeout(Duration::from_secs(120))
            .build();

        assert_eq!(workflow.timeout, Some(Duration::from_secs(120)));
    }

    #[test]
    fn test_fluent_workflow_with_timeout_zero() {
        let workflow = FluentWorkflow::define("zero_timeout")
            .with_timeout(Duration::from_secs(0))
            .build();

        assert_eq!(workflow.timeout, Some(Duration::from_secs(0)));
    }

    #[test]
    fn test_fluent_workflow_chained_operations() {
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

        let workflow = FluentWorkflow::define("chained")
            .then(step1)
            .parallel(vec![step2])
            .on_error(ErrorStrategy::Continue)
            .with_timeout(Duration::from_secs(300))
            .build();

        assert_eq!(workflow.name, "chained");
        assert_eq!(workflow.steps.len(), 2);
        assert!(matches!(workflow.error_strategy, ErrorStrategy::Continue));
        assert_eq!(workflow.timeout, Some(Duration::from_secs(300)));
    }
}
