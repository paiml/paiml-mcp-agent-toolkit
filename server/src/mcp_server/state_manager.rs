use crate::mcp_server::snapshots::SnapshotManager;
use crate::models::refactor::{RefactorConfig, RefactorStateMachine};
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// State manager for MCP refactoring sessions with persistence and recovery.
///
/// This component manages the lifecycle of refactoring sessions in the MCP server,
/// providing state persistence, snapshot management, and session isolation.
/// Critical for maintaining session consistency and preventing state drift across
/// MCP protocol interactions.
///
/// # Features
///
/// - **Session Management**: Start, stop, and track refactoring sessions
/// - **State Persistence**: Automatic snapshots for crash recovery
/// - **Session Isolation**: Each session has unique ID and isolated state
/// - **State Machine Control**: Advance through refactoring phases
/// - **Error Recovery**: Graceful handling of state transition failures
///
/// # Session Lifecycle
///
/// ```text
/// New StateManager → start_session() → Active Session → advance() → Complete
///                                   ↓                    ↑
///                                   └─── stop_session() ──┘
/// ```
///
/// # State Machine Phases
///
/// 1. **Scan**: Discover files and build initial analysis
/// 2. **Analyze**: Compute complexity and quality metrics
/// 3. **Plan**: Generate refactoring operations
/// 4. **Refactor**: Apply transformations
/// 5. **Complete**: Finalize and cleanup
///
/// # Examples
///
/// ```rust
/// use pmat::mcp_server::state_manager::StateManager;
/// use pmat::models::refactor::RefactorConfig;
/// use std::path::PathBuf;
///
/// // Create state manager
/// let mut manager = StateManager::new();
///
/// // Start refactoring session
/// let targets = vec![PathBuf::from("/tmp/test.rs")];
/// let config = RefactorConfig::default();
/// let result = manager.start_session(targets, config);
/// assert!(result.is_ok());
///
/// // Get session info
/// let session_id = manager.get_session_id();
/// assert!(session_id.starts_with("refactor-session-"));
///
/// // Session state is available
/// let state = manager.get_state();
/// assert!(state.is_ok());
///
/// // Stop session
/// let stop_result = manager.stop_session();
/// assert!(stop_result.is_ok());
/// ```
pub struct StateManager {
    state: Option<RefactorStateMachine>,
    snapshot_manager: SnapshotManager,
    session_id: String,
}

impl StateManager {
    /// Creates a new state manager with default configuration.
    ///
    /// Initializes the state manager with no active session, a new snapshot
    /// manager for persistence, and a unique session ID ready for the next
    /// refactoring session.
    ///
    /// # Returns
    ///
    /// A new `StateManager` instance ready to manage refactoring sessions.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::mcp_server::state_manager::StateManager;
    ///
    /// let manager = StateManager::new();
    ///
    /// // Manager is ready but has no active session
    /// assert!(manager.get_state().is_err());
    ///
    /// // Session ID is pre-generated
    /// let session_id = manager.get_session_id();
    /// assert!(session_id.starts_with("refactor-session-"));
    /// ```
    pub fn new() -> Self {
        Self {
            state: None,
            snapshot_manager: SnapshotManager::new(),
            session_id: Self::generate_session_id(),
        }
    }

    pub fn with_temp_dir(temp_dir: &Path) -> Self {
        Self {
            state: None,
            snapshot_manager: SnapshotManager::with_path(temp_dir),
            session_id: Self::generate_session_id(),
        }
    }

    /// Starts a new refactoring session with specified targets and configuration.
    ///
    /// Creates a new refactoring state machine, generates a unique session ID,
    /// and saves an initial snapshot for recovery. Ensures only one session
    /// is active at a time to maintain state consistency.
    ///
    /// # Parameters
    ///
    /// * `targets` - Vector of file paths to include in the refactoring session
    /// * `config` - Refactoring configuration (complexity thresholds, etc.)
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Session started successfully
    /// * `Err(String)` - Session already active or configuration invalid
    ///
    /// # Session Management
    ///
    /// - Validates no existing session is active
    /// - Creates new state machine with provided targets
    /// - Generates unique session ID with timestamp
    /// - Saves initial state snapshot for recovery
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::mcp_server::state_manager::StateManager;
    /// use pmat::models::refactor::RefactorConfig;
    /// use std::path::PathBuf;
    ///
    /// let mut manager = StateManager::new();
    ///
    /// // Start session with multiple files
    /// let targets = vec![
    ///     PathBuf::from("/project/src/main.rs"),
    ///     PathBuf::from("/project/src/lib.rs"),
    /// ];
    /// let config = RefactorConfig {
    ///     target_complexity: 15,
    ///     remove_satd: true,
    ///     max_function_lines: 50,
    ///     parallel_workers: 4,
    ///     memory_limit_mb: 512,
    ///     batch_size: 10,
    /// };
    ///
    /// let result = manager.start_session(targets, config);
    /// assert!(result.is_ok());
    ///
    /// // Session is now active
    /// assert!(manager.get_state().is_ok());
    ///
    /// // Cannot start another session while one is active
    /// let duplicate_result = manager.start_session(vec![], RefactorConfig::default());
    /// assert!(duplicate_result.is_err());
    /// ```
    ///
    /// # MCP Protocol Integration
    ///
    /// This method is typically called from the `refactor.start` MCP handler:
    ///
    /// ```json
    /// {
    ///   "jsonrpc": "2.0",
    ///   "method": "refactor.start",
    ///   "params": {
    ///     "targets": ["/path/to/file.rs"],
    ///     "config": {
    ///       "target_complexity": 15,
    ///       "remove_satd": true
    ///     }
    ///   }
    /// }
    /// ```
    pub fn start_session(
        &mut self,
        targets: Vec<PathBuf>,
        config: RefactorConfig,
    ) -> Result<(), String> {
        if self.state.is_some() {
            return Err(
                "Session already active. Stop current session before starting a new one."
                    .to_string(),
            );
        }

        info!(
            "Starting new refactor session with {} targets",
            targets.len()
        );

        self.state = Some(RefactorStateMachine::new(targets, config));
        self.session_id = Self::generate_session_id();

        // Save initial state
        self.save_snapshot()?;

        Ok(())
    }

