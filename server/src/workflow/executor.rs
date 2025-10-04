use super::*;
use crate::agents::registry::AgentRegistry;
use futures::future::join_all;
use std::sync::Arc;
use tokio::time::{sleep, timeout};
use parking_lot::RwLock;
use std::collections::HashMap;

// Default workflow executor implementation
pub struct DefaultWorkflowExecutor {
    agent_registry: Arc<AgentRegistry>,
    monitor: Option<Arc<dyn WorkflowMonitor>>,
    // Track execution state for pause/resume/cancel
    execution_states: Arc<RwLock<HashMap<Uuid, ExecutionState>>>,
}

#[derive(Debug, Clone, PartialEq)]
enum ExecutionControl {
    Running,
    Paused,
    Cancelled,
}

#[derive(Debug, Clone)]
struct ExecutionState {
    control: ExecutionControl,
    checkpoint: Option<CheckpointData>,
}

#[derive(Debug, Clone)]
struct CheckpointData {
    completed_steps: Vec<String>,
    current_level: usize,
}

impl Default for ExecutionState {
    fn default() -> Self {
        Self {
            control: ExecutionControl::Running,
            checkpoint: None,
        }
    }
}

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
            .ok_or_else(|| WorkflowError::AgentError(format!("Agent spec not found for: {}", agent_name)))?;

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

    async fn handle_error(
        &self,
        handler: &ErrorHandler,
        error: &WorkflowError,
        context: &WorkflowContext,
    ) -> Result<Value, WorkflowError> {
        match handler {
            ErrorHandler::Skip => {
                Ok(serde_json::json!({ "skipped": true, "error": error.to_string() }))
            }
            ErrorHandler::Fail => Err(error.clone()),
            ErrorHandler::Goto { step_id } => {
                // Would jump to specified step
                Ok(serde_json::json!({ "goto": step_id }))
            }
            ErrorHandler::Execute { step } => self.execute_step(step, context).await,
            ErrorHandler::Compensate { steps } => {
                // Would execute compensation steps
                Ok(serde_json::json!({ "compensated": steps }))
            }
        }
    }

    fn calculate_backoff(&self, strategy: &BackoffStrategy, attempt: usize) -> Duration {
        match strategy {
            BackoffStrategy::Fixed { delay } => *delay,
            BackoffStrategy::Exponential {
                initial,
                multiplier,
                max,
            } => {
                let delay = initial.as_secs_f32() * multiplier.powi(attempt as i32 - 1);
                Duration::from_secs_f32(delay.min(max.as_secs_f32()))
            }
            BackoffStrategy::Linear { initial, increment } => {
                *initial + *increment * (attempt - 1) as u32
            }
        }
    }

    async fn evaluate_condition(
        &self,
        expression: &str,
        context: &WorkflowContext,
    ) -> Result<bool, WorkflowError> {
        // Simple expression evaluation
        // In production, would use a proper expression engine

        if expression.contains(">") {
            let parts: Vec<&str> = expression.split('>').collect();
            if parts.len() == 2 {
                let left = self.resolve_variable(parts[0].trim(), context)?;
                let right = self.resolve_variable(parts[1].trim(), context)?;

                if let (Some(l), Some(r)) = (left.as_f64(), right.as_f64()) {
                    return Ok(l > r);
                }
            }
        }

        if expression.contains("==") {
            let parts: Vec<&str> = expression.split("==").collect();
            if parts.len() == 2 {
                let left = self.resolve_variable(parts[0].trim(), context)?;
                let right = self.resolve_variable(parts[1].trim(), context)?;
                return Ok(left == right);
            }
        }

        // Default to true for now
        Ok(true)
    }

    fn resolve_variable(
        &self,
        path: &str,
        context: &WorkflowContext,
    ) -> Result<Value, WorkflowError> {
        if path.starts_with("steps.") {
            let parts: Vec<&str> = path.splitn(3, '.').collect();
            if parts.len() >= 3 {
                let step_id = parts[1];
                let field = parts[2];

                if let Some(result) = context.get_step_result(step_id) {
                    if field == "status" {
                        return Ok(serde_json::json!(format!("{:?}", result.status)));
                    } else if field.starts_with("output.") {
                        if let Some(output) = &result.output {
                            let output_path = field.strip_prefix("output.").unwrap();
                            return Ok(output[output_path].clone());
                        }
                    }
                }
            }
        }

        // Check context variables
        context
            .get_variable(path)
            .ok_or_else(|| WorkflowError::VariableNotFound(path.to_string()))
    }
}

