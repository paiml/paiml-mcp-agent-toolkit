use crate::mcp_server::state_manager::StateManager;
use crate::models::refactor::RefactorConfig;
use async_trait::async_trait;
use pmcp::{Error as PmcpError, Result as PmcpResult, ToolHandler};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;

#[derive(Debug, Deserialize)]
struct RefactorStartArgs {
    targets: Vec<String>,
    config: Option<RefactorConfig>,
}

#[derive(Debug, Serialize)]
struct RefactorStartResult {
    session_id: String,
    state: Value,
}

pub struct RefactorStartTool {
    state_manager: Arc<Mutex<StateManager>>,
}

impl RefactorStartTool {
    pub fn new(state_manager: Arc<Mutex<StateManager>>) -> Self {
        Self { state_manager }
    }
}

#[async_trait]
impl ToolHandler for RefactorStartTool {
    async fn handle(&self, args: Value) -> PmcpResult<Value> {
        debug!("Handling refactor.start with args: {}", args);

        let params: RefactorStartArgs = serde_json::from_value(args)
            .map_err(|e| PmcpError::validation(format!("Invalid arguments: {}", e)))?;

        let targets: Vec<PathBuf> = params
            .targets
            .into_iter()
            .map(PathBuf::from)
            .collect();

        let config = params.config.unwrap_or_default();

        let mut manager = self.state_manager.lock().await;
        manager
            .start_session(targets, config)
            .map_err(|e| PmcpError::internal(format!("Failed to start session: {}", e)))?;

        let state = manager
            .get_state()
            .map_err(|e| PmcpError::internal(format!("Failed to get state: {}", e)))?;
        let session_id = manager.get_session_id().to_string();

        let state_value = serialize_state(state)
            .map_err(|e| PmcpError::internal(format!("Failed to serialize state: {}", e)))?;

        Ok(serde_json::to_value(RefactorStartResult {
            session_id,
            state: state_value,
        })?)
    }
}

pub struct RefactorNextIterationTool {
    state_manager: Arc<Mutex<StateManager>>,
}

impl RefactorNextIterationTool {
    pub fn new(state_manager: Arc<Mutex<StateManager>>) -> Self {
        Self { state_manager }
    }
}

#[async_trait]
impl ToolHandler for RefactorNextIterationTool {
    async fn handle(&self, _args: Value) -> PmcpResult<Value> {
        debug!("Handling refactor.nextIteration");

        let mut manager = self.state_manager.lock().await;
        manager
            .advance()
            .map_err(|e| PmcpError::internal(format!("Failed to advance: {}", e)))?;

        let state = manager
            .get_state()
            .map_err(|e| PmcpError::internal(format!("Failed to get state: {}", e)))?;

        serialize_state(state)
            .map_err(|e| PmcpError::internal(format!("Failed to serialize state: {}", e)))
    }
}

pub struct RefactorGetStateTool {
    state_manager: Arc<Mutex<StateManager>>,
}

impl RefactorGetStateTool {
    pub fn new(state_manager: Arc<Mutex<StateManager>>) -> Self {
        Self { state_manager }
    }
}

#[async_trait]
impl ToolHandler for RefactorGetStateTool {
    async fn handle(&self, _args: Value) -> PmcpResult<Value> {
        debug!("Handling refactor.getState");

        let manager = self.state_manager.lock().await;
        let state = manager
            .get_state()
            .map_err(|e| PmcpError::internal(format!("Failed to get state: {}", e)))?;

        serialize_state(state)
            .map_err(|e| PmcpError::internal(format!("Failed to serialize state: {}", e)))
    }
}

pub struct RefactorStopTool {
    state_manager: Arc<Mutex<StateManager>>,
}

impl RefactorStopTool {
    pub fn new(state_manager: Arc<Mutex<StateManager>>) -> Self {
        Self { state_manager }
    }
}

#[async_trait]
impl ToolHandler for RefactorStopTool {
    async fn handle(&self, _args: Value) -> PmcpResult<Value> {
        debug!("Handling refactor.stop");

        let mut manager = self.state_manager.lock().await;
        manager
            .stop_session()
            .map_err(|e| PmcpError::internal(format!("Failed to stop session: {}", e)))?;

        Ok(json!({
            "status": "stopped",
            "message": "Refactoring session stopped successfully"
        }))
    }
}

fn serialize_state(state_machine: &crate::models::refactor::RefactorStateMachine) -> Result<Value, Box<dyn std::error::Error>> {
    let state_json = match &state_machine.current {
        crate::models::refactor::State::Scan { targets } => {
            json!({
                "current": "Scan",
                "targets": targets,
                "current_target_index": state_machine.current_target_index,
                "config": state_machine.config
            })
        }
        crate::models::refactor::State::Analyze { current } => {
            json!({
                "current": "Analyze",
                "current_file": current,
                "targets": state_machine.targets,
                "current_target_index": state_machine.current_target_index
            })
        }
        crate::models::refactor::State::Plan { violations } => {
            json!({
                "current": "Plan",
                "violations": violations,
                "targets": state_machine.targets,
                "current_target_index": state_machine.current_target_index
            })
        }
        crate::models::refactor::State::Refactor { operation } => {
            json!({
                "current": "Refactor",
                "operation": operation,
                "targets": state_machine.targets,
                "current_target_index": state_machine.current_target_index
            })
        }
        crate::models::refactor::State::Test { command } => {
            json!({
                "current": "Test",
                "command": command,
                "targets": state_machine.targets,
                "current_target_index": state_machine.current_target_index
            })
        }
        crate::models::refactor::State::Lint { strict } => {
            json!({
                "current": "Lint",
                "strict": strict,
                "targets": state_machine.targets,
                "current_target_index": state_machine.current_target_index
            })
        }
        crate::models::refactor::State::Emit { payload } => {
            json!({
                "current": "Emit",
                "payload": payload,
                "targets": state_machine.targets,
                "current_target_index": state_machine.current_target_index
            })
        }
        crate::models::refactor::State::Checkpoint { reason } => {
            json!({
                "current": "Checkpoint",
                "reason": reason,
                "targets": state_machine.targets,
                "current_target_index": state_machine.current_target_index
            })
        }
        crate::models::refactor::State::Complete { summary } => {
            json!({
                "current": "Complete",
                "summary": summary
            })
        }
    };

    Ok(state_json)
}