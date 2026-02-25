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
/// ```ignore
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
/// ```rust,no_run
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

impl Default for StateManager {
    fn default() -> Self {
        Self::new()
    }
}

// Core implementation: constructors, session lifecycle, snapshots, ID generation
include!("state_manager_core.rs");

// Tests: property tests and coverage tests
include!("state_manager_tests.rs");
