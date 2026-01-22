//! Tests for workflow executor
//! Extracted to separate file for file health compliance (CB-040)

use super::*;
use crate::agents::{AgentClass, AgentRegistry, AgentSpec};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use super::*;

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
// Coverage tests
use crate::agents::{AgentClass, AgentSpec};

// Helper to create an executor with registered agents
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

// ===== execute_sequence tests =====

#[actix_rt::test]
async fn test_execute_sequence_empty_steps() {
    let registry = Arc::new(AgentRegistry::new());
    let executor = DefaultWorkflowExecutor::new(registry.clone());
    let context = WorkflowContext::new(Uuid::new_v4(), registry);

    let result = executor.execute_sequence(&[], &context).await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.is_object());
}

#[actix_rt::test]
async fn test_execute_sequence_with_registered_agent() {
    let (executor, registry) = setup_executor_with_agent().await;
    let context = WorkflowContext::new(Uuid::new_v4(), registry);

    let steps = vec![
        StepBuilder::action("step1", "Step 1", "test_agent", "op1").build(),
        StepBuilder::action("step2", "Step 2", "test_agent", "op2").build(),
    ];

    let result = executor.execute_sequence(&steps, &context).await;

    assert!(result.is_ok());
    // Last step's output should be returned
    let output = result.unwrap();
    assert_eq!(output["operation"], "op2");
}

#[actix_rt::test]
async fn test_execute_sequence_stops_on_error() {
    let registry = Arc::new(AgentRegistry::new());
    let executor = DefaultWorkflowExecutor::new(registry.clone());
    let context = WorkflowContext::new(Uuid::new_v4(), registry);

    let steps = vec![
        StepBuilder::action("step1", "Step 1", "nonexistent", "op1").build(),
        StepBuilder::action("step2", "Step 2", "nonexistent", "op2").build(),
    ];

    let result = executor.execute_sequence(&steps, &context).await;

    // Should fail on first step
    assert!(result.is_err());
}

// ===== execute_conditional tests =====

#[actix_rt::test]
async fn test_execute_conditional_true_branch() {
    let (executor, registry) = setup_executor_with_agent().await;
    let context = WorkflowContext::new(Uuid::new_v4(), registry);
    context.set_variable("value".to_string(), serde_json::json!(10));

    let if_true = StepBuilder::action("true_step", "True Step", "test_agent", "true_op").build();
    let if_false = Some(Box::new(
        StepBuilder::action("false_step", "False Step", "test_agent", "false_op").build(),
    ));

    // This condition should evaluate to true (default)
    let result = executor
        .execute_conditional("true", &if_true, &if_false, &context)
        .await;

    assert!(result.is_ok());
}

#[actix_rt::test]
async fn test_execute_conditional_no_else_branch() {
    let (executor, registry) = setup_executor_with_agent().await;
    let context = WorkflowContext::new(Uuid::new_v4(), registry);
    context.set_variable("x".to_string(), serde_json::json!(5));
    context.set_variable("y".to_string(), serde_json::json!(10));

    let if_true = StepBuilder::action("true_step", "True Step", "test_agent", "op").build();

    // x > y should be false, and there's no else branch
    let result = executor
        .execute_conditional("x > y", &if_true, &None, &context)
        .await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert_eq!(output["skipped"], true);
}

// ===== execute_loop tests =====

#[actix_rt::test]
async fn test_execute_loop_max_iterations() {
    let (executor, registry) = setup_executor_with_agent().await;
    let context = WorkflowContext::new(Uuid::new_v4(), registry);

    let step = StepBuilder::action("loop_step", "Loop Step", "test_agent", "op").build();

    // Condition will always be true (default), but max_iterations limits it
    let result = executor
        .execute_loop("true", &step, Some(3), &context)
        .await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert_eq!(output["iterations"], 3);
    assert_eq!(output["outputs"].as_array().unwrap().len(), 3);
}

