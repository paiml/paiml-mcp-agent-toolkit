#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_builder() {
        let workflow = WorkflowBuilder::new("test_workflow")
            .description("Test workflow")
            .version("2.0.0")
            .error_strategy(ErrorStrategy::Continue)
            .timeout(Duration::from_secs(300))
            .build();

        assert_eq!(workflow.name, "test_workflow");
        assert_eq!(workflow.version, "2.0.0");
        assert!(matches!(workflow.error_strategy, ErrorStrategy::Continue));
        assert_eq!(workflow.timeout, Some(Duration::from_secs(300)));
    }

    #[test]
    fn test_step_builder() {
        let step = StepBuilder::action("step1", "Analyze", "analyzer", "analyze")
            .params(serde_json::json!({"language": "rust"}))
            .condition("result.score > 0.8", true)
            .retry(
                3,
                BackoffStrategy::Exponential {
                    initial: Duration::from_secs(1),
                    multiplier: 2.0,
                    max: Duration::from_secs(10),
                },
            )
            .timeout(Duration::from_secs(30))
            .build();

        assert_eq!(step.id, "step1");
        assert_eq!(step.name, "Analyze");
        assert!(step.condition.is_some());
        assert!(step.retry.is_some());
        assert_eq!(step.timeout, Some(Duration::from_secs(30)));
    }

    #[test]
    fn test_step_status_variants() {
        assert_ne!(StepStatus::Pending, StepStatus::Running);
        assert_ne!(StepStatus::Running, StepStatus::Completed);
        assert_ne!(StepStatus::Completed, StepStatus::Failed);
        assert_ne!(StepStatus::Failed, StepStatus::Skipped);
        assert_ne!(StepStatus::Skipped, StepStatus::Cancelled);
    }

    #[test]
    fn test_workflow_state_variants() {
        assert_ne!(WorkflowState::Created, WorkflowState::Running);
        assert_ne!(WorkflowState::Running, WorkflowState::Paused);
        assert_ne!(WorkflowState::Paused, WorkflowState::Completed);
        assert_ne!(WorkflowState::Completed, WorkflowState::Failed);
        assert_ne!(WorkflowState::Failed, WorkflowState::Cancelled);
    }

    #[test]
    fn test_backoff_strategy_fixed() {
        let strategy = BackoffStrategy::Fixed {
            delay: Duration::from_secs(5),
        };
        if let BackoffStrategy::Fixed { delay } = strategy {
            assert_eq!(delay, Duration::from_secs(5));
        } else {
            panic!("Expected Fixed strategy");
        }
    }

    #[test]
    fn test_backoff_strategy_exponential() {
        let strategy = BackoffStrategy::Exponential {
            initial: Duration::from_secs(1),
            multiplier: 2.0,
            max: Duration::from_secs(60),
        };
        if let BackoffStrategy::Exponential {
            initial,
            multiplier,
            max,
        } = strategy
        {
            assert_eq!(initial, Duration::from_secs(1));
            assert_eq!(multiplier, 2.0);
            assert_eq!(max, Duration::from_secs(60));
        } else {
            panic!("Expected Exponential strategy");
        }
    }

    #[test]
    fn test_backoff_strategy_linear() {
        let strategy = BackoffStrategy::Linear {
            initial: Duration::from_secs(1),
            increment: Duration::from_secs(1),
        };
        if let BackoffStrategy::Linear { initial, increment } = strategy {
            assert_eq!(initial, Duration::from_secs(1));
            assert_eq!(increment, Duration::from_secs(1));
        } else {
            panic!("Expected Linear strategy");
        }
    }

    #[test]
    fn test_error_handler_variants() {
        let skip = ErrorHandler::Skip;
        let fail = ErrorHandler::Fail;
        let goto = ErrorHandler::Goto {
            step_id: "step2".to_string(),
        };
        let compensate = ErrorHandler::Compensate {
            steps: vec!["cleanup".to_string()],
        };

        assert!(matches!(skip, ErrorHandler::Skip));
        assert!(matches!(fail, ErrorHandler::Fail));
        assert!(matches!(goto, ErrorHandler::Goto { .. }));
        assert!(matches!(compensate, ErrorHandler::Compensate { .. }));
    }

    #[test]
    fn test_error_strategy_variants() {
        assert!(matches!(ErrorStrategy::FailFast, ErrorStrategy::FailFast));
        assert!(matches!(ErrorStrategy::Continue, ErrorStrategy::Continue));
        assert!(matches!(ErrorStrategy::Rollback, ErrorStrategy::Rollback));
        assert!(matches!(
            ErrorStrategy::Compensate,
            ErrorStrategy::Compensate
        ));
    }

    #[test]
    fn test_step_condition() {
        let condition = StepCondition {
            expression: "result.valid == true".to_string(),
            skip_on_false: true,
        };
        assert_eq!(condition.expression, "result.valid == true");
        assert!(condition.skip_on_false);
    }

    #[test]
    fn test_retry_policy() {
        let policy = RetryPolicy {
            max_attempts: 3,
            backoff: BackoffStrategy::Fixed {
                delay: Duration::from_secs(1),
            },
            retry_on: vec!["NetworkError".to_string()],
        };
        assert_eq!(policy.max_attempts, 3);
        assert_eq!(policy.retry_on.len(), 1);
    }

    #[test]
    fn test_step_result_creation() {
        let result = StepResult {
            step_id: "test_step".to_string(),
            status: StepStatus::Completed,
            output: Some(serde_json::json!({"value": 42})),
            error: None,
            started_at: Instant::now(),
            completed_at: Some(Instant::now()),
            attempts: 1,
        };
        assert_eq!(result.step_id, "test_step");
        assert_eq!(result.status, StepStatus::Completed);
        assert!(result.output.is_some());
        assert!(result.error.is_none());
        assert_eq!(result.attempts, 1);
    }

    #[test]
    fn test_workflow_builder_defaults() {
        let workflow = WorkflowBuilder::new("default_test").build();
        assert_eq!(workflow.name, "default_test");
        assert_eq!(workflow.version, "1.0.0");
        assert!(matches!(workflow.error_strategy, ErrorStrategy::FailFast));
        assert!(workflow.timeout.is_none());
        assert!(workflow.description.is_none());
    }

    #[test]
    fn test_step_type_action() {
        let step_type = StepType::Action {
            agent: "analyzer".to_string(),
            operation: "analyze".to_string(),
            params: serde_json::json!({"path": "."}),
        };
        if let StepType::Action {
            agent,
            operation,
            params,
        } = step_type
        {
            assert_eq!(agent, "analyzer");
            assert_eq!(operation, "analyze");
            assert!(params.is_object());
        } else {
            panic!("Expected Action step type");
        }
    }

    #[test]
    fn test_step_type_wait() {
        let step_type = StepType::Wait {
            duration: Duration::from_secs(10),
        };
        if let StepType::Wait { duration } = step_type {
            assert_eq!(duration, Duration::from_secs(10));
        } else {
            panic!("Expected Wait step type");
        }
    }

    #[test]
    fn test_workflow_serialization() {
        let workflow = WorkflowBuilder::new("serialization_test")
            .description("Test serialization")
            .version("1.0.0")
            .build();
        let json = serde_json::to_string(&workflow).unwrap();
        assert!(json.contains("serialization_test"));
        assert!(json.contains("Test serialization"));
    }

    #[test]
    fn test_step_result_with_error() {
        let result = StepResult {
            step_id: "failing_step".to_string(),
            status: StepStatus::Failed,
            output: None,
            error: Some("Connection timeout".to_string()),
            started_at: Instant::now(),
            completed_at: Some(Instant::now()),
            attempts: 3,
        };
        assert_eq!(result.status, StepStatus::Failed);
        assert!(result.error.is_some());
        assert_eq!(result.error.unwrap(), "Connection timeout");
    }

    #[test]
    fn test_workflow_error_display() {
        let err = WorkflowError::StepFailed("Test step failed".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Step failed") || msg.contains("Test step"));

        let invalid_err = WorkflowError::InvalidDefinition("Invalid config".to_string());
        assert!(invalid_err.to_string().contains("Invalid"));

        let timeout_err = WorkflowError::Timeout;
        assert!(timeout_err.to_string().contains("Timeout"));

        let not_found_err = WorkflowError::NotFound(Uuid::new_v4());
        assert!(not_found_err.to_string().contains("not found"));

        let cancelled_err = WorkflowError::Cancelled;
        assert!(cancelled_err.to_string().contains("cancelled"));
    }
}
