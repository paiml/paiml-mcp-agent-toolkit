use super::*;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

// Default workflow monitor implementation
pub struct DefaultWorkflowMonitor {
    metrics: Arc<RwLock<HashMap<Uuid, WorkflowMetrics>>>,
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