#[actix_rt::test]
async fn test_execute_loop_false_condition() {
    let (executor, registry) = setup_executor_with_agent().await;
    let context = WorkflowContext::new(Uuid::new_v4(), registry);
    context.set_variable("x".to_string(), serde_json::json!(1));
    context.set_variable("y".to_string(), serde_json::json!(10));

    let step = StepBuilder::action("loop_step", "Loop Step", "test_agent", "op").build();

    // x > y is false, so loop should not execute
    let result = executor.execute_loop("x > y", &step, None, &context).await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert_eq!(output["iterations"], 0);
}

// ===== execute_with_retry tests =====

#[actix_rt::test]
async fn test_execute_with_retry_success_first_attempt() {
    let (executor, registry) = setup_executor_with_agent().await;
    let context = WorkflowContext::new(Uuid::new_v4(), registry);

    let step = StepBuilder::action("retry_step", "Retry Step", "test_agent", "op").build();
    let retry = RetryPolicy {
        max_attempts: 3,
        backoff: BackoffStrategy::Fixed {
            delay: Duration::from_millis(10),
        },
        retry_on: vec![],
    };

    let result = executor.execute_with_retry(&step, &context, &retry).await;

    assert!(result.is_ok());
}

#[actix_rt::test]
async fn test_execute_with_retry_exhausts_attempts() {
    let registry = Arc::new(AgentRegistry::new());
    let executor = DefaultWorkflowExecutor::new(registry.clone());
    let context = WorkflowContext::new(Uuid::new_v4(), registry);

    let step = StepBuilder::action("retry_step", "Retry Step", "nonexistent", "op").build();
    let retry = RetryPolicy {
        max_attempts: 2,
        backoff: BackoffStrategy::Fixed {
            delay: Duration::from_millis(1),
        },
        retry_on: vec![],
    };

    let result = executor.execute_with_retry(&step, &context, &retry).await;

    assert!(result.is_err());
}

// ===== execute_step_internal tests =====

#[actix_rt::test]
async fn test_execute_step_internal_with_skip_condition() {
    let (executor, registry) = setup_executor_with_agent().await;
    let context = WorkflowContext::new(Uuid::new_v4(), registry);
    context.set_variable("x".to_string(), serde_json::json!(1));
    context.set_variable("y".to_string(), serde_json::json!(10));

    let step = StepBuilder::action("cond_step", "Conditional Step", "test_agent", "op")
        .condition("x > y", true) // skip_on_false = true
        .build();

    let result = executor.execute_step_internal(&step, &context).await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert_eq!(output["skipped"], true);
}

#[actix_rt::test]
async fn test_execute_step_internal_with_fail_condition() {
    let (executor, registry) = setup_executor_with_agent().await;
    let context = WorkflowContext::new(Uuid::new_v4(), registry);
    context.set_variable("x".to_string(), serde_json::json!(1));
    context.set_variable("y".to_string(), serde_json::json!(10));

    let step = StepBuilder::action("cond_step", "Conditional Step", "test_agent", "op")
        .condition("x > y", false) // skip_on_false = false, should fail
        .build();

    let result = executor.execute_step_internal(&step, &context).await;

    assert!(result.is_err());
    match result {
        Err(WorkflowError::ConditionError(msg)) => {
            assert!(msg.contains("Step condition failed"));
        }
        _ => panic!("Expected ConditionError"),
    }
}

#[actix_rt::test]
async fn test_execute_step_internal_wait_step() {
    let registry = Arc::new(AgentRegistry::new());
    let executor = DefaultWorkflowExecutor::new(registry.clone());
    let context = WorkflowContext::new(Uuid::new_v4(), registry);

    let step = WorkflowStep {
        id: "wait_step".to_string(),
        name: "Wait Step".to_string(),
        step_type: StepType::Wait {
            duration: Duration::from_millis(10),
        },
        condition: None,
        retry: None,
        timeout: None,
        on_error: None,
        metadata: HashMap::new(),
    };

    let start = std::time::Instant::now();
    let result = executor.execute_step_internal(&step, &context).await;
    let elapsed = start.elapsed();

    assert!(result.is_ok());
    assert!(elapsed >= Duration::from_millis(10));
}