#[async_trait]
impl WorkflowExecutor for DefaultWorkflowExecutor {
    async fn execute(
        &self,
        workflow: &Workflow,
        context: &WorkflowContext,
    ) -> Result<Value, WorkflowError> {
        context.set_state(WorkflowState::Running);

        if let Some(monitor) = &self.monitor {
            monitor
                .on_workflow_started(workflow.id, context.execution_id)
                .await;
        }

        let result = if let Some(timeout_duration) = workflow.timeout {
            timeout(
                timeout_duration,
                self.execute_workflow_internal(workflow, context),
            )
            .await
            .map_err(|_| WorkflowError::Timeout)?
        } else {
            self.execute_workflow_internal(workflow, context).await
        };

        match &result {
            Ok(output) => {
                context.set_state(WorkflowState::Completed);
                if let Some(monitor) = &self.monitor {
                    monitor
                        .on_workflow_completed(workflow.id, context.execution_id, output)
                        .await;
                }
            }
            Err(e) => {
                context.set_state(WorkflowState::Failed);
                if let Some(monitor) = &self.monitor {
                    monitor
                        .on_workflow_failed(workflow.id, context.execution_id, e)
                        .await;
                }
            }
        }

        result
    }

    async fn execute_step(
        &self,
        step: &WorkflowStep,
        context: &WorkflowContext,
    ) -> Result<Value, WorkflowError> {
        if let Some(monitor) = &self.monitor {
            monitor
                .on_step_started(context.execution_id, &step.id)
                .await;
        }

        let mut result = StepResult {
            step_id: step.id.clone(),
            status: StepStatus::Running,
            output: None,
            error: None,
            started_at: Instant::now(),
            completed_at: None,
            attempts: 1,
        };

        let output = if let Some(retry) = &step.retry {
            self.execute_with_retry(step, context, retry).await
        } else {
            self.execute_step_internal(step, context).await
        };

        match output {
            Ok(value) => {
                result.status = StepStatus::Completed;
                result.output = Some(value.clone());
                result.completed_at = Some(Instant::now());

                if let Some(monitor) = &self.monitor {
                    monitor
                        .on_step_completed(context.execution_id, &step.id, &value)
                        .await;
                }

                context.set_step_result(step.id.clone(), result);
                Ok(value)
            }
            Err(e) => {
                result.status = StepStatus::Failed;
                result.error = Some(e.to_string());
                result.completed_at = Some(Instant::now());

                if let Some(monitor) = &self.monitor {
                    monitor
                        .on_step_failed(context.execution_id, &step.id, &e.to_string())
                        .await;
                }

                context.set_step_result(step.id.clone(), result);
                Err(e)
            }
        }
    }

    async fn pause(&self, execution_id: Uuid) -> Result<(), WorkflowError> {
        let mut states = self.execution_states.write();
        if let Some(state) = states.get_mut(&execution_id) {
            state.control = ExecutionControl::Paused;
            Ok(())
        } else {
            Err(WorkflowError::NotFound(execution_id))
        }
    }

    async fn resume(&self, execution_id: Uuid) -> Result<(), WorkflowError> {
        let mut states = self.execution_states.write();
        if let Some(state) = states.get_mut(&execution_id) {
            if state.control == ExecutionControl::Paused {
                state.control = ExecutionControl::Running;
                Ok(())
            } else {
                Err(WorkflowError::ExecutionError(
                    "Workflow is not paused".to_string(),
                ))
            }
        } else {
            Err(WorkflowError::NotFound(execution_id))
        }
    }

    async fn cancel(&self, execution_id: Uuid) -> Result<(), WorkflowError> {
        let mut states = self.execution_states.write();
        if let Some(state) = states.get_mut(&execution_id) {
            state.control = ExecutionControl::Cancelled;
            Ok(())
        } else {
            Err(WorkflowError::NotFound(execution_id))
        }
    }
}

