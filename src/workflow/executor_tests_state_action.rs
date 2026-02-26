// Tests for ExecutionState/ExecutionControl types, executor construction,
// execute_action, and execute_parallel with state variations.

// ===== ExecutionState and ExecutionControl tests =====

#[test]
fn test_execution_state_default() {
    let state = ExecutionState::default();
    assert_eq!(state.control, ExecutionControl::Running);
    assert!(state.checkpoint.is_none());
}

#[test]
fn test_execution_control_clone() {
    let control = ExecutionControl::Paused;
    let cloned = control.clone();
    assert_eq!(cloned, ExecutionControl::Paused);
}

#[test]
fn test_execution_control_debug() {
    let control = ExecutionControl::Cancelled;
    let debug_str = format!("{:?}", control);
    assert!(debug_str.contains("Cancelled"));
}

// ===== DefaultWorkflowExecutor construction tests =====

#[test]
fn test_executor_new() {
    let registry = Arc::new(AgentRegistry::new());
    let executor = DefaultWorkflowExecutor::new(registry.clone());

    assert!(executor.monitor.is_none());
    assert!(executor.execution_states.read().is_empty());
}

#[actix_rt::test]
async fn test_executor_with_monitor() {
    let registry = Arc::new(AgentRegistry::new());
    let monitor = Arc::new(super::super::monitoring::DefaultWorkflowMonitor::new());
    let executor = DefaultWorkflowExecutor::new(registry).with_monitor(monitor.clone());

    assert!(executor.monitor.is_some());
}

// ===== execute_action tests =====

#[actix_rt::test]
async fn test_execute_action_agent_not_found() {
    let registry = Arc::new(AgentRegistry::new());
    let executor = DefaultWorkflowExecutor::new(registry.clone());
    let context = WorkflowContext::new(Uuid::new_v4(), registry);

    let result = executor
        .execute_action(
            "nonexistent_agent",
            "operation",
            &serde_json::json!({}),
            &context,
        )
        .await;

    assert!(result.is_err());
    match result {
        Err(WorkflowError::AgentError(msg)) => {
            assert!(msg.contains("Agent not found"));
        }
        _ => panic!("Expected AgentError"),
    }
}

#[actix_rt::test]
async fn test_execute_action_with_registered_agent() {
    let (executor, registry) = setup_executor_with_agent().await;
    let context = WorkflowContext::new(Uuid::new_v4(), registry);

    let result = executor
        .execute_action(
            "test_agent",
            "test_operation",
            &serde_json::json!({"key": "value"}),
            &context,
        )
        .await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert_eq!(output["agent_name"], "test_agent");
    assert_eq!(output["operation"], "test_operation");
    assert_eq!(output["status"], "agent_execution_pending");
}

// ===== execute_parallel tests =====

#[actix_rt::test]
async fn test_execute_parallel_empty_steps() {
    let registry = Arc::new(AgentRegistry::new());
    let executor = DefaultWorkflowExecutor::new(registry.clone());
    let context = WorkflowContext::new(Uuid::new_v4(), registry);

    let result = executor.execute_parallel(&[], &context).await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output["results"].as_array().unwrap().is_empty());
}

#[actix_rt::test]
async fn test_execute_parallel_with_non_running_state() {
    let (executor, registry) = setup_executor_with_agent().await;
    let context = WorkflowContext::new(Uuid::new_v4(), registry);
    context.set_state(WorkflowState::Paused); // Not Running state

    let steps = vec![StepBuilder::action("step1", "Step 1", "test_agent", "op").build()];

    let result = executor.execute_parallel(&steps, &context).await;
    // When state is not Running, errors are collected but don't fail
    assert!(result.is_ok());
}
