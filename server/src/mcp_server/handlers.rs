use crate::mcp_server::state_manager::StateManager;
use crate::models::refactor::RefactorConfig;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;

pub async fn handle_refactor_start(
    state_manager: &Arc<Mutex<StateManager>>,
    params: Value,
) -> Result<Value, Box<dyn std::error::Error>> {
    debug!("Handling refactor.start with params: {}", params);

    // Parse parameters
    let targets = parse_targets(&params)?;
    let config = parse_config(&params)?;

    // Start new session
    let mut manager = state_manager.lock().await;
    manager.start_session(targets, config)?;

    // Get initial state
    let state = manager.get_state()?;
    let session_id = manager.get_session_id().to_string();

    Ok(json!({
        "session_id": session_id,
        "state": serialize_state(state)?
    }))
}

pub async fn handle_refactor_next_iteration(
    state_manager: &Arc<Mutex<StateManager>>,
) -> Result<Value, Box<dyn std::error::Error>> {
    debug!("Handling refactor.nextIteration");

    let mut manager = state_manager.lock().await;
    manager.advance()?;

    let state = manager.get_state()?;

    serialize_state(state)
}

pub async fn handle_refactor_get_state(
    state_manager: &Arc<Mutex<StateManager>>,
) -> Result<Value, Box<dyn std::error::Error>> {
    debug!("Handling refactor.getState");

    let manager = state_manager.lock().await;
    let state = manager.get_state()?;

    serialize_state(state)
}

pub async fn handle_refactor_stop(
    state_manager: &Arc<Mutex<StateManager>>,
) -> Result<Value, Box<dyn std::error::Error>> {
    debug!("Handling refactor.stop");

    let mut manager = state_manager.lock().await;
    manager.stop_session()?;

    Ok(json!({
        "message": "Refactoring session stopped successfully"
    }))
}

fn parse_targets(params: &Value) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let targets = params
        .get("targets")
        .and_then(|t| t.as_array())
        .ok_or("Missing or invalid 'targets' parameter")?;

    let paths: Result<Vec<PathBuf>, _> = targets
        .iter()
        .map(|v| {
            v.as_str()
                .map(PathBuf::from)
                .ok_or("Invalid target path")
        })
        .collect();

    paths.map_err(|e| e.into())
}

fn parse_config(params: &Value) -> Result<RefactorConfig, Box<dyn std::error::Error>> {
    // Start with default config
    let mut config = RefactorConfig::default();

    // Override with provided parameters
    if let Some(cfg) = params.get("config") {
        if let Some(target_complexity) = cfg.get("target_complexity").and_then(|v| v.as_u64()) {
            config.target_complexity = target_complexity as u16;
        }
        if let Some(remove_satd) = cfg.get("remove_satd").and_then(|v| v.as_bool()) {
            config.remove_satd = remove_satd;
        }
        if let Some(max_function_lines) = cfg.get("max_function_lines").and_then(|v| v.as_u64()) {
            config.max_function_lines = max_function_lines as u32;
        }
        if let Some(parallel_workers) = cfg.get("parallel_workers").and_then(|v| v.as_u64()) {
            config.parallel_workers = parallel_workers as usize;
        }
        if let Some(memory_limit_mb) = cfg.get("memory_limit_mb").and_then(|v| v.as_u64()) {
            config.memory_limit_mb = memory_limit_mb as usize;
        }
        if let Some(batch_size) = cfg.get("batch_size").and_then(|v| v.as_u64()) {
            config.batch_size = batch_size as usize;
        }
    }

    Ok(config)
}

fn serialize_state(state: &crate::models::refactor::RefactorStateMachine) -> Result<Value, Box<dyn std::error::Error>> {
    // For now, use serde_json serialization
    // In the future, we could return a more structured representation
    Ok(serde_json::to_value(state)?)
}