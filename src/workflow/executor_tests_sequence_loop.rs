// Tests for execute_sequence, execute_conditional, execute_loop,
// and execute_with_retry methods.

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
