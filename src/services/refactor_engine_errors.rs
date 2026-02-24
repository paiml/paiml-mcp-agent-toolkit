/// Comprehensive error handling for the unified refactoring engine.
///
/// This enum covers all possible failure modes during refactoring operations,
/// from state machine transitions to I/O operations and code analysis.
/// Each variant provides detailed context about the specific failure.
///
/// # Error Recovery
///
/// The engine implements different recovery strategies based on error type:
/// - **`StateMachine` errors**: Rollback to last checkpoint
/// - **IO errors**: Retry with exponential backoff
/// - **Serialization errors**: Graceful degradation to simplified format
/// - **Analysis errors**: Skip problematic files and continue
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::services::refactor_engine::EngineError;
///
/// // State machine errors
/// let state_error = EngineError::StateMachine(
///     "Invalid transition from Analyze to Complete".to_string()
/// );
/// assert_eq!(
///     state_error.to_string(),
///     "State machine error: Invalid transition from Analyze to Complete"
/// );
///
/// // IO errors are automatically converted
/// let io_error: EngineError = std::io::Error::new(
///     std::io::ErrorKind::NotFound,
///     "File not found"
/// ).into();
/// assert!(io_error.to_string().contains("IO error:"));
///
/// // Analysis errors with context
/// let analysis_error = EngineError::Analysis(
///     "Failed to parse AST: unexpected token".to_string()
/// );
/// assert!(analysis_error.to_string().contains("Analysis error:"));
/// ```
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("State machine error: {0}")]
    StateMachine(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Analysis error: {0}")]
    Analysis(String),
}

impl From<String> for EngineError {
    fn from(s: String) -> Self {
        EngineError::StateMachine(s)
    }
}
