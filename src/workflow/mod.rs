#![cfg_attr(coverage_nightly, coverage(off))]
// Workflow orchestration engine
pub mod conditions;
pub mod dag;
pub mod dsl;
pub mod executor;
pub mod monitoring;
pub mod recovery;
pub mod repository;
pub mod steps;

// Re-export main types
pub use dag::{DagAnalysis, DagEngine, DagNode};
pub use executor::DefaultWorkflowExecutor;
pub use monitoring::DefaultWorkflowMonitor;
pub use repository::InMemoryWorkflowRepository;

use async_trait::async_trait;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use uuid::Uuid;

// Workflow definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub version: String,
    pub steps: Vec<WorkflowStep>,
    pub error_strategy: ErrorStrategy,
    pub timeout: Option<Duration>,
    pub metadata: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub id: String,
    pub name: String,
    pub step_type: StepType,
    pub condition: Option<StepCondition>,
    pub retry: Option<RetryPolicy>,
    pub timeout: Option<Duration>,
    pub on_error: Option<ErrorHandler>,
    pub metadata: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StepType {
    #[serde(rename = "action")]
    Action {
        agent: String,
        operation: String,
        params: Value,
    },
    #[serde(rename = "parallel")]
    Parallel { steps: Vec<WorkflowStep> },
    #[serde(rename = "sequence")]
    Sequence { steps: Vec<WorkflowStep> },
    #[serde(rename = "conditional")]
    Conditional {
        condition: String,
        if_true: Box<WorkflowStep>,
        if_false: Option<Box<WorkflowStep>>,
    },
    #[serde(rename = "loop")]
    Loop {
        condition: String,
        step: Box<WorkflowStep>,
        max_iterations: Option<usize>,
    },
    #[serde(rename = "wait")]
    Wait { duration: Duration },
    #[serde(rename = "subworkflow")]
    SubWorkflow { workflow_id: Uuid, params: Value },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepCondition {
    pub expression: String,
    pub skip_on_false: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_attempts: usize,
    pub backoff: BackoffStrategy,
    pub retry_on: Vec<String>, // Error types to retry
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackoffStrategy {
    Fixed {
        delay: Duration,
    },
    Exponential {
        initial: Duration,
        multiplier: f32,
        max: Duration,
    },
    Linear {
        initial: Duration,
        increment: Duration,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorHandler {
    Skip,
    Fail,
    Goto { step_id: String },
    Execute { step: Box<WorkflowStep> },
    Compensate { steps: Vec<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorStrategy {
    FailFast,
    Continue,
    Rollback,
    Compensate,
}

// Workflow execution context
pub struct WorkflowContext {
    pub workflow_id: Uuid,
    pub execution_id: Uuid,
    pub variables: Arc<RwLock<HashMap<String, Value>>>,
    pub step_results: Arc<RwLock<HashMap<String, StepResult>>>,
    pub state: Arc<RwLock<WorkflowState>>,
    pub started_at: Instant,
    pub agent_registry: Arc<crate::agents::registry::AgentRegistry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub step_id: String,
    pub status: StepStatus,
    pub output: Option<Value>,
    pub error: Option<String>,
    #[serde(skip, default = "Instant::now")]
    pub started_at: Instant,
    #[serde(skip)]
    pub completed_at: Option<Instant>,
    pub attempts: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum StepStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum WorkflowState {
    Created,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

impl WorkflowContext {
    pub fn new(
        workflow_id: Uuid,
        agent_registry: Arc<crate::agents::registry::AgentRegistry>,
    ) -> Self {
        Self {
            workflow_id,
            execution_id: Uuid::new_v4(),
            variables: Arc::new(RwLock::new(HashMap::new())),
            step_results: Arc::new(RwLock::new(HashMap::new())),
            state: Arc::new(RwLock::new(WorkflowState::Created)),
            started_at: Instant::now(),
            agent_registry,
        }
    }

    pub fn set_variable(&self, name: String, value: Value) {
        self.variables.write().insert(name, value);
    }

    pub fn get_variable(&self, name: &str) -> Option<Value> {
        self.variables.read().get(name).cloned()
    }

    pub fn set_step_result(&self, step_id: String, result: StepResult) {
        self.step_results.write().insert(step_id, result);
    }

    pub fn get_step_result(&self, step_id: &str) -> Option<StepResult> {
        self.step_results.read().get(step_id).cloned()
    }

    pub fn set_state(&self, state: WorkflowState) {
        *self.state.write() = state;
    }

    pub fn get_state(&self) -> WorkflowState {
        *self.state.read()
    }

    pub fn get_elapsed(&self) -> Duration {
        self.started_at.elapsed()
    }
}

// Workflow executor trait
#[async_trait]
pub trait WorkflowExecutor: Send + Sync {
    async fn execute(
        &self,
        workflow: &Workflow,
        context: &WorkflowContext,
    ) -> Result<Value, WorkflowError>;
    async fn execute_step(
        &self,
        step: &WorkflowStep,
        context: &WorkflowContext,
    ) -> Result<Value, WorkflowError>;
    async fn pause(&self, execution_id: Uuid) -> Result<(), WorkflowError>;
    async fn resume(&self, execution_id: Uuid) -> Result<(), WorkflowError>;
    async fn cancel(&self, execution_id: Uuid) -> Result<(), WorkflowError>;
}

// Workflow repository
#[async_trait]
pub trait WorkflowRepository: Send + Sync {
    async fn save(&self, workflow: &Workflow) -> Result<(), WorkflowError>;
    async fn get(&self, id: Uuid) -> Result<Option<Workflow>, WorkflowError>;
    async fn list(&self) -> Result<Vec<Workflow>, WorkflowError>;
    async fn delete(&self, id: Uuid) -> Result<(), WorkflowError>;
    async fn get_by_name(&self, name: &str) -> Result<Option<Workflow>, WorkflowError>;
}

// Workflow monitor
#[async_trait]
pub trait WorkflowMonitor: Send + Sync {
    async fn on_workflow_started(&self, workflow_id: Uuid, execution_id: Uuid);
    async fn on_workflow_completed(&self, workflow_id: Uuid, execution_id: Uuid, result: &Value);
    async fn on_workflow_failed(
        &self,
        workflow_id: Uuid,
        execution_id: Uuid,
        error: &WorkflowError,
    );
    async fn on_step_started(&self, execution_id: Uuid, step_id: &str);
    async fn on_step_completed(&self, execution_id: Uuid, step_id: &str, result: &Value);
    async fn on_step_failed(&self, execution_id: Uuid, step_id: &str, error: &str);
    async fn get_metrics(&self, execution_id: Uuid) -> WorkflowMetrics;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowMetrics {
    pub execution_id: Uuid,
    pub workflow_id: Uuid,
    pub state: WorkflowState,
    pub total_steps: usize,
    pub completed_steps: usize,
    pub failed_steps: usize,
    pub skipped_steps: usize,
    pub elapsed_time: Duration,
    pub average_step_time: Option<Duration>,
    pub retry_count: usize,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum WorkflowError {
    #[error("Workflow not found: {0}")]
    NotFound(Uuid),
    #[error("Step failed: {0}")]
    StepFailed(String),
    #[error("Condition evaluation failed: {0}")]
    ConditionError(String),
    #[error("Timeout exceeded")]
    Timeout,
    #[error("Workflow cancelled")]
    Cancelled,
    #[error("Invalid workflow definition: {0}")]
    InvalidDefinition(String),
    #[error("Agent error: {0}")]
    AgentError(String),
    #[error("Variable not found: {0}")]
    VariableNotFound(String),
    #[error("Maximum retries exceeded")]
    MaxRetriesExceeded,
    #[error("Execution error: {0}")]
    ExecutionError(String),
}

// Workflow builder for programmatic workflow creation
pub struct WorkflowBuilder {
    workflow: Workflow,
}

impl WorkflowBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            workflow: Workflow {
                id: Uuid::new_v4(),
                name: name.into(),
                description: None,
                version: "1.0.0".to_string(),
                steps: Vec::new(),
                error_strategy: ErrorStrategy::FailFast,
                timeout: None,
                metadata: HashMap::new(),
            },
        }
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.workflow.description = Some(desc.into());
        self
    }

    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.workflow.version = version.into();
        self
    }

    pub fn add_step(mut self, step: WorkflowStep) -> Self {
        self.workflow.steps.push(step);
        self
    }

    pub fn error_strategy(mut self, strategy: ErrorStrategy) -> Self {
        self.workflow.error_strategy = strategy;
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.workflow.timeout = Some(timeout);
        self
    }

    pub fn metadata(mut self, key: impl Into<String>, value: Value) -> Self {
        self.workflow.metadata.insert(key.into(), value);
        self
    }

    pub fn build(self) -> Workflow {
        self.workflow
    }
}

// Step builder
pub struct StepBuilder {
    step: WorkflowStep,
}

impl StepBuilder {
    pub fn action(
        id: impl Into<String>,
        name: impl Into<String>,
        agent: impl Into<String>,
        operation: impl Into<String>,
    ) -> Self {
        Self {
            step: WorkflowStep {
                id: id.into(),
                name: name.into(),
                step_type: StepType::Action {
                    agent: agent.into(),
                    operation: operation.into(),
                    params: Value::Object(serde_json::Map::new()),
                },
                condition: None,
                retry: None,
                timeout: None,
                on_error: None,
                metadata: HashMap::new(),
            },
        }
    }

    pub fn params(mut self, new_params: Value) -> Self {
        if let StepType::Action { params, .. } = &mut self.step.step_type {
            *params = new_params;
        }
        self
    }

    pub fn condition(mut self, expression: impl Into<String>, skip_on_false: bool) -> Self {
        self.step.condition = Some(StepCondition {
            expression: expression.into(),
            skip_on_false,
        });
        self
    }

    pub fn retry(mut self, max_attempts: usize, backoff: BackoffStrategy) -> Self {
        self.step.retry = Some(RetryPolicy {
            max_attempts,
            backoff,
            retry_on: vec![],
        });
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.step.timeout = Some(timeout);
        self
    }

    pub fn on_error(mut self, handler: ErrorHandler) -> Self {
        self.step.on_error = Some(handler);
        self
    }

    pub fn build(self) -> WorkflowStep {
        self.step
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_builder() {
        let workflow = WorkflowBuilder::new("test_workflow")
            .description("Test workflow")
            .version("2.0.0")
            .error_strategy(ErrorStrategy::Continue)
            .timeout(Duration::from_secs(300))
            .build();

        assert_eq!(workflow.name, "test_workflow");
        assert_eq!(workflow.version, "2.0.0");
        assert!(matches!(workflow.error_strategy, ErrorStrategy::Continue));
        assert_eq!(workflow.timeout, Some(Duration::from_secs(300)));
    }

    #[test]
    fn test_step_builder() {
        let step = StepBuilder::action("step1", "Analyze", "analyzer", "analyze")
            .params(serde_json::json!({"language": "rust"}))
            .condition("result.score > 0.8", true)
            .retry(
                3,
                BackoffStrategy::Exponential {
                    initial: Duration::from_secs(1),
                    multiplier: 2.0,
                    max: Duration::from_secs(10),
                },
            )
            .timeout(Duration::from_secs(30))
            .build();

        assert_eq!(step.id, "step1");
        assert_eq!(step.name, "Analyze");
        assert!(step.condition.is_some());
        assert!(step.retry.is_some());
        assert_eq!(step.timeout, Some(Duration::from_secs(30)));
    }

    #[test]
    fn test_step_status_variants() {
        assert_ne!(StepStatus::Pending, StepStatus::Running);
        assert_ne!(StepStatus::Running, StepStatus::Completed);
        assert_ne!(StepStatus::Completed, StepStatus::Failed);
        assert_ne!(StepStatus::Failed, StepStatus::Skipped);
        assert_ne!(StepStatus::Skipped, StepStatus::Cancelled);
    }

    #[test]
    fn test_workflow_state_variants() {
        assert_ne!(WorkflowState::Created, WorkflowState::Running);
        assert_ne!(WorkflowState::Running, WorkflowState::Paused);
        assert_ne!(WorkflowState::Paused, WorkflowState::Completed);
        assert_ne!(WorkflowState::Completed, WorkflowState::Failed);
        assert_ne!(WorkflowState::Failed, WorkflowState::Cancelled);
    }

    #[test]
    fn test_backoff_strategy_fixed() {
        let strategy = BackoffStrategy::Fixed {
            delay: Duration::from_secs(5),
        };
        if let BackoffStrategy::Fixed { delay } = strategy {
            assert_eq!(delay, Duration::from_secs(5));
        } else {
            panic!("Expected Fixed strategy");
        }
    }

    #[test]
    fn test_backoff_strategy_exponential() {
        let strategy = BackoffStrategy::Exponential {
            initial: Duration::from_secs(1),
            multiplier: 2.0,
            max: Duration::from_secs(60),
        };
        if let BackoffStrategy::Exponential {
            initial,
            multiplier,
            max,
        } = strategy
        {
            assert_eq!(initial, Duration::from_secs(1));
            assert_eq!(multiplier, 2.0);
            assert_eq!(max, Duration::from_secs(60));
        } else {
            panic!("Expected Exponential strategy");
        }
    }

    #[test]
    fn test_backoff_strategy_linear() {
        let strategy = BackoffStrategy::Linear {
            initial: Duration::from_secs(1),
            increment: Duration::from_secs(1),
        };
        if let BackoffStrategy::Linear { initial, increment } = strategy {
            assert_eq!(initial, Duration::from_secs(1));
            assert_eq!(increment, Duration::from_secs(1));
        } else {
            panic!("Expected Linear strategy");
        }
    }

    #[test]
    fn test_error_handler_variants() {
        let skip = ErrorHandler::Skip;
        let fail = ErrorHandler::Fail;
        let goto = ErrorHandler::Goto {
            step_id: "step2".to_string(),
        };
        let compensate = ErrorHandler::Compensate {
            steps: vec!["cleanup".to_string()],
        };

        assert!(matches!(skip, ErrorHandler::Skip));
        assert!(matches!(fail, ErrorHandler::Fail));
        assert!(matches!(goto, ErrorHandler::Goto { .. }));
        assert!(matches!(compensate, ErrorHandler::Compensate { .. }));
    }

    #[test]
    fn test_error_strategy_variants() {
        assert!(matches!(ErrorStrategy::FailFast, ErrorStrategy::FailFast));
        assert!(matches!(ErrorStrategy::Continue, ErrorStrategy::Continue));
        assert!(matches!(ErrorStrategy::Rollback, ErrorStrategy::Rollback));
        assert!(matches!(
            ErrorStrategy::Compensate,
            ErrorStrategy::Compensate
        ));
    }

    #[test]
    fn test_step_condition() {
        let condition = StepCondition {
            expression: "result.valid == true".to_string(),
            skip_on_false: true,
        };
        assert_eq!(condition.expression, "result.valid == true");
        assert!(condition.skip_on_false);
    }

    #[test]
    fn test_retry_policy() {
        let policy = RetryPolicy {
            max_attempts: 3,
            backoff: BackoffStrategy::Fixed {
                delay: Duration::from_secs(1),
            },
            retry_on: vec!["NetworkError".to_string()],
        };
        assert_eq!(policy.max_attempts, 3);
        assert_eq!(policy.retry_on.len(), 1);
    }

    #[test]
    fn test_step_result_creation() {
        let result = StepResult {
            step_id: "test_step".to_string(),
            status: StepStatus::Completed,
            output: Some(serde_json::json!({"value": 42})),
            error: None,
            started_at: Instant::now(),
            completed_at: Some(Instant::now()),
            attempts: 1,
        };
        assert_eq!(result.step_id, "test_step");
        assert_eq!(result.status, StepStatus::Completed);
        assert!(result.output.is_some());
        assert!(result.error.is_none());
        assert_eq!(result.attempts, 1);
    }

    #[test]
    fn test_workflow_builder_defaults() {
        let workflow = WorkflowBuilder::new("default_test").build();
        assert_eq!(workflow.name, "default_test");
        assert_eq!(workflow.version, "1.0.0");
        assert!(matches!(workflow.error_strategy, ErrorStrategy::FailFast));
        assert!(workflow.timeout.is_none());
        assert!(workflow.description.is_none());
    }

    #[test]
    fn test_step_type_action() {
        let step_type = StepType::Action {
            agent: "analyzer".to_string(),
            operation: "analyze".to_string(),
            params: serde_json::json!({"path": "."}),
        };
        if let StepType::Action {
            agent,
            operation,
            params,
        } = step_type
        {
            assert_eq!(agent, "analyzer");
            assert_eq!(operation, "analyze");
            assert!(params.is_object());
        } else {
            panic!("Expected Action step type");
        }
    }

    #[test]
    fn test_step_type_wait() {
        let step_type = StepType::Wait {
            duration: Duration::from_secs(10),
        };
        if let StepType::Wait { duration } = step_type {
            assert_eq!(duration, Duration::from_secs(10));
        } else {
            panic!("Expected Wait step type");
        }
    }

    #[test]
    fn test_workflow_serialization() {
        let workflow = WorkflowBuilder::new("serialization_test")
            .description("Test serialization")
            .version("1.0.0")
            .build();
        let json = serde_json::to_string(&workflow).unwrap();
        assert!(json.contains("serialization_test"));
        assert!(json.contains("Test serialization"));
    }

    #[test]
    fn test_step_result_with_error() {
        let result = StepResult {
            step_id: "failing_step".to_string(),
            status: StepStatus::Failed,
            output: None,
            error: Some("Connection timeout".to_string()),
            started_at: Instant::now(),
            completed_at: Some(Instant::now()),
            attempts: 3,
        };
        assert_eq!(result.status, StepStatus::Failed);
        assert!(result.error.is_some());
        assert_eq!(result.error.unwrap(), "Connection timeout");
    }

    #[test]
    fn test_workflow_error_display() {
        let err = WorkflowError::StepFailed("Test step failed".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Step failed") || msg.contains("Test step"));

        let invalid_err = WorkflowError::InvalidDefinition("Invalid config".to_string());
        assert!(invalid_err.to_string().contains("Invalid"));

        let timeout_err = WorkflowError::Timeout;
        assert!(timeout_err.to_string().contains("Timeout"));

        let not_found_err = WorkflowError::NotFound(Uuid::new_v4());
        assert!(not_found_err.to_string().contains("not found"));

        let cancelled_err = WorkflowError::Cancelled;
        assert!(cancelled_err.to_string().contains("cancelled"));
    }
}