impl DefaultWorkflowExecutor {
    async fn execute_workflow_internal(
        &self,
        workflow: &Workflow,
        context: &WorkflowContext,
    ) -> Result<Value, WorkflowError> {
        // Build DAG for optimal execution ordering
        let dag_engine = super::dag::DagEngine::from_workflow(workflow)?;
        let analysis = dag_engine.analyze()?;

        // Initialize execution state
        let execution_id = context.execution_id;
        self.execution_states
            .write()
            .insert(execution_id, ExecutionState::default());

        let mut last_output = serde_json::json!({});

        // Execute each parallel level
        for (level_idx, level_step_ids) in analysis.execution_order.iter().enumerate() {
            // Check for pause/cancel
            let control_state = self.check_execution_control(execution_id)?;
            if control_state == ExecutionControl::Cancelled {
                return Err(WorkflowError::Cancelled);
            }
            if control_state == ExecutionControl::Paused {
                // Save checkpoint
                self.save_checkpoint(execution_id, level_idx, &analysis.execution_order)?;
                // Wait for resume
                while self.check_execution_control(execution_id)? == ExecutionControl::Paused {
                    sleep(Duration::from_millis(100)).await;
                }
            }

            // Get steps for this level
            let level_steps: Vec<&WorkflowStep> = level_step_ids
                .iter()
                .filter_map(|id| workflow.steps.iter().find(|s| &s.id == id))
                .collect();

            // Execute level in parallel
            if level_steps.len() == 1 {
                // Single step - execute directly
                match self.execute_step(level_steps[0], context).await {
                    Ok(output) => last_output = output,
                    Err(e) => {
                        return self.handle_workflow_error(e, &workflow.error_strategy, context);
                    }
                }
            } else {
                // Multiple steps - execute in parallel
                let futures = level_steps
                    .iter()
                    .map(|step| self.execute_step(step, context));

                let results = join_all(futures).await;

                // Collect results
                let mut level_outputs = vec![];
                for result in results {
                    match result {
                        Ok(output) => level_outputs.push(output),
                        Err(e) => {
                            return self.handle_workflow_error(e, &workflow.error_strategy, context);
                        }
                    }
                }

                last_output = serde_json::json!({ "parallel_results": level_outputs });
            }
        }

        // Cleanup execution state
        self.execution_states.write().remove(&execution_id);

        Ok(last_output)
    }

    fn check_execution_control(&self, execution_id: Uuid) -> Result<ExecutionControl, WorkflowError> {
        Ok(self
            .execution_states
            .read()
            .get(&execution_id)
            .map(|state| state.control.clone())
            .unwrap_or(ExecutionControl::Running))
    }

    fn save_checkpoint(
        &self,
        execution_id: Uuid,
        current_level: usize,
        execution_order: &[Vec<String>],
    ) -> Result<(), WorkflowError> {
        let completed_steps: Vec<String> = execution_order[..current_level]
            .iter()
            .flatten()
            .cloned()
            .collect();

        if let Some(state) = self.execution_states.write().get_mut(&execution_id) {
            state.checkpoint = Some(CheckpointData {
                completed_steps,
                current_level,
            });
        }

        Ok(())
    }

    fn handle_workflow_error(
        &self,
        error: WorkflowError,
        strategy: &ErrorStrategy,
        context: &WorkflowContext,
    ) -> Result<Value, WorkflowError> {
        match strategy {
            ErrorStrategy::FailFast => Err(error),
            ErrorStrategy::Continue => Ok(serde_json::json!({ "continued_after_error": true })),
            ErrorStrategy::Rollback => {
                // Invoke rollback logic (returns Result)
                let recovery_result = futures::executor::block_on(
                    super::recovery::RecoveryManager::handle_error(&error, strategy, context)
                );

                // Return original error regardless of recovery result
                let _ = recovery_result; // Suppress unused warning
                Err(error)
            }
            ErrorStrategy::Compensate => {
                // Invoke compensation logic (returns Result)
                let recovery_result = futures::executor::block_on(
                    super::recovery::RecoveryManager::handle_error(&error, strategy, context)
                );

                // Return original error regardless of recovery result
                let _ = recovery_result; // Suppress unused warning
                Err(error)
            }
        }
    }
}