#[actix_rt::test]
async fn test_execute_step_internal_subworkflow() {
    let registry = Arc::new(AgentRegistry::new());
    let executor = DefaultWorkflowExecutor::new(registry.clone());
    let context = WorkflowContext::new(Uuid::new_v4(), registry);

    let params = serde_json::json!({"param1": "value1"});
    let step = WorkflowStep {
        id: "subworkflow_step".to_string(),
        name: "Subworkflow Step".to_string(),
        step_type: StepType::SubWorkflow {
            workflow_id: Uuid::new_v4(),
            params: params.clone(),
        },
        condition: None,
        retry: None,
        timeout: None,
        on_error: None,
        metadata: HashMap::new(),
    };

    let result = executor.execute_step_internal(&step, &context).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), params);
}

#[actix_rt::test]
async fn test_execute_step_internal_with_error_handler_skip() {
    let registry = Arc::new(AgentRegistry::new());
    let executor = DefaultWorkflowExecutor::new(registry.clone());
    let context = WorkflowContext::new(Uuid::new_v4(), registry);

    let step = StepBuilder::action("error_step", "Error Step", "nonexistent", "op")
        .on_error(ErrorHandler::Skip)
        .build();

    let result = executor.execute_step_internal(&step, &context).await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert_eq!(output["skipped"], true);
}

#[actix_rt::test]
async fn test_execute_step_internal_with_error_handler_fail() {
    let registry = Arc::new(AgentRegistry::new());
    let executor = DefaultWorkflowExecutor::new(registry.clone());
    let context = WorkflowContext::new(Uuid::new_v4(), registry);

    let step = StepBuilder::action("error_step", "Error Step", "nonexistent", "op")
        .on_error(ErrorHandler::Fail)
        .build();

    let result = executor.execute_step_internal(&step, &context).await;

    assert!(result.is_err());
}

#[actix_rt::test]
async fn test_execute_step_internal_with_error_handler_goto() {
    let registry = Arc::new(AgentRegistry::new());
    let executor = DefaultWorkflowExecutor::new(registry.clone());
    let context = WorkflowContext::new(Uuid::new_v4(), registry);

    let step = StepBuilder::action("error_step", "Error Step", "nonexistent", "op")
        .on_error(ErrorHandler::Goto {
            step_id: "recovery_step".to_string(),
        })
        .build();

    let result = executor.execute_step_internal(&step, &context).await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert_eq!(output["goto"], "recovery_step");
}

#[actix_rt::test]
async fn test_execute_step_internal_with_error_handler_compensate() {
    let registry = Arc::new(AgentRegistry::new());
    let executor = DefaultWorkflowExecutor::new(registry.clone());
    let context = WorkflowContext::new(Uuid::new_v4(), registry);

    let step = StepBuilder::action("error_step", "Error Step", "nonexistent", "op")
        .on_error(ErrorHandler::Compensate {
            steps: vec!["comp1".to_string(), "comp2".to_string()],
        })
        .build();

    let result = executor.execute_step_internal(&step, &context).await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output["compensated"].as_array().is_some());
}

#[actix_rt::test]
async fn test_execute_step_internal_with_error_handler_execute() {
    let (executor, registry) = setup_executor_with_agent().await;
    let context = WorkflowContext::new(Uuid::new_v4(), registry);

    let fallback_step =
        Box::new(StepBuilder::action("fallback", "Fallback", "test_agent", "fallback_op").build());

    let step = StepBuilder::action("error_step", "Error Step", "nonexistent", "op")
        .on_error(ErrorHandler::Execute {
            step: fallback_step,
        })
        .build();

    let result = executor.execute_step_internal(&step, &context).await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert_eq!(output["operation"], "fallback_op");
}

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
