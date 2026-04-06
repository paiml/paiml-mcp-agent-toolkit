#![cfg_attr(coverage_nightly, coverage(off))]
// DAP (Debug Adapter Protocol) Server Implementation
// Sprint 71 - TRACE-001: DAP Protocol Server Implementation
// Sprint 71 - TRACE-004: DAP-PMAT Integration
// Sprint 76 - CAPTURE-002: DAP Server Recording Capture
//
// This implements a Debug Adapter Protocol server for PMAT
// allowing integration with VSCode and other DAP-compatible debuggers
//
// Sprint 76: Now supports optional recording to .pmat files during debug sessions

use super::execution_recorder::ExecutionRecorder;
use super::types::*;
use super::variable_inspector::VariableInspector;
use crate::cli::language_analyzer::Language;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tree_sitter::Tree;

/// Server state enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerState {
    Uninitialized,
    Initialized,
    Running,
    Stopped,
}

/// DAP Server structure
pub struct DapServer {
    /// Current server state
    state: Arc<Mutex<ServerState>>,
    /// Response sequence counter
    response_seq: Arc<Mutex<i64>>,
    /// Server capabilities
    capabilities: DapCapabilities,
    /// Current program being debugged
    program: Arc<Mutex<Option<String>>>,
    /// Breakpoints storage
    breakpoints: Arc<Mutex<HashMap<String, HashSet<i64>>>>,
    /// TRACE-004: Language detection integration
    current_language: Arc<Mutex<Option<Language>>>,
    /// TRACE-004: AST cache for tree-sitter parse trees
    ast_cache: Arc<Mutex<HashMap<PathBuf, Tree>>>,
    /// TRACE-004: Variable inspector for extracting variables
    variable_inspector: VariableInspector,
    /// TRACE-004: Current stopped file (for simulation)
    current_stopped_file: Arc<Mutex<Option<String>>>,
    /// TRACE-004: Current stopped line (for simulation)
    current_stopped_line: Arc<Mutex<Option<usize>>>,
    /// CAPTURE-002: Optional recording directory
    recording_dir: Option<PathBuf>,
    /// CAPTURE-002: Current recording file path
    recording_path: Arc<Mutex<Option<PathBuf>>>,
    /// CAPTURE-002: Execution recorder for snapshot capture
    execution_recorder: Arc<Mutex<Option<ExecutionRecorder<File>>>>,
}

impl DapServer {
    /// Create a new DAP server instance without recording
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self::with_recording(None)
    }

    /// Create a new DAP server instance with optional recording
    ///
    /// Sprint 76 - CAPTURE-002: Enable recording capture to .pmat files
    ///
    /// # Arguments
    /// * `recording_dir` - Optional directory to save recording files
    ///
    /// # Example
    /// ```rust,no_run
    /// use pmat::services::dap::server::DapServer;
    /// use std::path::PathBuf;
    ///
    /// // Without recording
    /// let server1 = DapServer::new();
    ///
    /// // With recording
    /// let server2 = DapServer::with_recording(Some(PathBuf::from("./recordings")));
    /// ```
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn with_recording(recording_dir: Option<PathBuf>) -> Self {
        Self {
            state: Arc::new(Mutex::new(ServerState::Uninitialized)),
            response_seq: Arc::new(Mutex::new(0)),
            capabilities: Self::default_capabilities(),
            program: Arc::new(Mutex::new(None)),
            breakpoints: Arc::new(Mutex::new(HashMap::new())),
            current_language: Arc::new(Mutex::new(None)),
            ast_cache: Arc::new(Mutex::new(HashMap::new())),
            variable_inspector: VariableInspector::new(),
            current_stopped_file: Arc::new(Mutex::new(None)),
            current_stopped_line: Arc::new(Mutex::new(None)),
            recording_dir,
            recording_path: Arc::new(Mutex::new(None)),
            execution_recorder: Arc::new(Mutex::new(None)),
        }
    }

    /// Get default capabilities
    fn default_capabilities() -> DapCapabilities {
        debug_assert!(true, "contract: default_capabilities");
        DapCapabilities {
            supports_configuration_done_request: true,
            supports_function_breakpoints: false,
            supports_conditional_breakpoints: true,
            supports_hit_conditional_breakpoints: false,
            supports_evaluate_for_hovers: false,
            supports_step_back: false,
            supports_set_variable: false,
            supports_restart_frame: false,
            supports_goto_targets_request: false,
            supports_step_in_targets_request: false,
            supports_completions_request: false,
            supports_modules_request: false,
            supports_restart_request: false,
            supports_exception_options: false,
            supports_value_formatting_options: false,
            supports_exception_info_request: false,
            supports_terminate_debuggee: true,
            supports_delayed_stack_trace_loading: false,
            supports_loaded_sources_request: false,
            supports_log_points: false,
            supports_terminate_threads_request: false,
            supports_set_expression: false,
            supports_terminate_request: true,
            supports_data_breakpoints: false,
            supports_read_memory_request: false,
            supports_write_memory_request: false,
            supports_disassemble_request: false,
            supports_cancel_request: false,
            supports_breakpoint_locations_request: false,
            supports_clipboard_context: false,
            supports_stepping_granularity: false,
            supports_instruction_breakpoints: false,
            supports_exception_filter_options: false,
        }
    }

    /// Get next response sequence number
    fn next_seq(&self) -> i64 {
        let mut seq = self
            .response_seq
            .lock()
            .expect("Mutex should not be poisoned");
        *seq += 1;
        *seq
    }

    /// Check if server is initialized
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn is_initialized(&self) -> bool {
        let state = self.state.lock().expect("Mutex should not be poisoned");
        *state != ServerState::Uninitialized
    }

    /// Check if server is running
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn is_running(&self) -> bool {
        let state = self.state.lock().expect("Mutex should not be poisoned");
        *state == ServerState::Running
    }

    /// Check if program is loaded
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn has_program_loaded(&self) -> bool {
        let program = self.program.lock().expect("Mutex should not be poisoned");
        program.is_some()
    }

    /// Get current program path
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn current_program(&self) -> Option<String> {
        let program = self.program.lock().expect("Mutex should not be poisoned");
        program.clone()
    }
}

// Request handler methods (handle_request, handle_initialize, handle_launch, etc.)
include!("server_request_handlers.rs");

// PMAT integration methods (language detection, AST parsing, variable inspection)
include!("server_pmat_integration.rs");

// Recording capture and TCP server methods
include!("server_recording.rs");

impl Default for DapServer {
    fn default() -> Self {
        Self::new()
    }
}

// Tests extracted to server_tests.rs for file health compliance (CB-040)
#[cfg(test)]
#[path = "server_tests.rs"]
mod tests;
