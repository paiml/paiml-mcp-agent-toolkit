use crate::mcp_server::state_manager::StateManager;
use crate::models::refactor::RefactorConfig;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;

/// Initiates a new refactoring session with specified targets and configuration.
///
/// This handler processes MCP refactor.start requests by parsing target files and
/// configuration parameters, then starting a new refactoring session in the state manager.
/// Returns the session ID and initial state for client tracking.
///
/// # Parameters
///
/// * `state_manager` - Shared state manager for coordinating refactoring sessions
/// * `params` - JSON parameters containing targets array and optional config object
///
/// # Returns
///
/// * `Ok(Value)` - JSON response with session_id and serialized initial state
/// * `Err(Box<dyn std::error::Error>)` - Parse errors, state manager errors, or serialization failures
///
/// # JSON Parameters
///
/// ```json
/// {
///   "targets": ["/path/to/file1.rs", "/path/to/file2.rs"],
///   "config": {
///     "target_complexity": 15,
///     "remove_satd": true,
///     "max_function_lines": 50,
///     "parallel_workers": 4,
///     "memory_limit_mb": 512,
///     "batch_size": 10
///   }
/// }
/// ```
///
/// # Response Format
///
/// ```json
/// {
///   "session_id": "uuid-string",
///   "state": {
///     "current": "Scan",
///     "targets": ["/path/to/file1.rs"],
///     "current_target_index": 0,
///     "summary": {...}
///   }
/// }
/// ```
///
/// # Examples
///
/// ```rust
/// use pmat::mcp_server::handlers::handle_refactor_start;
/// use pmat::mcp_server::state_manager::StateManager;
/// use serde_json::json;
/// use std::sync::Arc;
/// use tokio::sync::Mutex;
///
/// # tokio_test::block_on(async {
/// let state_manager = Arc::new(Mutex::new(StateManager::new()));
/// let params = json!({
///     "targets": ["/tmp/test.rs"],
///     "config": {
///         "target_complexity": 10,
///         "remove_satd": true
///     }
/// });
///
/// let result = handle_refactor_start(&state_manager, params).await;
/// assert!(result.is_ok());
///
/// let response = result.unwrap();
/// assert!(response.get("session_id").is_some());
/// assert!(response.get("state").is_some());
/// # });
/// ```
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

/// Advances the current refactoring session to the next iteration.
///
/// This handler processes MCP refactor.nextIteration requests by advancing the
/// state machine to the next step in the refactoring process. Used for iterative
/// refactoring where clients control the pace of execution.
///
/// # Parameters
///
/// * `state_manager` - Shared state manager containing the current session
///
/// # Returns
///
/// * `Ok(Value)` - JSON serialized state after advancing
/// * `Err(Box<dyn std::error::Error>)` - State manager errors or serialization failures
///
/// # State Machine Transitions
///
/// The state machine follows this progression:
/// Scan → Analyze → Plan → Refactor → Complete
///
/// # Examples
///
/// ```rust
/// use pmat::mcp_server::handlers::{handle_refactor_start, handle_refactor_next_iteration};
/// use pmat::mcp_server::state_manager::StateManager;
/// use serde_json::json;
/// use std::sync::Arc;
/// use tokio::sync::Mutex;
///
/// # tokio_test::block_on(async {
/// let state_manager = Arc::new(Mutex::new(StateManager::new()));
///
/// // Start a session first
/// let params = json!({
///     "targets": ["/tmp/test.rs"],
///     "config": {"target_complexity": 10}
/// });
/// let _start_result = handle_refactor_start(&state_manager, params).await.unwrap();
///
/// // Advance to next iteration
/// let result = handle_refactor_next_iteration(&state_manager).await;
/// assert!(result.is_ok());
///
/// let new_state = result.unwrap();
/// assert!(new_state.is_object());
/// # });
/// ```
pub async fn handle_refactor_next_iteration(
    state_manager: &Arc<Mutex<StateManager>>,
) -> Result<Value, Box<dyn std::error::Error>> {
    debug!("Handling refactor.nextIteration");

    let mut manager = state_manager.lock().await;
    manager.advance()?;

    let state = manager.get_state()?;

    serialize_state(state)
}

