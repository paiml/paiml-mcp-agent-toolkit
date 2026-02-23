#![cfg_attr(coverage_nightly, coverage(off))]

#[cfg(test)]
mod coverage_tests_conditionals {
    use crate::workflow::dsl::*;
    use crate::workflow::*;
    use std::collections::HashMap;
    use std::time::Duration;

    // =========================================================================
    // ConditionalFlow tests
    // =========================================================================

    #[test]
    fn test_conditional_flow_when_creates_flow() {
        let workflow = FluentWorkflow::define("cond_test");
        let conditional = workflow.when("x > 5");

        // The condition is stored correctly (we verify by completing the flow)
        let step = WorkflowStep {
            id: "if_step".to_string(),
            name: "If Step".to_string(),
            step_type: StepType::Wait {
                duration: Duration::from_secs(1),
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        };

        let else_obj = conditional.do_this(step);
        let wf = else_obj.end_if().build();

        // Verify the condition was used
        match &wf.steps[0].step_type {
            StepType::Conditional { condition, .. } => {
                assert_eq!(condition, "x > 5");
            }
            _ => panic!("Expected Conditional step"),
        }
    }

    #[test]
    fn test_conditional_flow_when_with_string_condition() {
        let workflow = FluentWorkflow::define("cond_test");
        let condition = String::from("status == 'active'");
        let conditional = workflow.when(condition);

        let step = WorkflowStep {
            id: "step".to_string(),
            name: "Step".to_string(),
            step_type: StepType::Wait {
                duration: Duration::from_secs(1),
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        };

        let wf = conditional.do_this(step).end_if().build();

        match &wf.steps[0].step_type {
            StepType::Conditional { condition, .. } => {
                assert_eq!(condition, "status == 'active'");
            }
            _ => panic!("Expected Conditional step"),
        }
    }

    #[test]
    fn test_conditional_flow_do_this() {
        let if_step = WorkflowStep {
            id: "if_step".to_string(),
            name: "If Step".to_string(),
            step_type: StepType::Wait {
                duration: Duration::from_secs(1),
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        };

        let conditional_else = FluentWorkflow::define("test")
            .when("condition")
            .do_this(if_step);

        // Verify do_this returns ConditionalElse with correct if_true step
        let wf = conditional_else.end_if().build();
        match &wf.steps[0].step_type {
            StepType::Conditional { if_true, .. } => {
                assert_eq!(if_true.id, "if_step");
            }
            _ => panic!("Expected Conditional step"),
        }
    }

    // =========================================================================
    // ConditionalElse tests
    // =========================================================================

    #[test]
    fn test_conditional_else_otherwise() {
        let if_step = WorkflowStep {
            id: "if_step".to_string(),
            name: "If Step".to_string(),
            step_type: StepType::Wait {
                duration: Duration::from_secs(1),
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        };
        let else_step = WorkflowStep {
            id: "else_step".to_string(),
            name: "Else Step".to_string(),
            step_type: StepType::Wait {
                duration: Duration::from_secs(2),
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        };

        let workflow = FluentWorkflow::define("conditional")
            .when("x > 10")
            .do_this(if_step)
            .otherwise(else_step)
            .build();

        assert_eq!(workflow.steps.len(), 1);
        match &workflow.steps[0].step_type {
            StepType::Conditional {
                condition,
                if_true,
                if_false,
            } => {
                assert_eq!(condition, "x > 10");
                assert_eq!(if_true.id, "if_step");
                assert!(if_false.is_some());
                assert_eq!(if_false.as_ref().unwrap().id, "else_step");
            }
            _ => panic!("Expected Conditional step"),
        }
        assert!(workflow.steps[0].id.starts_with("cond_"));
        assert_eq!(workflow.steps[0].name, "Conditional");
    }

    #[test]
    fn test_conditional_else_end_if() {
        let if_step = WorkflowStep {
            id: "only_if".to_string(),
            name: "Only If".to_string(),
            step_type: StepType::Wait {
                duration: Duration::from_secs(1),
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        };

        let workflow = FluentWorkflow::define("no_else")
            .when("y < 5")
            .do_this(if_step)
            .end_if()
            .build();

        assert_eq!(workflow.steps.len(), 1);
        match &workflow.steps[0].step_type {
            StepType::Conditional {
                condition,
                if_true,
                if_false,
            } => {
                assert_eq!(condition, "y < 5");
                assert_eq!(if_true.id, "only_if");
                assert!(if_false.is_none());
            }
            _ => panic!("Expected Conditional step"),
        }
    }

    #[test]
    fn test_conditional_else_end_if_then_continue() {
        let if_step = WorkflowStep {
            id: "if".to_string(),
            name: "If".to_string(),
            step_type: StepType::Wait {
                duration: Duration::from_secs(1),
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        };
        let next_step = WorkflowStep {
            id: "next".to_string(),
            name: "Next".to_string(),
            step_type: StepType::Wait {
                duration: Duration::from_secs(1),
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        };

        let workflow = FluentWorkflow::define("continue_after_if")
            .when("cond")
            .do_this(if_step)
            .end_if()
            .then(next_step)
            .build();

        assert_eq!(workflow.steps.len(), 2);
        assert!(matches!(
            &workflow.steps[0].step_type,
            StepType::Conditional { .. }
        ));
        assert_eq!(workflow.steps[1].id, "next");
    }

    #[test]
    fn test_conditional_else_otherwise_then_continue() {
        let if_step = WorkflowStep {
            id: "if".to_string(),
            name: "If".to_string(),
            step_type: StepType::Wait {
                duration: Duration::from_secs(1),
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        };
        let else_step = WorkflowStep {
            id: "else".to_string(),
            name: "Else".to_string(),
            step_type: StepType::Wait {
                duration: Duration::from_secs(1),
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        };
        let next_step = WorkflowStep {
            id: "next".to_string(),
            name: "Next".to_string(),
            step_type: StepType::Wait {
                duration: Duration::from_secs(1),
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        };

        let workflow = FluentWorkflow::define("continue_after_else")
            .when("cond")
            .do_this(if_step)
            .otherwise(else_step)
            .then(next_step)
            .build();

        assert_eq!(workflow.steps.len(), 2);
        assert!(matches!(
            &workflow.steps[0].step_type,
            StepType::Conditional { .. }
        ));
        assert_eq!(workflow.steps[1].id, "next");
    }
}
