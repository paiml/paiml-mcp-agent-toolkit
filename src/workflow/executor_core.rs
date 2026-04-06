// Core execution methods for DefaultWorkflowExecutor
// Extracted from executor.rs for file health compliance

impl DefaultWorkflowExecutor {
    pub fn new(agent_registry: Arc<AgentRegistry>) -> Self {
        Self {
            agent_registry,
            monitor: None,
            execution_states: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_monitor(mut self, monitor: Arc<dyn WorkflowMonitor>) -> Self {
        self.monitor = Some(monitor);
        self
    }

    async fn execute_action(
        &self,
        agent_name: &str,
        operation: &str,
        params: &Value,
        context: &WorkflowContext,
    ) -> Result<Value, WorkflowError> {
        debug_assert!(!agent_name.is_empty(), "agent_name must not be empty");
        debug_assert!(!operation.is_empty(), "operation must not be empty");
        // Get the agent ID from registry
        let agent_id = self
            .agent_registry
            .get_agent(agent_name)
            .await
            .ok_or_else(|| WorkflowError::AgentError(format!("Agent not found: {}", agent_name)))?;

        // Get agent spec for validation
        let _agent_spec = self
            .agent_registry
            .get_agent_spec(agent_id)
            .await
            .ok_or_else(|| {
                WorkflowError::AgentError(format!("Agent spec not found for: {}", agent_name))
            })?;

        // Build response with agent execution details
        // Note: Actual agent execution will be implemented when agent actors are available
        Ok(serde_json::json!({
            "agent_id": agent_id.to_string(),
            "agent_name": agent_name,
            "operation": operation,
            "params": params,
            "workflow_context": {
                "workflow_id": context.workflow_id.to_string(),
                "execution_id": context.execution_id.to_string(),
            },
            "status": "agent_execution_pending",
            "message": "Agent actor execution will be implemented in next phase"
        }))
    }

    async fn execute_parallel(
        &self,
        steps: &[WorkflowStep],
        context: &WorkflowContext,
    ) -> Result<Value, WorkflowError> {
        debug_assert!(!steps.is_empty(), "steps must not be empty");
        let futures = steps.iter().map(|step| self.execute_step(step, context));

        let results = join_all(futures).await;

        // Collect results and errors
        let mut outputs = vec![];
        for result in results.into_iter() {
            match result {
                Ok(output) => outputs.push(output),
                Err(e) => {
                    // Check error strategy
                    match context.get_state() {
                        WorkflowState::Running => return Err(e),
                        _ => {}
                    }
                }
            }
        }

        Ok(serde_json::json!({ "results": outputs }))
    }

    async fn execute_sequence(
        &self,
        steps: &[WorkflowStep],
        context: &WorkflowContext,
    ) -> Result<Value, WorkflowError> {
        debug_assert!(!steps.is_empty(), "steps must not be empty");
        let mut last_output = serde_json::json!({});

        for step in steps {
            last_output = self.execute_step(step, context).await?;
        }

        Ok(last_output)
    }

    async fn execute_conditional(
        &self,
        condition: &str,
        if_true: &WorkflowStep,
        if_false: &Option<Box<WorkflowStep>>,
        context: &WorkflowContext,
    ) -> Result<Value, WorkflowError> {
        debug_assert!(!condition.is_empty(), "condition must not be empty");
        // Evaluate condition
        let result = self.evaluate_condition(condition, context).await?;

        if result {
            self.execute_step(if_true, context).await
        } else if let Some(else_step) = if_false {
            self.execute_step(else_step, context).await
        } else {
            Ok(serde_json::json!({ "skipped": true }))
        }
    }

    async fn execute_loop(
        &self,
        condition: &str,
        step: &WorkflowStep,
        max_iterations: Option<usize>,
        context: &WorkflowContext,
    ) -> Result<Value, WorkflowError> {
        debug_assert!(!condition.is_empty(), "condition must not be empty");
        let mut iteration = 0;
        let mut outputs = vec![];

        while self.evaluate_condition(condition, context).await? {
            if let Some(max) = max_iterations {
                if iteration >= max {
                    break;
                }
            }

            let output = self.execute_step(step, context).await?;
            outputs.push(output);
            iteration += 1;
        }

        Ok(serde_json::json!({ "iterations": iteration, "outputs": outputs }))
    }

    async fn execute_with_retry(
        &self,
        step: &WorkflowStep,
        context: &WorkflowContext,
        retry: &RetryPolicy,
    ) -> Result<Value, WorkflowError> {
        let mut attempts = 0;
        let mut last_error = None;

        while attempts < retry.max_attempts {
            match self.execute_step_internal(step, context).await {
                Ok(output) => return Ok(output),
                Err(e) => {
                    last_error = Some(e);
                    attempts += 1;

                    if attempts < retry.max_attempts {
                        let delay = self.calculate_backoff(&retry.backoff, attempts);
                        sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or(WorkflowError::MaxRetriesExceeded))
    }

    async fn execute_step_internal(
        &self,
        step: &WorkflowStep,
        context: &WorkflowContext,
    ) -> Result<Value, WorkflowError> {
        // Check step condition
        if let Some(condition) = &step.condition {
            let should_execute = self
                .evaluate_condition(&condition.expression, context)
                .await?;
            if !should_execute {
                if condition.skip_on_false {
                    return Ok(serde_json::json!({ "skipped": true }));
                } else {
                    return Err(WorkflowError::ConditionError(format!(
                        "Step condition failed: {}",
                        condition.expression
                    )));
                }
            }
        }

        // Execute based on step type
        let result = match &step.step_type {
            StepType::Action {
                agent,
                operation,
                params,
            } => self.execute_action(agent, operation, params, context).await,
            StepType::Parallel { steps } => self.execute_parallel(steps, context).await,
            StepType::Sequence { steps } => self.execute_sequence(steps, context).await,
            StepType::Conditional {
                condition,
                if_true,
                if_false,
            } => {
                self.execute_conditional(condition, if_true, if_false, context)
                    .await
            }
            StepType::Loop {
                condition,
                step,
                max_iterations,
            } => {
                self.execute_loop(condition, step, *max_iterations, context)
                    .await
            }
            StepType::Wait { duration } => {
                sleep(*duration).await;
                Ok(serde_json::json!({ "waited": duration.as_secs() }))
            }
            StepType::SubWorkflow {
                workflow_id: _workflow_id,
                params,
            } => {
                // Would recursively execute sub-workflow
                Ok(params.clone())
            }
        };

        // Handle errors
        match result {
            Ok(output) => Ok(output),
            Err(e) => {
                if let Some(handler) = &step.on_error {
                    self.handle_error(handler, &e, context).await
                } else {
                    Err(e)
                }
            }
        }
    }
}