    /// Advances the refactoring state machine to the next phase.
    ///
    /// Transitions the active refactoring session through its lifecycle phases,
    /// automatically saving snapshots after each successful transition for
    /// crash recovery and state persistence.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - State advanced successfully
    /// * `Err(String)` - No active session or state transition failed
    ///
    /// # State Transitions
    ///
    /// The state machine follows this progression:
    /// 1. **Scan** → **Analyze**: Discovery complete, begin analysis
    /// 2. **Analyze** → **Plan**: Metrics computed, generate refactoring plan
    /// 3. **Plan** → **Refactor**: Operations planned, apply transformations
    /// 4. **Refactor** → **Complete**: Transformations applied, finalize
    ///
    /// # Persistence
    ///
    /// Each successful state transition triggers:
    /// - Automatic snapshot save for crash recovery
    /// - State validation and consistency checks
    /// - Progress tracking and metrics update
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::mcp_server::state_manager::StateManager;
    /// use pmat::models::refactor::RefactorConfig;
    /// use std::path::PathBuf;
    ///
    /// let mut manager = StateManager::new();
    ///
    /// // Start session first
    /// let targets = vec![PathBuf::from("/tmp/test.rs")];
    /// let config = RefactorConfig::default();
    /// manager.start_session(targets, config).unwrap();
    ///
    /// // Advance through state machine phases
    /// let advance1 = manager.advance();
    /// assert!(advance1.is_ok());
    ///
    /// let advance2 = manager.advance();
    /// assert!(advance2.is_ok());
    ///
    /// // Can continue advancing until Complete state
    /// // Each advancement saves a recovery snapshot
    /// ```
    ///
    /// # MCP Protocol Integration
    ///
    /// This method is called from the `refactor.nextIteration` MCP handler:
    ///
    /// ```json
    /// {
    ///   "jsonrpc": "2.0",
    ///   "method": "refactor.nextIteration",
    ///   "params": {}
    /// }
    /// ```
    ///
    /// # Error Handling
    ///
    /// - **No Active Session**: Returns error if no session is running
    /// - **Invalid Transition**: Returns error for illegal state transitions
    /// - **Snapshot Failure**: Returns error if state persistence fails
    /// - **File Access**: Returns error if target files are inaccessible
    pub fn advance(&mut self) -> Result<(), String> {
        let state = self.state.as_mut().ok_or("No active session")?;
        state.advance()?;

        // Save after each state transition
        self.save_snapshot()?;

        Ok(())
    }

    pub fn get_state(&self) -> Result<&RefactorStateMachine, String> {
        self.state.as_ref().ok_or("No active session".to_string())
    }

    pub fn get_session_id(&self) -> &str {
        &self.session_id
    }

    pub fn stop_session(&mut self) -> Result<(), String> {
        if self.state.is_none() {
            return Err("No active session to stop".to_string());
        }

        info!("Stopping refactor session");

        // Clear in-memory state
        self.state = None;

        // Remove snapshot file
        self.snapshot_manager.remove_snapshot()?;

        Ok(())
    }

    fn save_snapshot(&self) -> Result<(), String> {
        if let Some(state) = &self.state {
            self.snapshot_manager.save_snapshot(state)?;
        }
        Ok(())
    }

    #[allow(dead_code)]
    fn load_from_snapshot(&mut self) -> Result<(), String> {
        match self.snapshot_manager.load_snapshot() {
            Ok(state) => {
                self.state = Some(state);
                info!("Loaded existing refactor state from snapshot");
                Ok(())
            }
            Err(e) => {
                warn!("Failed to load snapshot: {}", e);
                Err(e)
            }
        }
    }

    fn generate_session_id() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        format!("refactor-session-{}", timestamp)
    }
}

impl Default for StateManager {
    fn default() -> Self {
        Self::new()
    }
}