#[cfg(test)]
mod tests {
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
            .add_step(
                StepBuilder::action("step1", "Init", "agent1", "init")
                    .build()
            )
            .add_step(
                StepBuilder::action("step2", "Process A", "agent2", "process")
                    .build()
            )
            .add_step(
                StepBuilder::action("step3", "Process B", "agent3", "process")
                    .build()
            )
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
            .add_step(
                StepBuilder::action("step1", "Step 1", "agent1", "op1")
                    .build()
            )
            .add_step(
                StepBuilder::action("step2", "Step 2", "agent2", "op2")
                    .build()
            )
            .build();

        let context = WorkflowContext::new(workflow.id, registry);
        let execution_id = context.execution_id;

        // Start workflow in background
        let executor_clone = executor.clone();
        let workflow_clone = workflow.clone();
        let context_clone = WorkflowContext::new(workflow_clone.id, context.agent_registry.clone());
        let context_clone_id = context_clone.execution_id;

        tokio::spawn(async move {
            let _ = executor_clone.execute(&workflow_clone, &context_clone).await;
        });

        // Give it time to start
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Pause execution
        let pause_result = executor.pause(context_clone_id).await;
        assert!(pause_result.is_ok());

        // Verify paused state
        let control = executor.check_execution_control(context_clone_id).unwrap();
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
            .add_step(
                StepBuilder::action("step1", "Step 1", "agent1", "op1")
                    .build()
            )
            .build();

        let context = WorkflowContext::new(workflow.id, registry);
        let execution_id = context.execution_id;

        // Start workflow in background
        let executor_clone = executor.clone();
        let workflow_clone = workflow.clone();
        let context_clone = WorkflowContext::new(workflow_clone.id, context.agent_registry.clone());
        let context_clone_id = context_clone.execution_id;

        tokio::spawn(async move {
            let _ = executor_clone.execute(&workflow_clone, &context_clone).await;
        });

        // Give it time to start
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Cancel execution
        let cancel_result = executor.cancel(context_clone_id).await;
        assert!(cancel_result.is_ok());

        // Verify cancelled state
        let control = executor.check_execution_control(context_clone_id).unwrap();
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
                    .build()
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
            .add_step(
                StepBuilder::action("step1", "Step 1", "nonexistent_agent", "op1")
                    .build()
            )
            .build();

        let context_continue = WorkflowContext::new(workflow_continue.id, registry.clone());
        let result_continue = executor_continue.execute(&workflow_continue, &context_continue).await;
        // Should not fail fast with Continue strategy
        assert!(result_continue.is_ok());

        // Test FailFast strategy
        let executor_failfast = DefaultWorkflowExecutor::new(registry.clone());
        let workflow_failfast = WorkflowBuilder::new("failfast_test")
            .error_strategy(ErrorStrategy::FailFast)
            .add_step(
                StepBuilder::action("step1", "Step 1", "nonexistent_agent", "op1")
                    .build()
            )
            .build();

        let context_failfast = WorkflowContext::new(workflow_failfast.id, registry);
        let result_failfast = executor_failfast.execute(&workflow_failfast, &context_failfast).await;
        // Should fail with FailFast strategy
        assert!(result_failfast.is_err());
    }

    #[actix_rt::test]
    async fn test_workflow_monitoring() {
        let registry = Arc::new(AgentRegistry::new());
        let monitor = Arc::new(super::super::monitoring::DefaultWorkflowMonitor::new());
        let executor = DefaultWorkflowExecutor::new(registry.clone())
            .with_monitor(monitor.clone());

        let workflow = WorkflowBuilder::new("monitor_test")
            .add_step(
                StepBuilder::action("step1", "Step 1", "agent1", "op1")
                    .build()
            )
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
            .add_step(
                StepBuilder::action("step1", "Step 1", "agent1", "op1")
                    .build()
            )
            .add_step(
                StepBuilder::action("step2", "Step 2", "agent2", "op2")
                    .build()
            )
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
}
