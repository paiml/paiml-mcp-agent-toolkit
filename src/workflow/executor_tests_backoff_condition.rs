// Tests for calculate_backoff strategies, evaluate_condition,
// resolve_variable, WorkflowExecutor trait (pause/resume/cancel),
// and handle_workflow_error strategies.

// ===== calculate_backoff tests =====

#[test]
fn test_calculate_backoff_fixed() {
    let registry = Arc::new(AgentRegistry::new());
    let executor = DefaultWorkflowExecutor::new(registry);

    let strategy = BackoffStrategy::Fixed {
        delay: Duration::from_secs(5),
    };

    assert_eq!(
        executor.calculate_backoff(&strategy, 1),
        Duration::from_secs(5)
    );
    assert_eq!(
        executor.calculate_backoff(&strategy, 5),
        Duration::from_secs(5)
    );
    assert_eq!(
        executor.calculate_backoff(&strategy, 100),
        Duration::from_secs(5)
    );
}

#[test]
fn test_calculate_backoff_exponential() {
    let registry = Arc::new(AgentRegistry::new());
    let executor = DefaultWorkflowExecutor::new(registry);

    let strategy = BackoffStrategy::Exponential {
        initial: Duration::from_secs(1),
        multiplier: 2.0,
        max: Duration::from_secs(16),
    };

    assert_eq!(
        executor.calculate_backoff(&strategy, 1),
        Duration::from_secs(1)
    );
    assert_eq!(
        executor.calculate_backoff(&strategy, 2),
        Duration::from_secs(2)
    );
    assert_eq!(
        executor.calculate_backoff(&strategy, 3),
        Duration::from_secs(4)
    );
    assert_eq!(
        executor.calculate_backoff(&strategy, 4),
        Duration::from_secs(8)
    );
    // Should cap at max
    assert_eq!(
        executor.calculate_backoff(&strategy, 5),
        Duration::from_secs(16)
    );
    assert_eq!(
        executor.calculate_backoff(&strategy, 10),
        Duration::from_secs(16)
    );
}

#[test]
fn test_calculate_backoff_linear() {
    let registry = Arc::new(AgentRegistry::new());
    let executor = DefaultWorkflowExecutor::new(registry);

    let strategy = BackoffStrategy::Linear {
        initial: Duration::from_secs(1),
        increment: Duration::from_secs(2),
    };

    assert_eq!(
        executor.calculate_backoff(&strategy, 1),
        Duration::from_secs(1)
    );
    assert_eq!(
        executor.calculate_backoff(&strategy, 2),
        Duration::from_secs(3)
    );
    assert_eq!(
        executor.calculate_backoff(&strategy, 3),
        Duration::from_secs(5)
    );
    assert_eq!(
        executor.calculate_backoff(&strategy, 4),
        Duration::from_secs(7)
    );
}

// ===== evaluate_condition tests =====

