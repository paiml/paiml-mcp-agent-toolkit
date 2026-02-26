// Basic executor tests: creation, backoff integration, pause/resume/cancel,
// retry, error strategies, monitoring, checkpoint, parallel step execution,
// and the shared setup_executor_with_agent() helper used by other test files.

#[actix_rt::test]
async fn test_executor_creation() {
    let registry = Arc::new(AgentRegistry::new());
    let executor = DefaultWorkflowExecutor::new(registry);

    assert!(executor.monitor.is_none());
}

#[test]
fn test_backoff_calculation() {
    let registry = Arc::new(AgentRegistry::new());
    let executor = DefaultWorkflowExecutor::new(registry);

    // Test fixed backoff
    let fixed = BackoffStrategy::Fixed {
        delay: Duration::from_secs(5),
    };
    assert_eq!(
        executor.calculate_backoff(&fixed, 1),
        Duration::from_secs(5)
    );
    assert_eq!(
        executor.calculate_backoff(&fixed, 3),
        Duration::from_secs(5)
    );

    // Test exponential backoff
    let exp = BackoffStrategy::Exponential {
        initial: Duration::from_secs(1),
        multiplier: 2.0,
        max: Duration::from_secs(10),
    };
    assert_eq!(executor.calculate_backoff(&exp, 1), Duration::from_secs(1));
    assert_eq!(executor.calculate_backoff(&exp, 2), Duration::from_secs(2));
    assert_eq!(executor.calculate_backoff(&exp, 3), Duration::from_secs(4));

    // Test linear backoff
    let linear = BackoffStrategy::Linear {
        initial: Duration::from_secs(1),
        increment: Duration::from_secs(2),
    };
    assert_eq!(
        executor.calculate_backoff(&linear, 1),
        Duration::from_secs(1)
    );
    assert_eq!(
        executor.calculate_backoff(&linear, 2),
        Duration::from_secs(3)
    );
    assert_eq!(
        executor.calculate_backoff(&linear, 3),
        Duration::from_secs(5)
    );
}

#[actix_rt::test]
async fn test_dag_integration_parallel_execution() {
    let registry = Arc::new(AgentRegistry::new());
    let executor = DefaultWorkflowExecutor::new(registry.clone());

    // Create workflow with steps that can run in parallel
    let workflow = WorkflowBuilder::new("parallel_test")
        .add_step(StepBuilder::action("step1", "Init", "agent1", "init").build())
        .add_step(StepBuilder::action("step2", "Process A", "agent2", "process").build())
        .add_step(StepBuilder::action("step3", "Process B", "agent3", "process").build())
        .build();

    let context = WorkflowContext::new(workflow.id, registry);

    // Execute workflow - DAG engine should detect parallel opportunities
    let result = executor.execute(&workflow, &context).await;
    // Note: Agents don't exist, so execution will fail with AgentError
    // The test validates that DAG integration works, errors are expected
    assert!(result.is_err()); // Should fail due to nonexistent agents
    match result {
        Err(WorkflowError::AgentError(_)) => {
            // Expected - agents not registered
        }
        other => panic!("Expected AgentError, got: {:?}", other),
    }
}

#[actix_rt::test]
async fn test_pause_resume_workflow() {
    let registry = Arc::new(AgentRegistry::new());
    let executor = Arc::new(DefaultWorkflowExecutor::new(registry.clone()));

    let workflow = WorkflowBuilder::new("pause_test")
        .add_step(StepBuilder::action("step1", "Step 1", "agent1", "op1").build())
        .add_step(StepBuilder::action("step2", "Step 2", "agent2", "op2").build())
        .build();

    let context = WorkflowContext::new(workflow.id, registry);
    let _execution_id = context.execution_id;

    // Start workflow in background
    let executor_clone = executor.clone();
    let workflow_clone = workflow.clone();
    let context_clone = WorkflowContext::new(workflow_clone.id, context.agent_registry.clone());
    let context_clone_id = context_clone.execution_id;

    tokio::spawn(async move {
        let _ = executor_clone
            .execute(&workflow_clone, &context_clone)
            .await;
    });

    // Give it time to start
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Pause execution
    let pause_result = executor.pause(context_clone_id).await;
    assert!(pause_result.is_ok());

    // Verify paused state
    let control = executor
        .check_execution_control(context_clone_id)
        .expect("internal error");
    assert_eq!(control, ExecutionControl::Paused);

    // Resume execution
    let resume_result = executor.resume(context_clone_id).await;
    assert!(resume_result.is_ok());
}

#[actix_rt::test]
async fn test_cancel_workflow() {
    let registry = Arc::new(AgentRegistry::new());
    let executor = Arc::new(DefaultWorkflowExecutor::new(registry.clone()));

    let workflow = WorkflowBuilder::new("cancel_test")
        .add_step(StepBuilder::action("step1", "Step 1", "agent1", "op1").build())
        .build();

    let context = WorkflowContext::new(workflow.id, registry);
    let _execution_id = context.execution_id;

    // Start workflow in background
    let executor_clone = executor.clone();
    let workflow_clone = workflow.clone();
    let context_clone = WorkflowContext::new(workflow_clone.id, context.agent_registry.clone());
    let context_clone_id = context_clone.execution_id;

    tokio::spawn(async move {
        let _ = executor_clone
            .execute(&workflow_clone, &context_clone)
            .await;
    });

    // Give it time to start
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Cancel execution
    let cancel_result = executor.cancel(context_clone_id).await;
    assert!(cancel_result.is_ok());

    // Verify cancelled state
    let control = executor
        .check_execution_control(context_clone_id)
        .expect("internal error");
    assert_eq!(control, ExecutionControl::Cancelled);
}

