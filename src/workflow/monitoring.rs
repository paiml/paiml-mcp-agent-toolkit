#![cfg_attr(coverage_nightly, coverage(off))]
use super::*;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

// Default workflow monitor implementation
pub struct DefaultWorkflowMonitor {
    metrics: Arc<RwLock<HashMap<Uuid, WorkflowMetrics>>>,
}

impl Default for DefaultWorkflowMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultWorkflowMonitor {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl WorkflowMonitor for DefaultWorkflowMonitor {
    async fn on_workflow_started(&self, workflow_id: Uuid, execution_id: Uuid) {
        let mut metrics = self.metrics.write();
        metrics.insert(
            execution_id,
            WorkflowMetrics {
                execution_id,
                workflow_id,
                state: WorkflowState::Running,
                total_steps: 0,
                completed_steps: 0,
                failed_steps: 0,
                skipped_steps: 0,
                elapsed_time: Duration::from_secs(0),
                average_step_time: None,
                retry_count: 0,
            },
        );
    }

    async fn on_workflow_completed(&self, _workflow_id: Uuid, execution_id: Uuid, _result: &Value) {
        if let Some(metric) = self.metrics.write().get_mut(&execution_id) {
            metric.state = WorkflowState::Completed;
        }
    }

    async fn on_workflow_failed(
        &self,
        _workflow_id: Uuid,
        execution_id: Uuid,
        _error: &WorkflowError,
    ) {
        if let Some(metric) = self.metrics.write().get_mut(&execution_id) {
            metric.state = WorkflowState::Failed;
        }
    }

    async fn on_step_started(&self, execution_id: Uuid, _step_id: &str) {
        if let Some(metric) = self.metrics.write().get_mut(&execution_id) {
            metric.total_steps += 1;
        }
    }

    async fn on_step_completed(&self, execution_id: Uuid, _step_id: &str, _result: &Value) {
        if let Some(metric) = self.metrics.write().get_mut(&execution_id) {
            metric.completed_steps += 1;
        }
    }

    async fn on_step_failed(&self, execution_id: Uuid, _step_id: &str, _error: &str) {
        if let Some(metric) = self.metrics.write().get_mut(&execution_id) {
            metric.failed_steps += 1;
        }
    }

    async fn get_metrics(&self, execution_id: Uuid) -> WorkflowMetrics {
        self.metrics
            .read()
            .get(&execution_id)
            .cloned()
            .unwrap_or(WorkflowMetrics {
                execution_id,
                workflow_id: Uuid::new_v4(),
                state: WorkflowState::Created,
                total_steps: 0,
                completed_steps: 0,
                failed_steps: 0,
                skipped_steps: 0,
                elapsed_time: Duration::from_secs(0),
                average_step_time: None,
                retry_count: 0,
            })
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_default_workflow_monitor_new() {
        let monitor = DefaultWorkflowMonitor::new();
        let execution_id = Uuid::new_v4();
        let metrics = monitor.get_metrics(execution_id).await;
        assert_eq!(metrics.state, WorkflowState::Created);
        assert_eq!(metrics.total_steps, 0);
    }

    #[tokio::test]
    async fn test_workflow_lifecycle() {
        let monitor = DefaultWorkflowMonitor::new();
        let workflow_id = Uuid::new_v4();
        let execution_id = Uuid::new_v4();

        // Start workflow
        monitor.on_workflow_started(workflow_id, execution_id).await;
        let metrics = monitor.get_metrics(execution_id).await;
        assert_eq!(metrics.state, WorkflowState::Running);
        assert_eq!(metrics.workflow_id, workflow_id);

        // Start and complete a step
        monitor.on_step_started(execution_id, "step1").await;
        let metrics = monitor.get_metrics(execution_id).await;
        assert_eq!(metrics.total_steps, 1);

        monitor
            .on_step_completed(execution_id, "step1", &serde_json::json!({}))
            .await;
        let metrics = monitor.get_metrics(execution_id).await;
        assert_eq!(metrics.completed_steps, 1);

        // Complete workflow
        monitor
            .on_workflow_completed(workflow_id, execution_id, &serde_json::json!({}))
            .await;
        let metrics = monitor.get_metrics(execution_id).await;
        assert_eq!(metrics.state, WorkflowState::Completed);
    }

    #[tokio::test]
    async fn test_workflow_failure() {
        let monitor = DefaultWorkflowMonitor::new();
        let workflow_id = Uuid::new_v4();
        let execution_id = Uuid::new_v4();

        monitor.on_workflow_started(workflow_id, execution_id).await;

        // Fail a step
        monitor.on_step_started(execution_id, "step1").await;
        monitor
            .on_step_failed(execution_id, "step1", "Test error")
            .await;
        let metrics = monitor.get_metrics(execution_id).await;
        assert_eq!(metrics.failed_steps, 1);

        // Fail workflow
        let error = WorkflowError::StepFailed("Test failure".to_string());
        monitor
            .on_workflow_failed(workflow_id, execution_id, &error)
            .await;
        let metrics = monitor.get_metrics(execution_id).await;
        assert_eq!(metrics.state, WorkflowState::Failed);
    }

    #[test]
    fn test_default_trait() {
        let monitor = DefaultWorkflowMonitor::default();
        assert!(monitor.metrics.read().is_empty());
    }

    #[tokio::test]
    async fn test_get_metrics_unknown_execution() {
        let monitor = DefaultWorkflowMonitor::new();
        let unknown_id = Uuid::new_v4();
        let metrics = monitor.get_metrics(unknown_id).await;
        // Should return default metrics
        assert_eq!(metrics.execution_id, unknown_id);
        assert_eq!(metrics.state, WorkflowState::Created);
    }
}
