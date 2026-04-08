#![cfg_attr(coverage_nightly, coverage(off))]
use super::*;
use crate::agents::registry::AgentRegistry;
use futures::future::join_all;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::{sleep, timeout};

// Default workflow executor implementation
/// Default workflow executor.
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
    _completed_steps: Vec<String>,
    _current_level: usize,
}

impl Default for ExecutionState {
    fn default() -> Self {
        Self {
            control: ExecutionControl::Running,
            checkpoint: None,
        }
    }
}

// Constructor and step execution methods (execute_action, execute_parallel,
// execute_sequence, execute_conditional, execute_loop, execute_with_retry,
// execute_step_internal)
include!("executor_core.rs");

// Helper methods (handle_error, calculate_backoff, evaluate_condition,
// resolve_variable)
include!("executor_helpers.rs");

// WorkflowExecutor trait impl and workflow-level execution logic
// (execute_workflow_internal, check_execution_control, save_checkpoint,
// handle_workflow_error)
include!("executor_trait_impl.rs");

// Tests extracted to executor_tests.rs for file health compliance (CB-040)
// Fixed duplicate imports in Sprint 45
#[cfg(test)]
#[path = "executor_tests.rs"]
mod tests;