/// Retrieves the current state of the active refactoring session.
///
/// This handler processes MCP refactor.getState requests by returning the current
/// state of the refactoring state machine. Used for client synchronization and
/// debugging without advancing the state.
///
/// # Parameters
///
/// * `state_manager` - Shared state manager containing the current session
///
/// # Returns
///
/// * `Ok(Value)` - JSON serialized current state
/// * `Err(Box<dyn std::error::Error>)` - State manager errors or serialization failures
///
/// # State Information
///
/// The returned state includes:
/// - Current state type (Scan, Analyze, Plan, Refactor, Complete)
/// - Target files and current index
/// - Refactoring summary and progress
/// - State transition history
///
/// # Examples
///
/// ```rust
/// use pmat::mcp_server::handlers::{handle_refactor_start, handle_refactor_get_state};
/// use pmat::mcp_server::state_manager::StateManager;
/// use serde_json::json;
/// use std::sync::Arc;
/// use tokio::sync::Mutex;
///
/// # tokio_test::block_on(async {
/// let state_manager = Arc::new(Mutex::new(StateManager::new()));
///
/// // Start a session first
/// let params = json!({
///     "targets": ["/tmp/test.rs"],
///     "config": {"target_complexity": 10}
/// });
/// let _start_result = handle_refactor_start(&state_manager, params).await.unwrap();
///
/// // Get current state
/// let result = handle_refactor_get_state(&state_manager).await;
/// assert!(result.is_ok());
///
/// let state = result.unwrap();
/// assert!(state.is_object());
/// # });
/// ```
pub async fn handle_refactor_get_state(
    state_manager: &Arc<Mutex<StateManager>>,
) -> Result<Value, Box<dyn std::error::Error>> {
    debug!("Handling refactor.getState");

    let manager = state_manager.lock().await;
    let state = manager.get_state()?;

    serialize_state(state)
}

/// Stops the current refactoring session and cleans up resources.
///
/// This handler processes MCP refactor.stop requests by terminating the active
/// refactoring session and clearing session state. Returns a confirmation message
/// when successful.
///
/// # Parameters
///
/// * `state_manager` - Shared state manager containing the current session
///
/// # Returns
///
/// * `Ok(Value)` - JSON response with success message
/// * `Err(Box<dyn std::error::Error>)` - State manager errors or session cleanup failures
///
/// # Session Cleanup
///
/// Stopping a session will:
/// - Clear the current state machine
/// - Reset target file tracking
/// - Clean up any temporary resources
/// - Invalidate the session ID
///
/// # Examples
///
/// ```rust
/// use pmat::mcp_server::handlers::{handle_refactor_start, handle_refactor_stop};
/// use pmat::mcp_server::state_manager::StateManager;
/// use serde_json::json;
/// use std::sync::Arc;
/// use tokio::sync::Mutex;
///
/// # tokio_test::block_on(async {
/// let state_manager = Arc::new(Mutex::new(StateManager::new()));
///
/// // Start a session first
/// let params = json!({
///     "targets": ["/tmp/test.rs"],
///     "config": {"target_complexity": 10}
/// });
/// let _start_result = handle_refactor_start(&state_manager, params).await.unwrap();
///
/// // Stop the session
/// let result = handle_refactor_stop(&state_manager).await;
/// assert!(result.is_ok());
///
/// let response = result.unwrap();
/// assert_eq!(response["message"], "Refactoring session stopped successfully");
/// # });
/// ```
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

/// Parses target file paths from JSON parameters.
///
/// Extracts the "targets" array from request parameters and converts each string
/// path to a PathBuf. Validates that targets are present and correctly formatted.
///
/// # Parameters
///
/// * `params` - JSON value containing the request parameters
///
/// # Returns
///
/// * `Ok(Vec<PathBuf>)` - Vector of parsed file paths
/// * `Err(Box<dyn std::error::Error>)` - Missing targets, invalid array, or invalid paths
///
/// # Expected JSON Format
///
/// ```json
/// {
///   "targets": ["/path/to/file1.rs", "/path/to/file2.rs"]
/// }
/// ```
///
/// # Examples
///
/// ```rust
/// use pmat::mcp_server::handlers::parse_targets;
/// use serde_json::json;
///
/// let params = json!({
///     "targets": ["/tmp/test1.rs", "/tmp/test2.rs"]
/// });
///
/// let result = parse_targets(&params);
/// assert!(result.is_ok());
///
/// let paths = result.unwrap();
/// assert_eq!(paths.len(), 2);
/// assert_eq!(paths[0].to_string_lossy(), "/tmp/test1.rs");
/// ```
fn parse_targets(params: &Value) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let targets = params
        .get("targets")
        .and_then(|t| t.as_array())
        .ok_or("Missing or invalid 'targets' parameter")?;

    let paths: Result<Vec<PathBuf>, _> = targets
        .iter()
        .map(|v| v.as_str().map(PathBuf::from).ok_or("Invalid target path"))
        .collect();

    paths.map_err(|e| e.into())
}