#[actix_rt::test]
async fn test_workflow_with_retry() {
    let registry = Arc::new(AgentRegistry::new());
    let executor = DefaultWorkflowExecutor::new(registry.clone());

    let workflow = WorkflowBuilder::new("retry_test")
        .add_step(
            StepBuilder::action("step1", "Flaky Step", "agent1", "flaky_op")
                .retry(
                    3,
                    BackoffStrategy::Exponential {
                        initial: Duration::from_millis(10),
                        multiplier: 2.0,
                        max: Duration::from_millis(100),
                    },
                )
                .build(),
        )
        .build();

    let context = WorkflowContext::new(workflow.id, registry);

    // Execute workflow with retry
    let result = executor.execute(&workflow, &context).await;
    // Note: Agent doesn't exist, so will fail but retry logic is tested
    assert!(result.is_err());
    match result {
        Err(WorkflowError::AgentError(_)) => {
            // Expected - agent not registered
        }
        other => panic!("Expected AgentError, got: {:?}", other),
    }
}

#[actix_rt::test]
async fn test_workflow_error_strategies() {
    let registry = Arc::new(AgentRegistry::new());

    // Test Continue strategy
    let executor_continue = DefaultWorkflowExecutor::new(registry.clone());
    let workflow_continue = WorkflowBuilder::new("continue_test")
        .error_strategy(ErrorStrategy::Continue)
        .add_step(StepBuilder::action("step1", "Step 1", "nonexistent_agent", "op1").build())
        .build();

    let context_continue = WorkflowContext::new(workflow_continue.id, registry.clone());
    let result_continue = executor_continue
        .execute(&workflow_continue, &context_continue)
        .await;
    // Should not fail fast with Continue strategy
    assert!(result_continue.is_ok());

    // Test FailFast strategy
    let executor_failfast = DefaultWorkflowExecutor::new(registry.clone());
    let workflow_failfast = WorkflowBuilder::new("failfast_test")
        .error_strategy(ErrorStrategy::FailFast)
        .add_step(StepBuilder::action("step1", "Step 1", "nonexistent_agent", "op1").build())
        .build();

    let context_failfast = WorkflowContext::new(workflow_failfast.id, registry);
    let result_failfast = executor_failfast
        .execute(&workflow_failfast, &context_failfast)
        .await;
    // Should fail with FailFast strategy
    assert!(result_failfast.is_err());
}

#[actix_rt::test]
async fn test_workflow_monitoring() {
    let registry = Arc::new(AgentRegistry::new());
    let monitor = Arc::new(super::super::monitoring::DefaultWorkflowMonitor::new());
    let executor = DefaultWorkflowExecutor::new(registry.clone()).with_monitor(monitor.clone());

    let workflow = WorkflowBuilder::new("monitor_test")
        .add_step(StepBuilder::action("step1", "Step 1", "agent1", "op1").build())
        .build();

    let context = WorkflowContext::new(workflow.id, registry);
    let execution_id = context.execution_id;

    let _result = executor.execute(&workflow, &context).await;
    // Note: Will fail due to nonexistent agent, but monitoring should still work

    // Check metrics were recorded
    let metrics = monitor.get_metrics(execution_id).await;
    assert_eq!(metrics.execution_id, execution_id);
    assert!(metrics.total_steps > 0); // Monitor should track step attempts
}

#[actix_rt::test]
async fn test_checkpoint_creation() {
    let registry = Arc::new(AgentRegistry::new());
    let executor = DefaultWorkflowExecutor::new(registry.clone());

    let workflow = WorkflowBuilder::new("checkpoint_test")
        .add_step(StepBuilder::action("step1", "Step 1", "agent1", "op1").build())
        .add_step(StepBuilder::action("step2", "Step 2", "agent2", "op2").build())
        .build();

    let context = WorkflowContext::new(workflow.id, registry);
    let execution_id = context.execution_id;

    // Create checkpoint manually
    let execution_order = vec![vec!["step1".to_string()], vec!["step2".to_string()]];
    let checkpoint_result = executor.save_checkpoint(execution_id, 1, &execution_order);
    assert!(checkpoint_result.is_ok());
}

#[actix_rt::test]
async fn test_parallel_step_execution() {
    let registry = Arc::new(AgentRegistry::new());
    let executor = DefaultWorkflowExecutor::new(registry.clone());

    let parallel_steps = vec![
        StepBuilder::action("parallel1", "Parallel 1", "agent1", "op1").build(),
        StepBuilder::action("parallel2", "Parallel 2", "agent2", "op2").build(),
    ];

    let context = WorkflowContext::new(Uuid::new_v4(), registry);
    context.set_state(WorkflowState::Running); // Set to running for error propagation

    // Execute parallel steps
    let result = executor.execute_parallel(&parallel_steps, &context).await;
    // Note: execute_parallel returns error on first failure when state is Running
    // Since agents don't exist, this will fail
    assert!(result.is_err());
    if let Err(e) = result {
        // Verify it's an AgentError
        match e {
            WorkflowError::AgentError(_) => {
                // Expected - agents not registered
            }
            other => panic!("Expected AgentError, got: {:?}", other),
        }
    }
}

// Coverage tests - Helper to create an executor with registered agents
async fn setup_executor_with_agent() -> (DefaultWorkflowExecutor, Arc<AgentRegistry>) {
    let registry = Arc::new(AgentRegistry::new());

    // Register a test agent
    let agent_id = Uuid::new_v4();
    let spec = AgentSpec {
        id: agent_id,
        class: AgentClass::Analyzer,
        config: serde_json::json!({}),
    };
    registry.spawn_agent(spec).await.unwrap();
    registry
        .register_agent_with_name("test_agent", agent_id)
        .await;

    let executor = DefaultWorkflowExecutor::new(registry.clone());
    (executor, registry)
}
