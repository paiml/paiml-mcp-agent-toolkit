// Tests for execute_step_internal: skip/fail conditions, wait steps,
// subworkflow steps, and all ErrorHandler variants.

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
