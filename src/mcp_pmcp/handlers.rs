use crate::mcp_server::state_manager::StateManager;
use crate::models::refactor::RefactorConfig;
use async_trait::async_trait;
use pmcp::{Error as PmcpError, RequestHandlerExtra, Result as PmcpResult, ToolHandler};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;

// --- Argument/result types ---

/// MCP args for refactor.start: target file paths and optional refactoring configuration.
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

// --- Tool structs and constructors ---

pub struct RefactorStartTool {
    state_manager: Arc<Mutex<StateManager>>,
}

impl RefactorStartTool {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(state_manager: Arc<Mutex<StateManager>>) -> Self {
        Self { state_manager }
    }
}

pub struct RefactorNextIterationTool {
    state_manager: Arc<Mutex<StateManager>>,
}

impl RefactorNextIterationTool {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(state_manager: Arc<Mutex<StateManager>>) -> Self {
        Self { state_manager }
    }
}

pub struct RefactorGetStateTool {
    state_manager: Arc<Mutex<StateManager>>,
}

impl RefactorGetStateTool {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(state_manager: Arc<Mutex<StateManager>>) -> Self {
        Self { state_manager }
    }
}

pub struct RefactorStopTool {
    state_manager: Arc<Mutex<StateManager>>,
}

impl RefactorStopTool {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(state_manager: Arc<Mutex<StateManager>>) -> Self {
        Self { state_manager }
    }
}

// --- Included implementation files ---

include!("handlers_tool_impls.rs");
include!("handlers_serialize.rs");
include!("handlers_tests.rs");