/// Parses refactoring configuration from JSON parameters with fallback defaults.
///
/// Extracts the optional "config" object from request parameters and builds a
/// RefactorConfig, falling back to default values for missing fields.
/// All configuration fields are optional.
///
/// # Parameters
///
/// * `params` - JSON value containing the request parameters
///
/// # Returns
///
/// * `Ok(RefactorConfig)` - Parsed configuration with defaults applied
/// * `Err(Box<dyn std::error::Error>)` - Type conversion errors for invalid config values
///
/// # Configuration Fields
///
/// - `target_complexity`: u16 - Maximum acceptable cyclomatic complexity
/// - `remove_satd`: bool - Whether to remove SATD (TODO/FIXME) comments
/// - `max_function_lines`: u32 - Maximum lines per function before extraction
/// - `parallel_workers`: usize - Number of parallel processing workers
/// - `memory_limit_mb`: usize - Memory limit in megabytes
/// - `batch_size`: usize - Batch size for processing operations
///
/// # Examples
///
/// ```rust
/// use pmat::mcp_server::handlers::parse_config;
/// use serde_json::json;
///
/// // Full configuration
/// let params = json!({
///     "config": {
///         "target_complexity": 15,
///         "remove_satd": true,
///         "max_function_lines": 50,
///         "parallel_workers": 4,
///         "memory_limit_mb": 512,
///         "batch_size": 10
///     }
/// });
///
/// let result = parse_config(&params);
/// assert!(result.is_ok());
///
/// let config = result.unwrap();
/// assert_eq!(config.target_complexity, 15);
/// assert_eq!(config.remove_satd, true);
/// assert_eq!(config.max_function_lines, 50);
///
/// // Partial configuration (uses defaults)
/// let partial_params = json!({
///     "config": {
///         "target_complexity": 10
///     }
/// });
///
/// let partial_result = parse_config(&partial_params);
/// assert!(partial_result.is_ok());
///
/// let partial_config = partial_result.unwrap();
/// assert_eq!(partial_config.target_complexity, 10);
/// // Other fields use defaults from RefactorConfig::default()
///
/// // No config object (all defaults)
/// let no_config_params = json!({});
/// let no_config_result = parse_config(&no_config_params);
/// assert!(no_config_result.is_ok());
/// ```
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

/// Serializes a RefactorStateMachine to JSON for MCP client consumption.
///
/// Converts the internal state machine representation to a JSON format suitable
/// for transmission over the MCP protocol. Handles serialization errors gracefully.
///
/// # Parameters
///
/// * `state` - Reference to the refactor state machine to serialize
///
/// # Returns
///
/// * `Ok(Value)` - JSON representation of the state machine
/// * `Err(Box<dyn std::error::Error>)` - Serialization errors
///
/// # JSON Structure
///
/// The serialized state includes:
/// - Current state type and data
/// - Target files and processing index
/// - Refactoring summary and metrics
/// - State transition history
/// - Configuration settings
///
/// # Examples
///
/// ```rust
/// use pmat::mcp_server::handlers::serialize_state;
/// use pmat::models::refactor::{RefactorStateMachine, RefactorConfig};
/// use std::path::PathBuf;
///
/// let targets = vec![PathBuf::from("/tmp/test.rs")];
/// let config = RefactorConfig::default();
/// let state_machine = RefactorStateMachine::new(targets, config);
///
/// let result = serialize_state(&state_machine);
/// assert!(result.is_ok());
///
/// let json_state = result.unwrap();
/// assert!(json_state.is_object());
/// assert!(json_state.get("current").is_some());
/// assert!(json_state.get("targets").is_some());
/// ```
fn serialize_state(
    state: &crate::models::refactor::RefactorStateMachine,
) -> Result<Value, Box<dyn std::error::Error>> {
    // For now, use serde_json serialization
    // In the future, we could return a more structured representation
    Ok(serde_json::to_value(state)?)
}
