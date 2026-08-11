use crate::mcp_pmcp::tool_schemas::build_tool_info;
use crate::mcp_server::state_manager::StateManager;
use crate::models::refactor::RefactorConfig;
use async_trait::async_trait;
use pmcp::types::ToolInfo;
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
    #[serde(deserialize_with = "deserialize_existing_targets")]
    targets: Vec<String>,
    config: Option<RefactorConfig>,
}

/// Reject `refactor.start` targets that do not exist on disk.
///
/// `RefactorStateMachine::new` stores its targets verbatim — it never stats
/// them — so `refactor.start` on `/does/not/exist/at/all.rs` opened a session,
/// walked Scan → Analyze → Plan → Complete and reported
/// `files_processed: 0` with `isError: false`, i.e. a successful refactor of a
/// file nothing ever opened. Validating here (the same guard
/// `resolve_existing_paths` gives the path-taking tools) turns that into a
/// JSON-RPC validation error naming the missing paths.
fn deserialize_existing_targets<'de, D>(
    deserializer: D,
) -> std::result::Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let targets = Vec::<String>::deserialize(deserializer)?;
    let missing: Vec<&str> = targets
        .iter()
        .map(String::as_str)
        .filter(|t| !std::path::Path::new(t).exists())
        .collect();
    if !missing.is_empty() {
        return Err(serde::de::Error::custom(format!(
            "refactor.start target(s) do not exist: {}",
            missing.join(", ")
        )));
    }
    Ok(targets)
}

#[derive(Debug, Serialize)]
struct RefactorStartResult {
    session_id: String,
    state: Value,
}

// --- Tool structs and constructors ---

/// Refactor start tool.
pub struct RefactorStartTool {
    state_manager: Arc<Mutex<StateManager>>,
}

impl RefactorStartTool {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new(state_manager: Arc<Mutex<StateManager>>) -> Self {
        Self { state_manager }
    }
}

/// Refactor next iteration tool.
pub struct RefactorNextIterationTool {
    state_manager: Arc<Mutex<StateManager>>,
}

impl RefactorNextIterationTool {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new(state_manager: Arc<Mutex<StateManager>>) -> Self {
        Self { state_manager }
    }
}

/// Refactor get state tool.
pub struct RefactorGetStateTool {
    state_manager: Arc<Mutex<StateManager>>,
}

impl RefactorGetStateTool {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new(state_manager: Arc<Mutex<StateManager>>) -> Self {
        Self { state_manager }
    }
}

/// Refactor stop tool.
pub struct RefactorStopTool {
    state_manager: Arc<Mutex<StateManager>>,
}

impl RefactorStopTool {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new(state_manager: Arc<Mutex<StateManager>>) -> Self {
        Self { state_manager }
    }
}

#[cfg(test)]
mod refactor_start_target_tests {
    //! `refactor.start` must not open a session on a path it never opened.
    use super::RefactorStartArgs;

    #[test]
    fn nonexistent_targets_are_rejected() {
        let err = serde_json::from_value::<RefactorStartArgs>(serde_json::json!({
            "targets": ["/does/not/exist/at/all.rs"],
            "config": null
        }))
        .expect_err("a target that is not on disk must not start a session");
        let msg = err.to_string();
        assert!(msg.contains("do not exist"), "{msg}");
        assert!(msg.contains("/does/not/exist/at/all.rs"), "{msg}");
    }

    #[test]
    fn existing_targets_are_accepted() {
        let dir = tempfile::tempdir().expect("tempdir");
        let file = dir.path().join("lib.rs");
        std::fs::write(&file, "pub fn f() {}\n").expect("write fixture");
        let args = serde_json::from_value::<RefactorStartArgs>(serde_json::json!({
            "targets": [file.to_string_lossy()],
            "config": null
        }))
        .expect("an existing target is accepted");
        assert_eq!(args.targets.len(), 1);
    }
}

// --- Included implementation files ---

include!("handlers_tool_impls.rs");
include!("handlers_serialize.rs");
include!("handlers_tests.rs");
