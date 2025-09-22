use super::*;
use crate::agents::registry::AgentRegistry;
use crate::modules::{ModuleRequest, ModuleResponse};
use std::sync::Arc;
use tokio::time::{timeout, sleep};
use futures::future::join_all;

// Default workflow executor implementation
pub struct DefaultWorkflowExecutor {
    agent_registry: Arc<AgentRegistry>,
    monitor: Option<Arc<dyn WorkflowMonitor>>,
}

impl DefaultWorkflowExecutor {
    pub fn new(agent_registry: Arc<AgentRegistry>) -> Self {
        Self {
            agent_registry,
            monitor: None,
        }
    }

    pub fn with_monitor(mut self, monitor: Arc<dyn WorkflowMonitor>) -> Self {
        self.monitor = Some(monitor);
        self
    }

    async fn execute_action(
        &self,
        agent: &str,
        operation: &str,
        params: &Value,
        context: &WorkflowContext,
    ) -> Result<Value, WorkflowError> {
        // Get the agent
        let agent = self.agent_registry.get_agent(agent).await
            .ok_or_else(|| WorkflowError::AgentError(format!("Agent not found: {}", agent)))?;

        // Create request based on operation
        let request = match operation {
            "analyze" => ModuleRequest::Analyze {
                code: params["code"].as_str().unwrap_or("").to_string(),
                language: params["language"].as_str().unwrap_or("unknown").to_string(),
            },
            "transform" => ModuleRequest::Transform {
                ast: params["ast"].clone(),
                operation: params["operation"].as_str().unwrap_or("optimize").to_string(),
            },
            "validate" => ModuleRequest::Validate {
                data: params.clone(),
                rules: vec![],
            },
            "orchestrate" => ModuleRequest::Orchestrate {
                workflow: params["workflow"].clone(),
                context: params["context"].clone(),
            },
            _ => return Err(WorkflowError::AgentError(format!("Unknown operation: {}", operation))),
        };

        // Execute the request
        let response = agent.process(request).await
            .map_err(|e| WorkflowError::AgentError(e.to_string()))?;

        // Convert response to Value
        match response {
            ModuleResponse::Analysis(metrics) => Ok(serde_json::to_value(metrics).unwrap()),
            ModuleResponse::Transformation(result) => Ok(serde_json::to_value(result).unwrap()),
            ModuleResponse::Validation(result) => Ok(serde_json::to_value(result).unwrap()),
            ModuleResponse::Workflow(result) => Ok(result),
        }
    }

