#![cfg_attr(coverage_nightly, coverage(off))]
use super::*;
use serde_json::Value;

// Error recovery strategies
/// Recovery manager.
pub struct RecoveryManager;

impl RecoveryManager {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn handle_error(
        error: &WorkflowError,
        strategy: &ErrorStrategy,
        context: &WorkflowContext,
    ) -> Result<(), WorkflowError> {
        match strategy {
            ErrorStrategy::FailFast => Err(error.clone()),
            ErrorStrategy::Continue => Ok(()),
            ErrorStrategy::Rollback => Self::rollback_completed_steps(context).await,
            ErrorStrategy::Compensate => Self::compensate_completed_steps(context).await,
        }
    }

    async fn rollback_completed_steps(context: &WorkflowContext) -> Result<(), WorkflowError> {
        // Get all completed steps in reverse order
        let step_results = context.step_results.read();
        let mut completed_steps: Vec<(String, Value)> = step_results
            .iter()
            .filter(|(_, result)| result.status == StepStatus::Completed)
            .filter_map(|(step_id, result)| {
                result
                    .output
                    .as_ref()
                    .map(|output| (step_id.clone(), output.clone()))
            })
            .collect();

        // Reverse to rollback in opposite order
        completed_steps.reverse();

        drop(step_results); // Release lock

        // Execute rollback for each step
        for (step_id, output) in completed_steps {
            if let Some(rollback_action) = Self::get_rollback_action(&step_id, &output) {
                // Log rollback action
                tracing::info!(
                    "Rolling back step: {} with action: {}",
                    step_id,
                    rollback_action
                );

                // In production, would execute actual rollback
                // For now, just record in context
                context.set_variable(
                    format!("rollback_{}", step_id),
                    serde_json::json!({ "action": rollback_action, "status": "rolled_back" }),
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
                result
                    .output
                    .as_ref()
                    .map(|output| (step_id.clone(), output.clone()))
            })
            .collect();

        drop(step_results); // Release lock

        // Execute compensation for each step
        for (step_id, output) in completed_steps {
            if let Some(compensation_action) = Self::get_compensation_action(&step_id, &output) {
                // Log compensation action
                tracing::info!(
                    "Compensating step: {} with action: {}",
                    step_id,
                    compensation_action
                );

                // In production, would execute actual compensation
                // For now, just record in context
                context.set_variable(
                    format!("compensate_{}", step_id),
                    serde_json::json!({ "action": compensation_action, "status": "compensated" }),
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

#[cfg_attr(coverage_nightly, coverage(off))]
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

    #[actix_rt::test]
    async fn test_handle_error_fail_fast() {
        let agent_registry = Arc::new(crate::agents::registry::AgentRegistry::new());
        let context = WorkflowContext::new(Uuid::new_v4(), agent_registry);

        let error = WorkflowError::ExecutionError("Test error".to_string());
        let strategy = ErrorStrategy::FailFast;

        let result = RecoveryManager::handle_error(&error, &strategy, &context).await;
        assert!(result.is_err());
    }

    #[actix_rt::test]
    async fn test_handle_error_continue() {
        let agent_registry = Arc::new(crate::agents::registry::AgentRegistry::new());
        let context = WorkflowContext::new(Uuid::new_v4(), agent_registry);

        let error = WorkflowError::ExecutionError("Test error".to_string());
        let strategy = ErrorStrategy::Continue;

        let result = RecoveryManager::handle_error(&error, &strategy, &context).await;
        assert!(result.is_ok());
    }

    #[actix_rt::test]
    async fn test_handle_error_rollback() {
        let agent_registry = Arc::new(crate::agents::registry::AgentRegistry::new());
        let context = WorkflowContext::new(Uuid::new_v4(), agent_registry);

        // Add a completed step first
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

        let error = WorkflowError::ExecutionError("Test error".to_string());
        let strategy = ErrorStrategy::Rollback;

        let result = RecoveryManager::handle_error(&error, &strategy, &context).await;
        assert!(result.is_ok());
    }

    #[actix_rt::test]
    async fn test_handle_error_compensate() {
        let agent_registry = Arc::new(crate::agents::registry::AgentRegistry::new());
        let context = WorkflowContext::new(Uuid::new_v4(), agent_registry);

        // Add a completed step first
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

        let error = WorkflowError::ExecutionError("Test error".to_string());
        let strategy = ErrorStrategy::Compensate;

        let result = RecoveryManager::handle_error(&error, &strategy, &context).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_rollback_action() {
        let output = serde_json::json!({"key": "value"});
        let action = RecoveryManager::get_rollback_action("my_step", &output);
        assert!(action.is_some());
        assert_eq!(action.unwrap(), "undo_my_step");
    }

    #[test]
    fn test_get_compensation_action() {
        let output = serde_json::json!({"key": "value"});
        let action = RecoveryManager::get_compensation_action("my_step", &output);
        assert!(action.is_some());
        assert_eq!(action.unwrap(), "compensate_my_step");
    }

    #[actix_rt::test]
    async fn test_rollback_multiple_steps() {
        let agent_registry = Arc::new(crate::agents::registry::AgentRegistry::new());
        let context = WorkflowContext::new(Uuid::new_v4(), agent_registry);

        // Add multiple completed steps
        for i in 1..=3 {
            context.set_step_result(
                format!("step{}", i),
                StepResult {
                    step_id: format!("step{}", i),
                    status: StepStatus::Completed,
                    output: Some(serde_json::json!({"result": format!("success_{}", i)})),
                    error: None,
                    started_at: Instant::now(),
                    completed_at: Some(Instant::now()),
                    attempts: 1,
                },
            );
        }

        let result = RecoveryManager::rollback_completed_steps(&context).await;
        assert!(result.is_ok());

        // Check all rollbacks were recorded
        for i in 1..=3 {
            let rollback_var = context.get_variable(&format!("rollback_step{}", i));
            assert!(rollback_var.is_some());
        }
    }

    #[actix_rt::test]
    async fn test_rollback_skips_non_completed_steps() {
        let agent_registry = Arc::new(crate::agents::registry::AgentRegistry::new());
        let context = WorkflowContext::new(Uuid::new_v4(), agent_registry);

        // Add a pending step (should not be rolled back)
        context.set_step_result(
            "pending_step".to_string(),
            StepResult {
                step_id: "pending_step".to_string(),
                status: StepStatus::Pending,
                output: None,
                error: None,
                started_at: Instant::now(),
                completed_at: None,
                attempts: 0,
            },
        );

        // Add a completed step (should be rolled back)
        context.set_step_result(
            "completed_step".to_string(),
            StepResult {
                step_id: "completed_step".to_string(),
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

        // Only completed step should have rollback variable
        assert!(context.get_variable("rollback_completed_step").is_some());
        assert!(context.get_variable("rollback_pending_step").is_none());
    }
}
