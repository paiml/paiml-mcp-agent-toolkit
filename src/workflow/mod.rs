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
    pub retry_on: Vec<String>,
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

include!("workflow_context_impl.rs");
include!("workflow_builders.rs");
include!("workflow_tests.rs");
