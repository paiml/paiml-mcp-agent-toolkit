// Helper methods for DefaultWorkflowExecutor
// Extracted from executor.rs for file health compliance

impl DefaultWorkflowExecutor {
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
        debug_assert!(!expression.is_empty(), "expression must not be empty");
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
        debug_assert!(!path.is_empty(), "path must not be empty");
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
                            let output_path =
                                field.strip_prefix("output.").expect("internal error");
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
