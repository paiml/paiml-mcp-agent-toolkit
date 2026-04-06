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
/// * `Ok(Value)` - JSON response with `session_id` and serialized initial state
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
/// ```ignore
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
/// ```ignore
///
/// # Examples
///
/// ```rust,no_run
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
/// let response = result.expect("start succeeded");
/// assert!(response.get("session_id").is_some());
/// assert!(response.get("state").is_some());
/// # });
/// ```
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
/// ```rust,no_run
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
/// let _start_result = handle_refactor_start(&state_manager, params).await.expect("start");
///
/// // Advance to next iteration
/// let result = handle_refactor_next_iteration(&state_manager).await;
/// assert!(result.is_ok());
///
/// let new_state = result.expect("advance succeeded");
/// assert!(new_state.is_object());
/// # });
/// ```
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
/// ```rust,no_run
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
/// let _start_result = handle_refactor_start(&state_manager, params).await.expect("start");
///
/// // Get current state
/// let result = handle_refactor_get_state(&state_manager).await;
/// assert!(result.is_ok());
///
/// let state = result.expect("get state succeeded");
/// assert!(state.is_object());
/// # });
/// ```
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
/// ```rust,no_run
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
/// let _start_result = handle_refactor_start(&state_manager, params).await.expect("start");
///
/// // Stop the session
/// let result = handle_refactor_stop(&state_manager).await;
/// assert!(result.is_ok());
///
/// let response = result.expect("stop succeeded");
/// assert_eq!(response["message"], "Refactoring session stopped successfully");
/// # });
/// ```
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