#[actix_rt::test]
async fn test_evaluate_condition_greater_than_true() {
    let registry = Arc::new(AgentRegistry::new());
    let executor = DefaultWorkflowExecutor::new(registry.clone());
    let context = WorkflowContext::new(Uuid::new_v4(), registry);
    context.set_variable("x".to_string(), serde_json::json!(10));
    context.set_variable("y".to_string(), serde_json::json!(5));

    let result = executor.evaluate_condition("x > y", &context).await;

    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[actix_rt::test]
async fn test_evaluate_condition_greater_than_false() {
    let registry = Arc::new(AgentRegistry::new());
    let executor = DefaultWorkflowExecutor::new(registry.clone());
    let context = WorkflowContext::new(Uuid::new_v4(), registry);
    context.set_variable("x".to_string(), serde_json::json!(3));
    context.set_variable("y".to_string(), serde_json::json!(10));

    let result = executor.evaluate_condition("x > y", &context).await;

    assert!(result.is_ok());
    assert!(!result.unwrap());
}

#[actix_rt::test]
async fn test_evaluate_condition_equals_true() {
    let registry = Arc::new(AgentRegistry::new());
    let executor = DefaultWorkflowExecutor::new(registry.clone());
    let context = WorkflowContext::new(Uuid::new_v4(), registry);
    context.set_variable("status".to_string(), serde_json::json!("success"));
    context.set_variable("expected".to_string(), serde_json::json!("success"));

    let result = executor
        .evaluate_condition("status == expected", &context)
        .await;

    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[actix_rt::test]
async fn test_evaluate_condition_equals_false() {
    let registry = Arc::new(AgentRegistry::new());
    let executor = DefaultWorkflowExecutor::new(registry.clone());
    let context = WorkflowContext::new(Uuid::new_v4(), registry);
    context.set_variable("status".to_string(), serde_json::json!("error"));
    context.set_variable("expected".to_string(), serde_json::json!("success"));

    let result = executor
        .evaluate_condition("status == expected", &context)
        .await;

    assert!(result.is_ok());
    assert!(!result.unwrap());
}

#[actix_rt::test]
async fn test_evaluate_condition_default_true() {
    let registry = Arc::new(AgentRegistry::new());
    let executor = DefaultWorkflowExecutor::new(registry.clone());
    let context = WorkflowContext::new(Uuid::new_v4(), registry);

    // Unknown expression defaults to true
    let result = executor
        .evaluate_condition("some_unknown_expr", &context)
        .await;

    assert!(result.is_ok());
    assert!(result.unwrap());
}

// ===== resolve_variable tests =====

#[test]
fn test_resolve_variable_simple() {
    let registry = Arc::new(AgentRegistry::new());
    let executor = DefaultWorkflowExecutor::new(registry.clone());
    let context = WorkflowContext::new(Uuid::new_v4(), registry);
    context.set_variable("my_var".to_string(), serde_json::json!(42));

    let result = executor.resolve_variable("my_var", &context);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), serde_json::json!(42));
}

#[test]
fn test_resolve_variable_not_found() {
    let registry = Arc::new(AgentRegistry::new());
    let executor = DefaultWorkflowExecutor::new(registry.clone());
    let context = WorkflowContext::new(Uuid::new_v4(), registry);

    let result = executor.resolve_variable("nonexistent", &context);

    assert!(result.is_err());
    match result {
        Err(WorkflowError::VariableNotFound(name)) => {
            assert_eq!(name, "nonexistent");
        }
        _ => panic!("Expected VariableNotFound error"),
    }
}

#[test]
fn test_resolve_variable_step_result_status() {
    let registry = Arc::new(AgentRegistry::new());
    let executor = DefaultWorkflowExecutor::new(registry.clone());
    let context = WorkflowContext::new(Uuid::new_v4(), registry);

    // Add a step result
    context.set_step_result(
        "step1".to_string(),
        StepResult {
            step_id: "step1".to_string(),
            status: StepStatus::Completed,
            output: Some(serde_json::json!({"result": "ok"})),
            error: None,
            started_at: Instant::now(),
            completed_at: Some(Instant::now()),
            attempts: 1,
        },
    );

    let result = executor.resolve_variable("steps.step1.status", &context);

    assert!(result.is_ok());
    let value = result.unwrap();
    assert!(value.as_str().unwrap().contains("Completed"));
}

#[test]
fn test_resolve_variable_step_result_output() {
    let registry = Arc::new(AgentRegistry::new());
    let executor = DefaultWorkflowExecutor::new(registry.clone());
    let context = WorkflowContext::new(Uuid::new_v4(), registry);

    // Add a step result
    context.set_step_result(
        "step1".to_string(),
        StepResult {
            step_id: "step1".to_string(),
            status: StepStatus::Completed,
            output: Some(serde_json::json!({"result": "ok", "count": 5})),
            error: None,
            started_at: Instant::now(),
            completed_at: Some(Instant::now()),
            attempts: 1,
        },
    );

    let result = executor.resolve_variable("steps.step1.output.result", &context);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), serde_json::json!("ok"));
}

// ===== WorkflowExecutor trait tests =====

#[actix_rt::test]
async fn test_pause_not_found() {
    let registry = Arc::new(AgentRegistry::new());
    let executor = DefaultWorkflowExecutor::new(registry);

    let result = executor.pause(Uuid::new_v4()).await;

    assert!(result.is_err());
    match result {
        Err(WorkflowError::NotFound(_)) => {}
        _ => panic!("Expected NotFound error"),
    }
}