    async fn execute_parallel(
        &self,
        steps: &[WorkflowStep],
        context: &WorkflowContext,
    ) -> Result<Value, WorkflowError> {
        let futures = steps.iter().map(|step| {
            self.execute_step(step, context)
        });

        let results = join_all(futures).await;
        
        // Collect results and errors
        let mut outputs = vec![];
        for (i, result) in results.into_iter().enumerate() {
            match result {
                Ok(output) => outputs.push(output),
                Err(e) => {
                    // Check error strategy
                    match context.get_state() {
                        WorkflowState::Running => return Err(e),
                        _ => {},
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
            let should_execute = self.evaluate_condition(&condition.expression, context).await?;
            if !should_execute {
                if condition.skip_on_false {
                    return Ok(serde_json::json!({ "skipped": true }));
                } else {
                    return Err(WorkflowError::ConditionError(
                        format!("Step condition failed: {}", condition.expression)
                    ));
                }
            }
        }

        // Execute based on step type
        let result = match &step.step_type {
            StepType::Action { agent, operation, params } => {
                self.execute_action(agent, operation, params, context).await
            },
            StepType::Parallel { steps } => {
                self.execute_parallel(steps, context).await
            },
            StepType::Sequence { steps } => {
                self.execute_sequence(steps, context).await
            },
            StepType::Conditional { condition, if_true, if_false } => {
                self.execute_conditional(condition, if_true, if_false, context).await
            },
            StepType::Loop { condition, step, max_iterations } => {
                self.execute_loop(condition, step, *max_iterations, context).await
            },
            StepType::Wait { duration } => {
                sleep(*duration).await;
                Ok(serde_json::json!({ "waited": duration.as_secs() }))
            },
            StepType::SubWorkflow { workflow_id, params } => {
                // Would recursively execute sub-workflow
                Ok(params.clone())
            },
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
            ErrorHandler::Skip => Ok(serde_json::json!({ "skipped": true, "error": error.to_string() })),
            ErrorHandler::Fail => Err(error.clone()),
            ErrorHandler::Goto { step_id } => {
                // Would jump to specified step
                Ok(serde_json::json!({ "goto": step_id }))
            },
            ErrorHandler::Execute { step } => {
                self.execute_step(step, context).await
            },
            ErrorHandler::Compensate { steps } => {
                // Would execute compensation steps
                Ok(serde_json::json!({ "compensated": steps }))
            },
        }
    }

    fn calculate_backoff(&self, strategy: &BackoffStrategy, attempt: usize) -> Duration {
        match strategy {
            BackoffStrategy::Fixed { delay } => *delay,
            BackoffStrategy::Exponential { initial, multiplier, max } => {
                let delay = initial.as_secs_f32() * multiplier.powi(attempt as i32 - 1);
                Duration::from_secs_f32(delay.min(max.as_secs_f32()))
            },
            BackoffStrategy::Linear { initial, increment } => {
                *initial + *increment * (attempt - 1) as u32
            },
        }
    }

    async fn evaluate_condition(&self, expression: &str, context: &WorkflowContext) -> Result<bool, WorkflowError> {
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

    fn resolve_variable(&self, path: &str, context: &WorkflowContext) -> Result<Value, WorkflowError> {
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
        context.get_variable(path)
            .ok_or_else(|| WorkflowError::VariableNotFound(path.to_string()))
    }
}

#[async_trait]
impl WorkflowExecutor for DefaultWorkflowExecutor {
    async fn execute(&self, workflow: &Workflow, context: &WorkflowContext) -> Result<Value, WorkflowError> {
        context.set_state(WorkflowState::Running);

        if let Some(monitor) = &self.monitor {
            monitor.on_workflow_started(workflow.id, context.execution_id).await;
        }

        let result = if let Some(timeout_duration) = workflow.timeout {
            timeout(timeout_duration, self.execute_workflow_internal(workflow, context)).await
                .map_err(|_| WorkflowError::Timeout)?
        } else {
            self.execute_workflow_internal(workflow, context).await
        };

        match &result {
            Ok(output) => {
                context.set_state(WorkflowState::Completed);
                if let Some(monitor) = &self.monitor {
                    monitor.on_workflow_completed(workflow.id, context.execution_id, output).await;
                }
            },
            Err(e) => {
                context.set_state(WorkflowState::Failed);
                if let Some(monitor) = &self.monitor {
                    monitor.on_workflow_failed(workflow.id, context.execution_id, e).await;
                }
            },
        }

        result
    }

    async fn execute_step(&self, step: &WorkflowStep, context: &WorkflowContext) -> Result<Value, WorkflowError> {
        if let Some(monitor) = &self.monitor {
            monitor.on_step_started(context.execution_id, &step.id).await;
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
                    monitor.on_step_completed(context.execution_id, &step.id, &value).await;
                }
                
                context.set_step_result(step.id.clone(), result);
                Ok(value)
            },
            Err(e) => {
                result.status = StepStatus::Failed;
                result.error = Some(e.to_string());
                result.completed_at = Some(Instant::now());
                
                if let Some(monitor) = &self.monitor {
                    monitor.on_step_failed(context.execution_id, &step.id, &e.to_string()).await;
                }
                
                context.set_step_result(step.id.clone(), result);
                Err(e)
            },
        }
    }

    async fn pause(&self, _execution_id: Uuid) -> Result<(), WorkflowError> {
        // Implementation would pause execution
        Ok(())
    }

    async fn resume(&self, _execution_id: Uuid) -> Result<(), WorkflowError> {
        // Implementation would resume execution
        Ok(())
    }

    async fn cancel(&self, _execution_id: Uuid) -> Result<(), WorkflowError> {
        // Implementation would cancel execution
        Ok(())
    }
}

impl DefaultWorkflowExecutor {
    async fn execute_workflow_internal(
        &self,
        workflow: &Workflow,
        context: &WorkflowContext,
    ) -> Result<Value, WorkflowError> {
        let mut last_output = serde_json::json!({});
        
        for step in &workflow.steps {
            match self.execute_step(step, context).await {
                Ok(output) => last_output = output,
                Err(e) => {
                    match workflow.error_strategy {
                        ErrorStrategy::FailFast => return Err(e),
                        ErrorStrategy::Continue => continue,
                        ErrorStrategy::Rollback => {
                            // Would implement rollback logic
                            return Err(e);
                        },
                        ErrorStrategy::Compensate => {
                            // Would implement compensation logic
                            return Err(e);
                        },
                    }
                }
            }
        }

        Ok(last_output)
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
        let fixed = BackoffStrategy::Fixed { delay: Duration::from_secs(5) };
        assert_eq!(executor.calculate_backoff(&fixed, 1), Duration::from_secs(5));
        assert_eq!(executor.calculate_backoff(&fixed, 3), Duration::from_secs(5));
        
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
        assert_eq!(executor.calculate_backoff(&linear, 1), Duration::from_secs(1));
        assert_eq!(executor.calculate_backoff(&linear, 2), Duration::from_secs(3));
        assert_eq!(executor.calculate_backoff(&linear, 3), Duration::from_secs(5));
    }
}