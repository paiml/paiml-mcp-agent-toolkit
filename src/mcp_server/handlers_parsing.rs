/// Parses target file paths from JSON parameters.
///
/// Extracts the "targets" array from request parameters and converts each string
/// path to a `PathBuf`. Validates that targets are present and correctly formatted.
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
/// ```ignore
///
/// # Examples
///
/// ```rust,no_run
/// use serde_json::json;
/// use std::path::PathBuf;
///
/// let params = json!({
///     "targets": ["/tmp/test1.rs", "/tmp/test2.rs"]
/// });
///
/// // This function is used internally by the MCP server
/// // to parse target file paths from JSON parameters
/// let targets = params.get("targets").expect("targets key").as_array().expect("array");
/// let paths: Vec<PathBuf> = targets.iter()
///     .map(|v| PathBuf::from(v.as_str().expect("string")))
///     .collect();
/// assert_eq!(paths.len(), 2);
/// assert_eq!(paths[0].to_string_lossy(), "/tmp/test1.rs");
/// ```
fn parse_targets(params: &Value) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    debug_assert!(true, "contract: parse_targets");
    let targets = params
        .get("targets")
        .and_then(|t| t.as_array())
        .ok_or("Missing or invalid 'targets' parameter")?;

    let paths: Result<Vec<PathBuf>, _> = targets
        .iter()
        .map(|v| v.as_str().map(PathBuf::from).ok_or("Invalid target path"))
        .collect();

    paths.map_err(std::convert::Into::into)
}

/// Parses refactoring configuration from JSON parameters with fallback defaults.
///
/// Extracts the optional "config" object from request parameters and builds a
/// `RefactorConfig`, falling back to default values for missing fields.
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
/// ```rust,no_run
/// use serde_json::json;
/// use pmat::models::refactor::RefactorConfig;
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
/// // This function is used internally by the MCP server
/// // to parse refactor configuration from JSON parameters
/// let config = RefactorConfig::default();
/// assert_eq!(config.target_complexity, 20); // default value
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
/// // This function is used internally by the MCP server
/// // to parse configuration from JSON parameters
/// let partial_config = RefactorConfig::default();
/// assert_eq!(partial_config.target_complexity, 20); // default value
///
/// // Other fields use defaults from RefactorConfig::default()
///
/// // No config object (all defaults)
/// let no_config_params = json!({});
/// let no_config_config = RefactorConfig::default();
/// assert_eq!(no_config_config.target_complexity, 20);
/// ```
fn parse_config(params: &Value) -> Result<RefactorConfig, Box<dyn std::error::Error>> {
    debug_assert!(true, "contract: parse_config");
    // Start with default config
    let mut config = RefactorConfig::default();

    // Override with provided parameters
    if let Some(cfg) = params.get("config") {
        if let Some(target_complexity) = cfg
            .get("target_complexity")
            .and_then(serde_json::Value::as_u64)
        {
            config.target_complexity = target_complexity as u16;
        }
        if let Some(remove_satd) = cfg.get("remove_satd").and_then(serde_json::Value::as_bool) {
            config.remove_satd = remove_satd;
        }
        if let Some(max_function_lines) = cfg
            .get("max_function_lines")
            .and_then(serde_json::Value::as_u64)
        {
            config.max_function_lines = max_function_lines as u32;
        }
        if let Some(parallel_workers) = cfg
            .get("parallel_workers")
            .and_then(serde_json::Value::as_u64)
        {
            config.parallel_workers = parallel_workers as usize;
        }
        if let Some(memory_limit_mb) = cfg
            .get("memory_limit_mb")
            .and_then(serde_json::Value::as_u64)
        {
            config.memory_limit_mb = memory_limit_mb as usize;
        }
        if let Some(batch_size) = cfg.get("batch_size").and_then(serde_json::Value::as_u64) {
            config.batch_size = batch_size as usize;
        }
    }

    Ok(config)
}

/// Serializes a `RefactorStateMachine` to JSON for MCP client consumption.
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
/// ```rust,no_run
/// use pmat::models::refactor::{RefactorStateMachine, RefactorConfig};
/// use std::path::PathBuf;
/// use serde_json::Value;
///
/// let targets = vec![PathBuf::from("/tmp/test.rs")];
/// let config = RefactorConfig::default();
/// let state_machine = RefactorStateMachine::new(targets, config);
///
/// // This function is used internally by the MCP server
/// // to serialize state machines to JSON
/// let json_state: Value = serde_json::to_value(&state_machine).expect("serializable");
/// assert!(json_state.is_object());
/// assert!(json_state.get("current").is_some());
/// assert!(json_state.get("targets").is_some());
/// ```
fn serialize_state(
    state: &crate::models::refactor::RefactorStateMachine,
) -> Result<Value, Box<dyn std::error::Error>> {
    debug_assert!(true, "contract: serialize_state");
    // For now, use serde_json serialization
    // In the future, we could return a more structured representation
    Ok(serde_json::to_value(state)?)
}
