use super::*;
use serde_json::Value;

// Error recovery strategies
pub struct RecoveryManager;

impl RecoveryManager {
    pub async fn handle_error(
        error: &WorkflowError,
        strategy: &ErrorStrategy,
        context: &WorkflowContext,
    ) -> Result<(), WorkflowError> {
        match strategy {
            ErrorStrategy::FailFast => Err(error.clone()),
            ErrorStrategy::Continue => Ok(()),
            ErrorStrategy::Rollback => {
                Self::rollback_completed_steps(context).await
            }
            ErrorStrategy::Compensate => {
                Self::compensate_completed_steps(context).await
            }
        }
    }

    async fn rollback_completed_steps(context: &WorkflowContext) -> Result<(), WorkflowError> {
        // Get all completed steps in reverse order
        let step_results = context.step_results.read();
        let mut completed_steps: Vec<(String, Value)> = step_results
            .iter()
            .filter(|(_, result)| result.status == StepStatus::Completed)
            .filter_map(|(step_id, result)| {
                result.output.as_ref().map(|output| (step_id.clone(), output.clone()))
            })
            .collect();

        // Reverse to rollback in opposite order
        completed_steps.reverse();

        drop(step_results); // Release lock

        // Execute rollback for each step
        for (step_id, output) in completed_steps {
            if let Some(rollback_action) = Self::get_rollback_action(&step_id, &output) {
                // Log rollback action
                tracing::info!("Rolling back step: {} with action: {}", step_id, rollback_action);

                // In production, would execute actual rollback
                // For now, just record in context
                context.set_variable(
                    format!("rollback_{}", step_id),
                    serde_json::json!({ "action": rollback_action, "status": "rolled_back" })
                );
            }
        }

        Ok(())
    }

    async fn compensate_completed_steps(context: &WorkflowContext) -> Result<(), WorkflowError> {
        // Get all completed steps
        let step_results = context.step_results.read();
        let completed_steps: Vec<(String, Value)> = step_results
            .iter()
            .filter(|(_, result)| result.status == StepStatus::Completed)
            .filter_map(|(step_id, result)| {
                result.output.as_ref().map(|output| (step_id.clone(), output.clone()))
            })
            .collect();

        drop(step_results); // Release lock

        // Execute compensation for each step
        for (step_id, output) in completed_steps {
            if let Some(compensation_action) = Self::get_compensation_action(&step_id, &output) {
                // Log compensation action
                tracing::info!("Compensating step: {} with action: {}", step_id, compensation_action);

                // In production, would execute actual compensation
                // For now, just record in context
                context.set_variable(
                    format!("compensate_{}", step_id),
                    serde_json::json!({ "action": compensation_action, "status": "compensated" })
                );
            }
        }

        Ok(())
    }

    fn get_rollback_action(step_id: &str, _output: &Value) -> Option<String> {
        // In production, would look up rollback actions from step metadata
        // For now, return a placeholder action
        Some(format!("undo_{}", step_id))
    }

    fn get_compensation_action(step_id: &str, _output: &Value) -> Option<String> {
        // In production, would look up compensation actions from step metadata
        // For now, return a placeholder action
        Some(format!("compensate_{}", step_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[actix_rt::test]
    async fn test_rollback_completed_steps() {
        let agent_registry = Arc::new(crate::agents::registry::AgentRegistry::new());
        let context = WorkflowContext::new(Uuid::new_v4(), agent_registry);

        // Add completed step result
        context.set_step_result(
            "step1".to_string(),
            StepResult {
                step_id: "step1".to_string(),
                status: StepStatus::Completed,
                output: Some(serde_json::json!({"result": "success"})),
                error: None,
                started_at: Instant::now(),
                completed_at: Some(Instant::now()),
                attempts: 1,
            },
        );

        let result = RecoveryManager::rollback_completed_steps(&context).await;
        assert!(result.is_ok());

        // Check rollback was recorded
        let rollback_var = context.get_variable("rollback_step1");
        assert!(rollback_var.is_some());
    }

    #[actix_rt::test]
    async fn test_compensate_completed_steps() {
        let agent_registry = Arc::new(crate::agents::registry::AgentRegistry::new());
        let context = WorkflowContext::new(Uuid::new_v4(), agent_registry);

        // Add completed step result
        context.set_step_result(
            "step1".to_string(),
            StepResult {
                step_id: "step1".to_string(),
                status: StepStatus::Completed,
                output: Some(serde_json::json!({"result": "success"})),
                error: None,
                started_at: Instant::now(),
                completed_at: Some(Instant::now()),
                attempts: 1,
            },
        );

        let result = RecoveryManager::compensate_completed_steps(&context).await;
        assert!(result.is_ok());

        // Check compensation was recorded
        let compensate_var = context.get_variable("compensate_step1");
        assert!(compensate_var.is_some());
    }
}
