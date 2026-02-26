// Tests for check_execution_control, save_checkpoint, full workflow
// execution (timeout, state transitions, failures), execute_step result
// recording, and complex nested workflow scenarios.

// ===== check_execution_control tests =====

#[test]
fn test_check_execution_control_running() {
    let registry = Arc::new(AgentRegistry::new());
    let executor = DefaultWorkflowExecutor::new(registry);

    let execution_id = Uuid::new_v4();
    executor
        .execution_states
        .write()
        .insert(execution_id, ExecutionState::default());

    let control = executor.check_execution_control(execution_id).unwrap();
    assert_eq!(control, ExecutionControl::Running);
}

#[test]
fn test_check_execution_control_not_found_defaults_to_running() {
    let registry = Arc::new(AgentRegistry::new());
    let executor = DefaultWorkflowExecutor::new(registry);

    let control = executor.check_execution_control(Uuid::new_v4()).unwrap();
    assert_eq!(control, ExecutionControl::Running);
}

// ===== save_checkpoint tests =====

#[test]
fn test_save_checkpoint() {
    let registry = Arc::new(AgentRegistry::new());
    let executor = DefaultWorkflowExecutor::new(registry);

    let execution_id = Uuid::new_v4();
    executor
        .execution_states
        .write()
        .insert(execution_id, ExecutionState::default());

    let execution_order = vec![
        vec!["step1".to_string()],
        vec!["step2".to_string(), "step3".to_string()],
        vec!["step4".to_string()],
    ];

    let result = executor.save_checkpoint(execution_id, 1, &execution_order);

    assert!(result.is_ok());

    let state = executor
        .execution_states
        .read()
        .get(&execution_id)
        .unwrap()
        .clone();
    assert!(state.checkpoint.is_some());
    let checkpoint = state.checkpoint.unwrap();
    assert_eq!(checkpoint._current_level, 1);
    assert_eq!(checkpoint._completed_steps, vec!["step1".to_string()]);
}

#[test]
fn test_save_checkpoint_no_execution_state() {
    let registry = Arc::new(AgentRegistry::new());
    let executor = DefaultWorkflowExecutor::new(registry);

    let execution_order = vec![vec!["step1".to_string()]];

    // Should succeed even if execution state doesn't exist (no-op)
    let result = executor.save_checkpoint(Uuid::new_v4(), 0, &execution_order);

    assert!(result.is_ok());
}

// ===== execute workflow tests =====

#[actix_rt::test]
async fn test_execute_workflow_with_timeout() {
    let (executor, registry) = setup_executor_with_agent().await;

    let workflow = WorkflowBuilder::new("timeout_test")
        .timeout(Duration::from_millis(5))
        .add_step(WorkflowStep {
            id: "slow_step".to_string(),
            name: "Slow Step".to_string(),
            step_type: StepType::Wait {
                duration: Duration::from_millis(100),
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        })
        .build();

    let context = WorkflowContext::new(workflow.id, registry);
    let result = executor.execute(&workflow, &context).await;

    assert!(result.is_err());
    match result {
        Err(WorkflowError::Timeout) => {}
        _ => panic!("Expected Timeout error"),
    }
}

#[actix_rt::test]
async fn test_execute_workflow_state_transitions() {
    let (executor, registry) = setup_executor_with_agent().await;

    let workflow = WorkflowBuilder::new("state_test")
        .add_step(StepBuilder::action("step1", "Step 1", "test_agent", "op").build())
        .build();

    let context = WorkflowContext::new(workflow.id, registry);

    assert_eq!(context.get_state(), WorkflowState::Created);

    let _result = executor.execute(&workflow, &context).await;

    // After successful execution, state should be Completed
    assert_eq!(context.get_state(), WorkflowState::Completed);
}

#[actix_rt::test]
async fn test_execute_workflow_failed_state() {
    let registry = Arc::new(AgentRegistry::new());
    let executor = DefaultWorkflowExecutor::new(registry.clone());

    let workflow = WorkflowBuilder::new("fail_test")
        .error_strategy(ErrorStrategy::FailFast)
        .add_step(StepBuilder::action("step1", "Step 1", "nonexistent", "op").build())
        .build();

    let context = WorkflowContext::new(workflow.id, registry);

    let _result = executor.execute(&workflow, &context).await;

    // After failed execution, state should be Failed
    assert_eq!(context.get_state(), WorkflowState::Failed);
}

#[actix_rt::test]
async fn test_execute_step_records_result() {
    let (executor, registry) = setup_executor_with_agent().await;
    let context = WorkflowContext::new(Uuid::new_v4(), registry);

    let step = StepBuilder::action("tracked_step", "Tracked Step", "test_agent", "op").build();

    let result = executor.execute_step(&step, &context).await;

    assert!(result.is_ok());

    let step_result = context.get_step_result("tracked_step");
    assert!(step_result.is_some());
    let step_result = step_result.unwrap();
    assert_eq!(step_result.status, StepStatus::Completed);
    assert!(step_result.output.is_some());
    assert!(step_result.completed_at.is_some());
}

#[actix_rt::test]
async fn test_execute_step_records_failure() {
    let registry = Arc::new(AgentRegistry::new());
    let executor = DefaultWorkflowExecutor::new(registry.clone());
    let context = WorkflowContext::new(Uuid::new_v4(), registry);

    let step = StepBuilder::action("failing_step", "Failing Step", "nonexistent", "op").build();

    let result = executor.execute_step(&step, &context).await;

    assert!(result.is_err());

    let step_result = context.get_step_result("failing_step");
    assert!(step_result.is_some());
    let step_result = step_result.unwrap();
    assert_eq!(step_result.status, StepStatus::Failed);
    assert!(step_result.error.is_some());
}

// ===== Complex workflow tests =====

#[actix_rt::test]
async fn test_execute_nested_parallel_in_sequence() {
    let (executor, registry) = setup_executor_with_agent().await;
    let context = WorkflowContext::new(Uuid::new_v4(), registry);

    let parallel_step = WorkflowStep {
        id: "parallel_group".to_string(),
        name: "Parallel Group".to_string(),
        step_type: StepType::Parallel {
            steps: vec![
                StepBuilder::action("p1", "P1", "test_agent", "op1").build(),
                StepBuilder::action("p2", "P2", "test_agent", "op2").build(),
            ],
        },
        condition: None,
        retry: None,
        timeout: None,
        on_error: None,
        metadata: HashMap::new(),
    };

    let sequence_steps = vec![
        StepBuilder::action("s1", "S1", "test_agent", "op1").build(),
        parallel_step,
        StepBuilder::action("s2", "S2", "test_agent", "op2").build(),
    ];

    let result = executor.execute_sequence(&sequence_steps, &context).await;

    assert!(result.is_ok());
}

#[actix_rt::test]
async fn test_execute_workflow_with_multiple_parallel_levels() {
    let (executor, registry) = setup_executor_with_agent().await;

    // Create workflow with independent steps (can run in parallel)
    let workflow = WorkflowBuilder::new("parallel_levels")
        .add_step(StepBuilder::action("step1", "Step 1", "test_agent", "op").build())
        .add_step(StepBuilder::action("step2", "Step 2", "test_agent", "op").build())
        .add_step(StepBuilder::action("step3", "Step 3", "test_agent", "op").build())
        .build();

    let context = WorkflowContext::new(workflow.id, registry);
    let result = executor.execute(&workflow, &context).await;

    assert!(result.is_ok());
}