#[actix_rt::test]
async fn test_resume_not_found() {
    let registry = Arc::new(AgentRegistry::new());
    let executor = DefaultWorkflowExecutor::new(registry);

    let result = executor.resume(Uuid::new_v4()).await;

    assert!(result.is_err());
    match result {
        Err(WorkflowError::NotFound(_)) => {}
        _ => panic!("Expected NotFound error"),
    }
}

#[actix_rt::test]
async fn test_resume_not_paused() {
    let registry = Arc::new(AgentRegistry::new());
    let executor = DefaultWorkflowExecutor::new(registry);

    let execution_id = Uuid::new_v4();
    executor
        .execution_states
        .write()
        .insert(execution_id, ExecutionState::default());

    let result = executor.resume(execution_id).await;

    assert!(result.is_err());
    match result {
        Err(WorkflowError::ExecutionError(msg)) => {
            assert!(msg.contains("not paused"));
        }
        _ => panic!("Expected ExecutionError"),
    }
}

#[actix_rt::test]
async fn test_cancel_not_found() {
    let registry = Arc::new(AgentRegistry::new());
    let executor = DefaultWorkflowExecutor::new(registry);

    let result = executor.cancel(Uuid::new_v4()).await;

    assert!(result.is_err());
    match result {
        Err(WorkflowError::NotFound(_)) => {}
        _ => panic!("Expected NotFound error"),
    }
}

#[actix_rt::test]
async fn test_cancel_success() {
    let registry = Arc::new(AgentRegistry::new());
    let executor = DefaultWorkflowExecutor::new(registry);

    let execution_id = Uuid::new_v4();
    executor
        .execution_states
        .write()
        .insert(execution_id, ExecutionState::default());

    let result = executor.cancel(execution_id).await;

    assert!(result.is_ok());
    let state = executor
        .execution_states
        .read()
        .get(&execution_id)
        .unwrap()
        .control
        .clone();
    assert_eq!(state, ExecutionControl::Cancelled);
}

// ===== handle_workflow_error tests =====

#[actix_rt::test]
async fn test_handle_workflow_error_fail_fast() {
    let registry = Arc::new(AgentRegistry::new());
    let executor = DefaultWorkflowExecutor::new(registry.clone());
    let context = WorkflowContext::new(Uuid::new_v4(), registry);

    let error = WorkflowError::StepFailed("test error".to_string());
    let result = executor.handle_workflow_error(error, &ErrorStrategy::FailFast, &context);

    assert!(result.is_err());
}

#[actix_rt::test]
async fn test_handle_workflow_error_continue() {
    let registry = Arc::new(AgentRegistry::new());
    let executor = DefaultWorkflowExecutor::new(registry.clone());
    let context = WorkflowContext::new(Uuid::new_v4(), registry);

    let error = WorkflowError::StepFailed("test error".to_string());
    let result = executor.handle_workflow_error(error, &ErrorStrategy::Continue, &context);

    assert!(result.is_ok());
    let output = result.unwrap();
    assert_eq!(output["continued_after_error"], true);
}

#[actix_rt::test]
async fn test_handle_workflow_error_rollback() {
    let registry = Arc::new(AgentRegistry::new());
    let executor = DefaultWorkflowExecutor::new(registry.clone());
    let context = WorkflowContext::new(Uuid::new_v4(), registry);

    let error = WorkflowError::StepFailed("test error".to_string());
    let result = executor.handle_workflow_error(error, &ErrorStrategy::Rollback, &context);

    // Rollback returns original error
    assert!(result.is_err());
}

#[actix_rt::test]
async fn test_handle_workflow_error_compensate() {
    let registry = Arc::new(AgentRegistry::new());
    let executor = DefaultWorkflowExecutor::new(registry.clone());
    let context = WorkflowContext::new(Uuid::new_v4(), registry);

    let error = WorkflowError::StepFailed("test error".to_string());
    let result = executor.handle_workflow_error(error, &ErrorStrategy::Compensate, &context);

    // Compensate returns original error
    assert!(result.is_err());
}
