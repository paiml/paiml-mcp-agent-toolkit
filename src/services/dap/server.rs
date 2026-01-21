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
    pub fn is_initialized(&self) -> bool {
        let state = self.state.lock().expect("Mutex should not be poisoned");
        *state != ServerState::Uninitialized
    }

    /// Check if server is running
    pub fn is_running(&self) -> bool {
        let state = self.state.lock().expect("Mutex should not be poisoned");
        *state == ServerState::Running
    }

    /// Check if program is loaded
    pub fn has_program_loaded(&self) -> bool {
        let program = self.program.lock().expect("Mutex should not be poisoned");
        program.is_some()
    }

    /// Get current program path
    pub fn current_program(&self) -> Option<String> {
        let program = self.program.lock().expect("Mutex should not be poisoned");
        program.clone()
    }

    /// Handle a DAP request
    pub fn handle_request(&self, request: Value) -> Value {
        // Parse request
        let request: DapRequest = match serde_json::from_value(request) {
            Ok(req) => req,
            Err(e) => {
                return json!({
                    "seq": self.next_seq(),
                    "type": "response",
                    "request_seq": 0,
                    "success": false,
                    "command": "unknown",
                    "message": format!("Failed to parse request: {}", e)
                });
            }
        };

        // Dispatch based on command
        match request.command.as_str() {
            "initialize" => self.handle_initialize(request),
            "launch" => self.handle_launch(request),
            "configurationDone" => self.handle_configuration_done(request),
            "disconnect" => self.handle_disconnect(request),
            "terminate" => self.handle_terminate(request),
            "setBreakpoints" => self.handle_set_breakpoints(request),
            "threads" => self.handle_threads(request),
            "stackTrace" => self.handle_stack_trace(request),
            "scopes" => self.handle_scopes(request),
            "variables" => self.handle_variables(request),
            "continue" => self.handle_continue(request),
            "next" => self.handle_next(request),
            "stepIn" => self.handle_step_in(request),
            "stepOut" => self.handle_step_out(request),
            "pause" => self.handle_pause(request),
            _ => self.handle_unknown(request),
        }
    }

    /// Handle initialize request
    fn handle_initialize(&self, request: DapRequest) -> Value {
        let mut state = self.state.lock().expect("Mutex should not be poisoned");
        *state = ServerState::Initialized;
        drop(state);

        let seq = self.next_seq();
        let response = DapResponse::success(
            request.seq,
            seq,
            request.command,
            Some(
                serde_json::to_value(&self.capabilities)
                    .expect("DAP capabilities should be serializable"),
            ),
        );

        serde_json::to_value(&response).expect("DapResponse should be serializable")
    }

    /// Handle launch request
    fn handle_launch(&self, request: DapRequest) -> Value {
        // Parse launch arguments
        let args: LaunchRequestArguments = match serde_json::from_value(request.arguments.clone()) {
            Ok(args) => args,
            Err(e) => {
                let seq = self.next_seq();
                let response = DapResponse::error(
                    request.seq,
                    seq,
                    request.command,
                    format!("Invalid launch arguments: {}", e),
                );
                return serde_json::to_value(&response)
                    .expect("DapResponse should be serializable");
            }
        };

        // Store program
        let mut program = self.program.lock().expect("Mutex should not be poisoned");
        *program = Some(args.program.clone());
        drop(program);

        // TRACE-004: Detect language and cache AST
        let program_path = Path::new(&args.program);

        // Detect language
        if let Some(language) = self.detect_language_from_path(program_path) {
            let mut lang = self
                .current_language
                .lock()
                .expect("Mutex should not be poisoned");
            *lang = Some(language);
        }

        // Parse and cache AST
        let _ = self.parse_and_cache_ast(program_path);

        // CAPTURE-002: Start recording if configured
        // Note: LaunchRequestArguments doesn't expose command-line args in DAP spec
        // Recording metadata will use empty args vector for now
        if let Err(e) = self.start_recording(&args.program, vec![]) {
            eprintln!("Warning: Failed to start recording: {}", e);
            // Continue debug session even if recording fails
        }

        // Update state
        let mut state = self.state.lock().expect("Mutex should not be poisoned");
        *state = ServerState::Running;
        drop(state);

        let seq = self.next_seq();
        let response = DapResponse::success(request.seq, seq, request.command, None);

        serde_json::to_value(&response).expect("DapResponse should be serializable")
    }

    /// Handle configurationDone request
    fn handle_configuration_done(&self, request: DapRequest) -> Value {
        let seq = self.next_seq();
        let response = DapResponse::success(request.seq, seq, request.command, None);
        serde_json::to_value(&response).expect("DapResponse should be serializable")
    }

    /// Handle disconnect request
    fn handle_disconnect(&self, request: DapRequest) -> Value {
        let mut state = self.state.lock().expect("Mutex should not be poisoned");
        *state = ServerState::Stopped;
        drop(state);

        // CAPTURE-002: Finalize recording on disconnect
        if let Ok(Some(path)) = self.finalize_recording() {
            println!("Recording saved: {}", path.display());
        }

        let seq = self.next_seq();
        let response = DapResponse::success(request.seq, seq, request.command, None);
        serde_json::to_value(&response).expect("DapResponse should be serializable")
    }

    /// Handle terminate request
    fn handle_terminate(&self, request: DapRequest) -> Value {
        let mut state = self.state.lock().expect("Mutex should not be poisoned");
        *state = ServerState::Stopped;
        drop(state);

        // CAPTURE-002: Finalize recording on terminate
        if let Ok(Some(path)) = self.finalize_recording() {
            println!("Recording saved: {}", path.display());
        }

        let seq = self.next_seq();
        let response = DapResponse::success(request.seq, seq, request.command, None);
        serde_json::to_value(&response).expect("DapResponse should be serializable")
    }

    /// Handle setBreakpoints request
    fn handle_set_breakpoints(&self, request: DapRequest) -> Value {
        // Parse setBreakpoints arguments
        let args: SetBreakpointsArguments = match serde_json::from_value(request.arguments.clone())
        {
            Ok(args) => args,
            Err(e) => {
                let seq = self.next_seq();
                let response = DapResponse::error(
                    request.seq,
                    seq,
                    request.command,
                    format!("Invalid setBreakpoints arguments: {}", e),
                );
                return serde_json::to_value(&response).expect("JSON serialization cannot fail");
            }
        };

        // Store breakpoints
        let source_path = args.source.path.unwrap_or_else(|| "unknown".to_string());
        let mut breakpoints_map = self
            .breakpoints
            .lock()
            .expect("Mutex should not be poisoned");

        if let Some(bps) = args.breakpoints {
            let lines: HashSet<i64> = bps.iter().map(|bp| bp.line).collect();
            breakpoints_map.insert(source_path.clone(), lines);
        } else {
            breakpoints_map.remove(&source_path);
        }
        drop(breakpoints_map);

        // TRACE-004: Parse and cache AST for breakpoint validation
        let bp_path = Path::new(&source_path);
        let _ = self.parse_and_cache_ast(bp_path);

        let seq = self.next_seq();
        let response = DapResponse::success(
            request.seq,
            seq,
            request.command,
            Some(json!({"breakpoints": []})),
        );

        serde_json::to_value(&response).expect("DapResponse should be serializable")
    }

    /// Handle threads request
    fn handle_threads(&self, request: DapRequest) -> Value {
        let seq = self.next_seq();
        let threads = vec![Thread {
            id: 1,
            name: "main".to_string(),
        }];
        let response = DapResponse::success(
            request.seq,
            seq,
            request.command,
            Some(json!({"threads": threads})),
        );
        serde_json::to_value(&response).expect("DapResponse should be serializable")
    }

    /// Handle stackTrace request
    fn handle_stack_trace(&self, request: DapRequest) -> Value {
        let seq = self.next_seq();
        let response = DapResponse::success(
            request.seq,
            seq,
            request.command,
            Some(json!({"stackFrames": [], "totalFrames": 0})),
        );
        serde_json::to_value(&response).expect("DapResponse should be serializable")
    }

    /// Handle scopes request
    /// TRACE-004: Returns Locals scope when stopped at a line
    fn handle_scopes(&self, request: DapRequest) -> Value {
        let seq = self.next_seq();

        // Check if we're stopped at a line
        let stopped_file = self
            .current_stopped_file
            .lock()
            .expect("Mutex should not be poisoned");
        let stopped_line = self
            .current_stopped_line
            .lock()
            .expect("Mutex should not be poisoned");

        let scopes = if stopped_file.is_some() && stopped_line.is_some() {
            // Return Locals scope with variablesReference = 1
            vec![json!({
                "name": "Locals",
                "variablesReference": 1,
                "expensive": false
            })]
        } else {
            vec![]
        };

        let response = DapResponse::success(
            request.seq,
            seq,
            request.command,
            Some(json!({"scopes": scopes})),
        );
        serde_json::to_value(&response).expect("DapResponse should be serializable")
    }

    /// Handle variables request
    /// TRACE-004: Returns variables from VariableInspector
    fn handle_variables(&self, request: DapRequest) -> Value {
        let seq = self.next_seq();

        // Get stopped location
        let stopped_file = self
            .current_stopped_file
            .lock()
            .expect("Mutex should not be poisoned")
            .clone();
        let stopped_line = *self
            .current_stopped_line
            .lock()
            .expect("Mutex should not be poisoned");

        let variables = if let (Some(file), Some(line)) = (stopped_file, stopped_line) {
            // Use VariableInspector to get variables
            match self.get_variables_at_line(&file, line) {
                Ok(vars) => {
                    // Convert Variable to DAP variable format
                    vars.iter()
                        .map(|v| {
                            json!({
                                "name": v.name,
                                "value": v.value,
                                "type": v.type_info,
                                "variablesReference": 0
                            })
                        })
                        .collect()
                }
                Err(_) => vec![],
            }
        } else {
            vec![]
        };

        let response = DapResponse::success(
            request.seq,
            seq,
            request.command,
            Some(json!({"variables": variables})),
        );
        serde_json::to_value(&response).expect("DapResponse should be serializable")
    }

    /// Handle continue request
    fn handle_continue(&self, request: DapRequest) -> Value {
        let seq = self.next_seq();
        let response = DapResponse::success(
            request.seq,
            seq,
            request.command,
            Some(json!({"allThreadsContinued": true})),
        );
        serde_json::to_value(&response).expect("DapResponse should be serializable")
    }

    /// Handle next request (step over)
    fn handle_next(&self, request: DapRequest) -> Value {
        // CAPTURE-002: Capture snapshot after step
        self.capture_snapshot_if_recording();

        let seq = self.next_seq();
        let response = DapResponse::success(request.seq, seq, request.command, None);
        serde_json::to_value(&response).expect("DapResponse should be serializable")
    }

    /// Handle stepIn request
    fn handle_step_in(&self, request: DapRequest) -> Value {
        // CAPTURE-002: Capture snapshot after step
        self.capture_snapshot_if_recording();

        let seq = self.next_seq();
        let response = DapResponse::success(request.seq, seq, request.command, None);
        serde_json::to_value(&response).expect("DapResponse should be serializable")
    }

    /// Handle stepOut request
    fn handle_step_out(&self, request: DapRequest) -> Value {
        // CAPTURE-002: Capture snapshot after step
        self.capture_snapshot_if_recording();

        let seq = self.next_seq();
        let response = DapResponse::success(request.seq, seq, request.command, None);
        serde_json::to_value(&response).expect("DapResponse should be serializable")
    }

    /// Handle pause request
    fn handle_pause(&self, request: DapRequest) -> Value {
        let seq = self.next_seq();
        let response = DapResponse::success(request.seq, seq, request.command, None);
        serde_json::to_value(&response).expect("DapResponse should be serializable")
    }

    /// Handle unknown command
    fn handle_unknown(&self, request: DapRequest) -> Value {
        let seq = self.next_seq();
        let response = DapResponse::error(
            request.seq,
            seq,
            request.command,
            "Command not supported".to_string(),
        );
        serde_json::to_value(&response).expect("DapResponse should be serializable")
    }

    // TRACE-004: DAP-PMAT Integration Methods

    /// Get the current detected language
    pub fn current_language(&self) -> Option<Language> {
        let lang = self
            .current_language
            .lock()
            .expect("Mutex should not be poisoned");
        *lang
    }

    /// Check if AST is cached for a given file path
    pub fn has_ast_for(&self, path: &str) -> bool {
        let cache = self.ast_cache.lock().expect("Mutex should not be poisoned");
        cache.contains_key(Path::new(path))
    }

    /// Get variables at a specific line in a file using VariableInspector
    pub fn get_variables_at_line(&self, path: &str, line: usize) -> Result<Vec<Variable>, String> {
        // Read file contents
        let source = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file {}: {}", path, e))?;

        // Detect language from path
        let language = self
            .detect_language_from_path(Path::new(path))
            .ok_or_else(|| format!("Could not detect language for {}", path))?;

        // Use VariableInspector to extract variables
        match language {
            Language::Rust => self.variable_inspector.inspect_rust(&source, line),
            Language::TypeScript | Language::JavaScript => {
                self.variable_inspector.inspect_typescript(&source, line)
            }
            Language::Python => self.variable_inspector.inspect_python(&source, line),
            _ => Err(format!(
                "Language {:?} not supported for variable inspection",
                language
            )),
        }
    }

    /// Simulate stopping at a specific line (for testing)
    pub fn simulate_stop_at_line(&mut self, path: &str, line: usize) {
        let mut stopped_file = self
            .current_stopped_file
            .lock()
            .expect("Mutex should not be poisoned");
        *stopped_file = Some(path.to_string());
        drop(stopped_file);

        let mut stopped_line = self
            .current_stopped_line
            .lock()
            .expect("Mutex should not be poisoned");
        *stopped_line = Some(line);
    }

    /// Get current stopped file (TRACE-005)
    pub fn current_stopped_file(&self) -> Option<String> {
        self.current_stopped_file
            .lock()
            .expect("Mutex should not be poisoned")
            .clone()
    }

    /// Get current stopped line (TRACE-005)
    pub fn current_stopped_line(&self) -> Option<usize> {
        *self
            .current_stopped_line
            .lock()
            .expect("Mutex should not be poisoned")
    }

    /// Detect language from file path
    fn detect_language_from_path(&self, path: &Path) -> Option<Language> {
        let extension = path.extension()?.to_str()?;

        match extension {
            "rs" => Some(Language::Rust),
            "py" => Some(Language::Python),
            "ts" | "tsx" => Some(Language::TypeScript),
            "js" | "jsx" => Some(Language::JavaScript),
            _ => None,
        }
    }

    /// Parse and cache AST for a file
    fn parse_and_cache_ast(&self, path: &Path) -> Result<(), String> {
        // Read source
        let source =
            std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

        // Detect language
        let language = self
            .detect_language_from_path(path)
            .ok_or_else(|| format!("Could not detect language for {:?}", path))?;

        // Parse using tree-sitter
        let tree = match language {
            Language::Rust => {
                let mut parser = tree_sitter::Parser::new();
                parser
                    .set_language(&tree_sitter_rust::LANGUAGE.into())
                    .map_err(|e| format!("Failed to set Rust language: {}", e))?;
                parser
                    .parse(&source, None)
                    .ok_or_else(|| "Failed to parse Rust source".to_string())?
            }
            Language::Python => {
                #[cfg(feature = "python-ast")]
                {
                    let mut parser = tree_sitter::Parser::new();
                    parser
                        .set_language(&tree_sitter_python::LANGUAGE.into())
                        .map_err(|e| format!("Failed to set Python language: {}", e))?;
                    parser
                        .parse(&source, None)
                        .ok_or_else(|| "Failed to parse Python source".to_string())?
                }
                #[cfg(not(feature = "python-ast"))]
                {
                    return Err("python-ast feature is disabled".to_string());
                }
            }
            Language::TypeScript | Language::JavaScript => {
                let mut parser = tree_sitter::Parser::new();
                parser
                    .set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())
                    .map_err(|e| format!("Failed to set TypeScript language: {}", e))?;
                parser
                    .parse(&source, None)
                    .ok_or_else(|| "Failed to parse TypeScript source".to_string())?
            }
            _ => return Err(format!("Language {:?} not supported for parsing", language)),
        };

        // Cache the tree
        let mut cache = self.ast_cache.lock().expect("Mutex should not be poisoned");
        cache.insert(path.to_path_buf(), tree);

        Ok(())
    }

    // Sprint 76 - CAPTURE-002: Recording Capture Methods

    /// Generate a unique recording file path with timestamp
    ///
    /// Format: session-{timestamp}.pmat
    fn generate_recording_path(&self) -> Option<PathBuf> {
        use std::time::{SystemTime, UNIX_EPOCH};

        let recording_dir = self.recording_dir.as_ref()?;

        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();

        Some(recording_dir.join(format!("session-{}.pmat", timestamp)))
    }

    /// Initialize recording on session start
    ///
    /// Creates recording directory if needed and sets up ExecutionRecorder
    fn start_recording(&self, program: &str, args: Vec<String>) -> anyhow::Result<()> {
        // Only start recording if recording_dir is configured
        let recording_dir = match &self.recording_dir {
            Some(dir) => dir,
            None => return Ok(()), // No recording configured
        };

        // Create recording directory if it doesn't exist
        if !recording_dir.exists() {
            std::fs::create_dir_all(recording_dir)
                .map_err(|e| anyhow::anyhow!("Failed to create recording directory: {}", e))?;
        }

        // Generate recording file path
        let recording_path = self
            .generate_recording_path()
            .ok_or_else(|| anyhow::anyhow!("Failed to generate recording path"))?;

        // Create recording file
        let file = File::create(&recording_path)
            .map_err(|e| anyhow::anyhow!("Failed to create recording file: {}", e))?;

        // Create ExecutionRecorder with writer
        let dap_server_arc = Arc::new(Mutex::new(DapServer::new()));
        let mut recorder =
            ExecutionRecorder::with_writer(file, program.to_string(), args, dap_server_arc)?;

        // Start recording (enables snapshot capture)
        recorder.start_recording();

        // Store recorder and path
        *self
            .execution_recorder
            .lock()
            .expect("Mutex should not be poisoned") = Some(recorder);
        *self
            .recording_path
            .lock()
            .expect("Mutex should not be poisoned") = Some(recording_path);

        Ok(())
    }

    /// Finalize and save recording
    ///
    /// Called on disconnect or terminate to complete the .pmat file
    fn finalize_recording(&self) -> anyhow::Result<Option<PathBuf>> {
        let mut recorder_guard = self
            .execution_recorder
            .lock()
            .expect("Mutex should not be poisoned");
        let recorder = recorder_guard.take();

        if let Some(recorder) = recorder {
            recorder.finalize()?;
            let path = self
                .recording_path
                .lock()
                .expect("Mutex should not be poisoned")
                .clone();
            return Ok(path);
        }

        Ok(None)
    }

    /// CAPTURE-002: Attempt to capture a snapshot if recording is active
    ///
    /// This is called on debug events (breakpoint hits, step commands) to capture
    /// execution state. Silently fails if recording is not active or capture fails.
    fn capture_snapshot_if_recording(&self) {
        let mut recorder_guard = match self.execution_recorder.lock() {
            Ok(guard) => guard,
            Err(_) => return, // Lock poisoned, skip capture
        };

        if let Some(ref mut recorder) = *recorder_guard {
            // Only attempt capture if recorder is in recording state
            if recorder.is_recording() {
                if let Err(e) = recorder.capture_snapshot() {
                    eprintln!("Warning: Failed to capture snapshot: {}", e);
                    // Continue execution even if snapshot capture fails
                }
            }
        }
    }

    // Sprint 74 - DEBUG-002: DAP Server CLI Handler

    /// Run the DAP server on the specified port
    ///
    /// This starts an async TCP server that listens for DAP protocol connections.
    /// The server runs until the async task is aborted (e.g., via Ctrl+C).
    ///
    /// # Arguments
    /// * `port` - Port number to bind to (e.g., 5678)
    /// * `host` - Host address to bind to (e.g., "127.0.0.1")
    ///
    /// # Returns
    /// * `Ok(())` if server starts and shuts down cleanly
    /// * `Err` if port binding fails (e.g., "address already in use")
    pub async fn run(&self, port: u16, host: String) -> anyhow::Result<()> {
        use tokio::net::TcpListener;

        // Bind to TCP port
        let addr = format!("{}:{}", host, port);
        let listener = TcpListener::bind(&addr)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to bind to {}: {}", addr, e))?;

        // Server is now listening - accept connections in a loop
        loop {
            // Accept incoming connection
            let (_stream, _addr) = listener.accept().await?;

            // Minimal implementation: just accept and drop connections
            // Future enhancement: read DAP messages from stream and call handle_request()
        }
    }
}

impl Default for DapServer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_creation() {
        let server = DapServer::new();
        assert!(!server.is_initialized());
        assert!(!server.is_running());
        assert!(!server.has_program_loaded());
    }

    #[test]
    fn test_server_default() {
        let server = DapServer::default();
        assert!(!server.is_initialized());
    }

    #[test]
    fn test_next_seq_increments() {
        let server = DapServer::new();
        let seq1 = server.next_seq();
        let seq2 = server.next_seq();
        assert_eq!(seq2, seq1 + 1);
    }
}

#[cfg(test)]
mod coverage_tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;
    use tempfile::{tempdir, NamedTempFile};

    // Server Creation Tests

    #[test]
    fn test_server_new_creates_uninitialized_server() {
        let server = DapServer::new();
        assert!(!server.is_initialized());
        assert!(!server.is_running());
        assert!(!server.has_program_loaded());
        assert!(server.current_program().is_none());
    }

    #[test]
    fn test_server_with_recording_none() {
        let server = DapServer::with_recording(None);
        assert!(!server.is_initialized());
    }

    #[test]
    fn test_server_with_recording_some() {
        let temp_dir = tempdir().unwrap();
        let server = DapServer::with_recording(Some(temp_dir.path().to_path_buf()));
        assert!(!server.is_initialized());
    }

    #[test]
    fn test_server_default_creates_new_server() {
        let server1 = DapServer::new();
        let server2 = DapServer::default();
        assert_eq!(server1.is_initialized(), server2.is_initialized());
    }

    // Server State Tests

    #[test]
    fn test_is_initialized_false_initially() {
        let server = DapServer::new();
        assert!(!server.is_initialized());
    }

    #[test]
    fn test_is_running_false_initially() {
        let server = DapServer::new();
        assert!(!server.is_running());
    }

    #[test]
    fn test_has_program_loaded_false_initially() {
        let server = DapServer::new();
        assert!(!server.has_program_loaded());
    }

    #[test]
    fn test_current_program_none_initially() {
        let server = DapServer::new();
        assert!(server.current_program().is_none());
    }

    #[test]
    fn test_current_language_none_initially() {
        let server = DapServer::new();
        assert!(server.current_language().is_none());
    }

    #[test]
    fn test_has_ast_for_false_initially() {
        let server = DapServer::new();
        assert!(!server.has_ast_for("/some/path.rs"));
    }

    #[test]
    fn test_current_stopped_file_none_initially() {
        let server = DapServer::new();
        assert!(server.current_stopped_file().is_none());
    }

    #[test]
    fn test_current_stopped_line_none_initially() {
        let server = DapServer::new();
        assert!(server.current_stopped_line().is_none());
    }

    // Handle Request - Invalid Request Tests

    #[test]
    fn test_handle_request_invalid_json() {
        let server = DapServer::new();
        let invalid_request = json!("not a valid request object");
        let response = server.handle_request(invalid_request);

        assert_eq!(response["success"], false);
        assert!(response["message"]
            .as_str()
            .unwrap()
            .contains("Failed to parse request"));
    }

    #[test]
    fn test_handle_request_missing_fields() {
        let server = DapServer::new();
        let invalid_request = json!({
            "seq": 1
            // missing type, command, arguments
        });
        let response = server.handle_request(invalid_request);
        assert_eq!(response["success"], false);
    }

    // Initialize Command Tests

    #[test]
    fn test_handle_initialize() {
        let server = DapServer::new();
        let request = json!({
            "seq": 1,
            "type": "request",
            "command": "initialize",
            "arguments": {
                "adapterId": "pmat-dap"
            }
        });

        let response = server.handle_request(request);

        assert_eq!(response["type"], "response");
        assert_eq!(response["success"], true);
        assert_eq!(response["command"], "initialize");
        assert!(server.is_initialized());
    }

    #[test]
    fn test_initialize_returns_capabilities() {
        let server = DapServer::new();
        let request = json!({
            "seq": 1,
            "type": "request",
            "command": "initialize",
            "arguments": {}
        });

        let response = server.handle_request(request);

        // Verify capabilities in response body
        let body = response["body"].as_object().unwrap();
        assert!(body.contains_key("supportsConfigurationDoneRequest"));
        assert!(body.contains_key("supportsConditionalBreakpoints"));
        assert!(body.contains_key("supportsTerminateRequest"));
    }

    // Launch Command Tests

    #[test]
    fn test_handle_launch_success() {
        let server = DapServer::new();

        // Create a temp Rust file
        let mut temp_file = NamedTempFile::with_suffix(".rs").unwrap();
        writeln!(temp_file, "fn main() {{}}").unwrap();
        let file_path = temp_file.path().to_string_lossy().to_string();

        // First initialize
        let init_request = json!({
            "seq": 1,
            "type": "request",
            "command": "initialize",
            "arguments": {}
        });
        server.handle_request(init_request);

        // Then launch
        let launch_request = json!({
            "seq": 2,
            "type": "request",
            "command": "launch",
            "arguments": {
                "program": file_path
            }
        });

        let response = server.handle_request(launch_request);

        assert_eq!(response["success"], true);
        assert_eq!(response["command"], "launch");
        assert!(server.is_running());
        assert!(server.has_program_loaded());
    }

    #[test]
    fn test_handle_launch_invalid_arguments() {
        let server = DapServer::new();

        let launch_request = json!({
            "seq": 1,
            "type": "request",
            "command": "launch",
            "arguments": {
                // missing required "program" field
            }
        });

        let response = server.handle_request(launch_request);

        assert_eq!(response["success"], false);
        assert!(response["message"]
            .as_str()
            .unwrap()
            .contains("Invalid launch arguments"));
    }

    #[test]
    fn test_handle_launch_with_recording() {
        let temp_dir = tempdir().unwrap();
        let server = DapServer::with_recording(Some(temp_dir.path().to_path_buf()));

        // Create a temp Rust file
        let mut temp_file = NamedTempFile::with_suffix(".rs").unwrap();
        writeln!(temp_file, "fn main() {{}}").unwrap();
        let file_path = temp_file.path().to_string_lossy().to_string();

        // Initialize and launch
        let init_request = json!({
            "seq": 1,
            "type": "request",
            "command": "initialize",
            "arguments": {}
        });
        server.handle_request(init_request);

        let launch_request = json!({
            "seq": 2,
            "type": "request",
            "command": "launch",
            "arguments": {
                "program": file_path
            }
        });

        let response = server.handle_request(launch_request);
        assert_eq!(response["success"], true);
    }

    // Configuration Done Command Tests

    #[test]
    fn test_handle_configuration_done() {
        let server = DapServer::new();

        let request = json!({
            "seq": 1,
            "type": "request",
            "command": "configurationDone",
            "arguments": {}
        });

        let response = server.handle_request(request);

        assert_eq!(response["success"], true);
        assert_eq!(response["command"], "configurationDone");
    }

    // Disconnect Command Tests

    #[test]
    fn test_handle_disconnect() {
        let server = DapServer::new();

        // Initialize first
        let init_request = json!({
            "seq": 1,
            "type": "request",
            "command": "initialize",
            "arguments": {}
        });
        server.handle_request(init_request);

        // Then disconnect
        let disconnect_request = json!({
            "seq": 2,
            "type": "request",
            "command": "disconnect",
            "arguments": {}
        });

        let response = server.handle_request(disconnect_request);

        assert_eq!(response["success"], true);
        assert_eq!(response["command"], "disconnect");
        assert!(!server.is_running());
    }

    #[test]
    fn test_handle_disconnect_with_recording() {
        let temp_dir = tempdir().unwrap();
        let server = DapServer::with_recording(Some(temp_dir.path().to_path_buf()));

        // Create a temp Rust file
        let mut temp_file = NamedTempFile::with_suffix(".rs").unwrap();
        writeln!(temp_file, "fn main() {{}}").unwrap();
        let file_path = temp_file.path().to_string_lossy().to_string();

        // Initialize
        let init_request = json!({
            "seq": 1,
            "type": "request",
            "command": "initialize",
            "arguments": {}
        });
        server.handle_request(init_request);

        // Launch
        let launch_request = json!({
            "seq": 2,
            "type": "request",
            "command": "launch",
            "arguments": {
                "program": file_path
            }
        });
        server.handle_request(launch_request);

        // Disconnect (should finalize recording)
        let disconnect_request = json!({
            "seq": 3,
            "type": "request",
            "command": "disconnect",
            "arguments": {}
        });

        let response = server.handle_request(disconnect_request);
        assert_eq!(response["success"], true);
    }

    // Terminate Command Tests

    #[test]
    fn test_handle_terminate() {
        let server = DapServer::new();

        // Initialize first
        let init_request = json!({
            "seq": 1,
            "type": "request",
            "command": "initialize",
            "arguments": {}
        });
        server.handle_request(init_request);

        // Then terminate
        let terminate_request = json!({
            "seq": 2,
            "type": "request",
            "command": "terminate",
            "arguments": {}
        });

        let response = server.handle_request(terminate_request);

        assert_eq!(response["success"], true);
        assert_eq!(response["command"], "terminate");
        assert!(!server.is_running());
    }

    #[test]
    fn test_handle_terminate_with_recording() {
        let temp_dir = tempdir().unwrap();
        let server = DapServer::with_recording(Some(temp_dir.path().to_path_buf()));

        // Create a temp Rust file
        let mut temp_file = NamedTempFile::with_suffix(".rs").unwrap();
        writeln!(temp_file, "fn main() {{}}").unwrap();
        let file_path = temp_file.path().to_string_lossy().to_string();

        // Initialize
        let init_request = json!({
            "seq": 1,
            "type": "request",
            "command": "initialize",
            "arguments": {}
        });
        server.handle_request(init_request);

        // Launch
        let launch_request = json!({
            "seq": 2,
            "type": "request",
            "command": "launch",
            "arguments": {
                "program": file_path
            }
        });
        server.handle_request(launch_request);

        // Terminate (should finalize recording)
        let terminate_request = json!({
            "seq": 3,
            "type": "request",
            "command": "terminate",
            "arguments": {}
        });

        let response = server.handle_request(terminate_request);
        assert_eq!(response["success"], true);
    }

    // SetBreakpoints Command Tests

    #[test]
    fn test_handle_set_breakpoints_success() {
        let server = DapServer::new();

        let request = json!({
            "seq": 1,
            "type": "request",
            "command": "setBreakpoints",
            "arguments": {
                "source": {
                    "path": "/some/file.rs"
                },
                "breakpoints": [
                    { "line": 10 },
                    { "line": 20 }
                ]
            }
        });

        let response = server.handle_request(request);

        assert_eq!(response["success"], true);
        assert_eq!(response["command"], "setBreakpoints");
    }

    #[test]
    fn test_handle_set_breakpoints_no_breakpoints() {
        let server = DapServer::new();

        let request = json!({
            "seq": 1,
            "type": "request",
            "command": "setBreakpoints",
            "arguments": {
                "source": {
                    "path": "/some/file.rs"
                }
                // No breakpoints field - should clear breakpoints
            }
        });

        let response = server.handle_request(request);

        assert_eq!(response["success"], true);
    }

    #[test]
    fn test_handle_set_breakpoints_no_path() {
        let server = DapServer::new();

        let request = json!({
            "seq": 1,
            "type": "request",
            "command": "setBreakpoints",
            "arguments": {
                "source": {
                    // No path - should use "unknown"
                },
                "breakpoints": [
                    { "line": 10 }
                ]
            }
        });

        let response = server.handle_request(request);

        assert_eq!(response["success"], true);
    }

    #[test]
    fn test_handle_set_breakpoints_invalid_arguments() {
        let server = DapServer::new();

        let request = json!({
            "seq": 1,
            "type": "request",
            "command": "setBreakpoints",
            "arguments": {
                // Missing required "source" field
            }
        });

        let response = server.handle_request(request);

        assert_eq!(response["success"], false);
        assert!(response["message"]
            .as_str()
            .unwrap()
            .contains("Invalid setBreakpoints arguments"));
    }

    // Threads Command Tests

    #[test]
    fn test_handle_threads() {
        let server = DapServer::new();

        let request = json!({
            "seq": 1,
            "type": "request",
            "command": "threads",
            "arguments": {}
        });

        let response = server.handle_request(request);

        assert_eq!(response["success"], true);
        assert_eq!(response["command"], "threads");

        let threads = response["body"]["threads"].as_array().unwrap();
        assert_eq!(threads.len(), 1);
        assert_eq!(threads[0]["id"], 1);
        assert_eq!(threads[0]["name"], "main");
    }

    // StackTrace Command Tests

    #[test]
    fn test_handle_stack_trace() {
        let server = DapServer::new();

        let request = json!({
            "seq": 1,
            "type": "request",
            "command": "stackTrace",
            "arguments": {
                "threadId": 1
            }
        });

        let response = server.handle_request(request);

        assert_eq!(response["success"], true);
        assert_eq!(response["command"], "stackTrace");
        assert_eq!(response["body"]["totalFrames"], 0);
    }

    // Scopes Command Tests

    #[test]
    fn test_handle_scopes_no_stopped_location() {
        let server = DapServer::new();

        let request = json!({
            "seq": 1,
            "type": "request",
            "command": "scopes",
            "arguments": {
                "frameId": 1
            }
        });

        let response = server.handle_request(request);

        assert_eq!(response["success"], true);
        assert_eq!(response["command"], "scopes");

        // No stopped location, so scopes should be empty
        let scopes = response["body"]["scopes"].as_array().unwrap();
        assert!(scopes.is_empty());
    }

    #[test]
    fn test_handle_scopes_with_stopped_location() {
        let mut server = DapServer::new();

        // Create a temp file for simulating stop
        let mut temp_file = NamedTempFile::with_suffix(".rs").unwrap();
        writeln!(temp_file, "fn main() {{\n    let x = 42;\n}}").unwrap();

        server.simulate_stop_at_line(temp_file.path().to_str().unwrap(), 2);

        let request = json!({
            "seq": 1,
            "type": "request",
            "command": "scopes",
            "arguments": {
                "frameId": 1
            }
        });

        let response = server.handle_request(request);

        assert_eq!(response["success"], true);

        // With stopped location, should have Locals scope
        let scopes = response["body"]["scopes"].as_array().unwrap();
        assert_eq!(scopes.len(), 1);
        assert_eq!(scopes[0]["name"], "Locals");
        assert_eq!(scopes[0]["variablesReference"], 1);
    }

    // Variables Command Tests

    #[test]
    fn test_handle_variables_no_stopped_location() {
        let server = DapServer::new();

        let request = json!({
            "seq": 1,
            "type": "request",
            "command": "variables",
            "arguments": {
                "variablesReference": 1
            }
        });

        let response = server.handle_request(request);

        assert_eq!(response["success"], true);
        assert_eq!(response["command"], "variables");

        // No stopped location, so variables should be empty
        let variables = response["body"]["variables"].as_array().unwrap();
        assert!(variables.is_empty());
    }

    #[test]
    fn test_handle_variables_with_stopped_location() {
        let mut server = DapServer::new();

        // Create a temp file with a variable declaration
        let mut temp_file = NamedTempFile::with_suffix(".rs").unwrap();
        writeln!(
            temp_file,
            r#"fn main() {{
    let x = 42;
    println!("{{}}", x);
}}"#
        )
        .unwrap();

        server.simulate_stop_at_line(temp_file.path().to_str().unwrap(), 3);

        let request = json!({
            "seq": 1,
            "type": "request",
            "command": "variables",
            "arguments": {
                "variablesReference": 1
            }
        });

        let response = server.handle_request(request);

        assert_eq!(response["success"], true);
        // Variables should include "x"
    }

    // Continue Command Tests

    #[test]
    fn test_handle_continue() {
        let server = DapServer::new();

        let request = json!({
            "seq": 1,
            "type": "request",
            "command": "continue",
            "arguments": {
                "threadId": 1
            }
        });

        let response = server.handle_request(request);

        assert_eq!(response["success"], true);
        assert_eq!(response["command"], "continue");
        assert_eq!(response["body"]["allThreadsContinued"], true);
    }

    // Step Commands Tests

    #[test]
    fn test_handle_next() {
        let server = DapServer::new();

        let request = json!({
            "seq": 1,
            "type": "request",
            "command": "next",
            "arguments": {
                "threadId": 1
            }
        });

        let response = server.handle_request(request);

        assert_eq!(response["success"], true);
        assert_eq!(response["command"], "next");
    }

    #[test]
    fn test_handle_step_in() {
        let server = DapServer::new();

        let request = json!({
            "seq": 1,
            "type": "request",
            "command": "stepIn",
            "arguments": {
                "threadId": 1
            }
        });

        let response = server.handle_request(request);

        assert_eq!(response["success"], true);
        assert_eq!(response["command"], "stepIn");
    }

    #[test]
    fn test_handle_step_out() {
        let server = DapServer::new();

        let request = json!({
            "seq": 1,
            "type": "request",
            "command": "stepOut",
            "arguments": {
                "threadId": 1
            }
        });

        let response = server.handle_request(request);

        assert_eq!(response["success"], true);
        assert_eq!(response["command"], "stepOut");
    }

    // Pause Command Tests

    #[test]
    fn test_handle_pause() {
        let server = DapServer::new();

        let request = json!({
            "seq": 1,
            "type": "request",
            "command": "pause",
            "arguments": {
                "threadId": 1
            }
        });

        let response = server.handle_request(request);

        assert_eq!(response["success"], true);
        assert_eq!(response["command"], "pause");
    }

    // Unknown Command Tests

    #[test]
    fn test_handle_unknown_command() {
        let server = DapServer::new();

        let request = json!({
            "seq": 1,
            "type": "request",
            "command": "unknownCommand",
            "arguments": {}
        });

        let response = server.handle_request(request);

        assert_eq!(response["success"], false);
        assert_eq!(response["command"], "unknownCommand");
        assert_eq!(response["message"], "Command not supported");
    }

    // Language Detection Tests

    #[test]
    fn test_detect_language_rust() {
        let server = DapServer::new();

        // Launch a Rust file
        let mut temp_file = NamedTempFile::with_suffix(".rs").unwrap();
        writeln!(temp_file, "fn main() {{}}").unwrap();

        let init_request = json!({
            "seq": 1,
            "type": "request",
            "command": "initialize",
            "arguments": {}
        });
        server.handle_request(init_request);

        let launch_request = json!({
            "seq": 2,
            "type": "request",
            "command": "launch",
            "arguments": {
                "program": temp_file.path().to_string_lossy().to_string()
            }
        });
        server.handle_request(launch_request);

        assert_eq!(server.current_language(), Some(Language::Rust));
    }

    #[test]
    fn test_detect_language_python() {
        let server = DapServer::new();

        let mut temp_file = NamedTempFile::with_suffix(".py").unwrap();
        writeln!(temp_file, "x = 42").unwrap();

        let init_request = json!({
            "seq": 1,
            "type": "request",
            "command": "initialize",
            "arguments": {}
        });
        server.handle_request(init_request);

        let launch_request = json!({
            "seq": 2,
            "type": "request",
            "command": "launch",
            "arguments": {
                "program": temp_file.path().to_string_lossy().to_string()
            }
        });
        server.handle_request(launch_request);

        assert_eq!(server.current_language(), Some(Language::Python));
    }

    #[test]
    fn test_detect_language_typescript() {
        let server = DapServer::new();

        let mut temp_file = NamedTempFile::with_suffix(".ts").unwrap();
        writeln!(temp_file, "const x: number = 42;").unwrap();

        let init_request = json!({
            "seq": 1,
            "type": "request",
            "command": "initialize",
            "arguments": {}
        });
        server.handle_request(init_request);

        let launch_request = json!({
            "seq": 2,
            "type": "request",
            "command": "launch",
            "arguments": {
                "program": temp_file.path().to_string_lossy().to_string()
            }
        });
        server.handle_request(launch_request);

        assert_eq!(server.current_language(), Some(Language::TypeScript));
    }

    #[test]
    fn test_detect_language_tsx() {
        let server = DapServer::new();

        let mut temp_file = NamedTempFile::with_suffix(".tsx").unwrap();
        writeln!(temp_file, "const x: number = 42;").unwrap();

        let init_request = json!({
            "seq": 1,
            "type": "request",
            "command": "initialize",
            "arguments": {}
        });
        server.handle_request(init_request);

        let launch_request = json!({
            "seq": 2,
            "type": "request",
            "command": "launch",
            "arguments": {
                "program": temp_file.path().to_string_lossy().to_string()
            }
        });
        server.handle_request(launch_request);

        assert_eq!(server.current_language(), Some(Language::TypeScript));
    }

    #[test]
    fn test_detect_language_javascript() {
        let server = DapServer::new();

        let mut temp_file = NamedTempFile::with_suffix(".js").unwrap();
        writeln!(temp_file, "const x = 42;").unwrap();

        let init_request = json!({
            "seq": 1,
            "type": "request",
            "command": "initialize",
            "arguments": {}
        });
        server.handle_request(init_request);

        let launch_request = json!({
            "seq": 2,
            "type": "request",
            "command": "launch",
            "arguments": {
                "program": temp_file.path().to_string_lossy().to_string()
            }
        });
        server.handle_request(launch_request);

        assert_eq!(server.current_language(), Some(Language::JavaScript));
    }

    #[test]
    fn test_detect_language_jsx() {
        let server = DapServer::new();

        let mut temp_file = NamedTempFile::with_suffix(".jsx").unwrap();
        writeln!(temp_file, "const x = 42;").unwrap();

        let init_request = json!({
            "seq": 1,
            "type": "request",
            "command": "initialize",
            "arguments": {}
        });
        server.handle_request(init_request);

        let launch_request = json!({
            "seq": 2,
            "type": "request",
            "command": "launch",
            "arguments": {
                "program": temp_file.path().to_string_lossy().to_string()
            }
        });
        server.handle_request(launch_request);

        assert_eq!(server.current_language(), Some(Language::JavaScript));
    }

    #[test]
    fn test_detect_language_unknown() {
        let server = DapServer::new();

        let mut temp_file = NamedTempFile::with_suffix(".xyz").unwrap();
        writeln!(temp_file, "unknown content").unwrap();

        let init_request = json!({
            "seq": 1,
            "type": "request",
            "command": "initialize",
            "arguments": {}
        });
        server.handle_request(init_request);

        let launch_request = json!({
            "seq": 2,
            "type": "request",
            "command": "launch",
            "arguments": {
                "program": temp_file.path().to_string_lossy().to_string()
            }
        });
        server.handle_request(launch_request);

        // Unknown extension - language should be None
        assert!(server.current_language().is_none());
    }

    // AST Caching Tests

    #[test]
    fn test_ast_caching_for_rust_file() {
        let server = DapServer::new();

        let mut temp_file = NamedTempFile::with_suffix(".rs").unwrap();
        writeln!(temp_file, "fn main() {{\n    let x = 42;\n}}").unwrap();
        let file_path = temp_file.path().to_string_lossy().to_string();

        // Launch the program
        let init_request = json!({
            "seq": 1,
            "type": "request",
            "command": "initialize",
            "arguments": {}
        });
        server.handle_request(init_request);

        let launch_request = json!({
            "seq": 2,
            "type": "request",
            "command": "launch",
            "arguments": {
                "program": file_path.clone()
            }
        });
        server.handle_request(launch_request);

        // AST should be cached
        assert!(server.has_ast_for(&file_path));
    }

    #[test]
    fn test_ast_caching_for_typescript_file() {
        let server = DapServer::new();

        let mut temp_file = NamedTempFile::with_suffix(".ts").unwrap();
        writeln!(temp_file, "const x: number = 42;").unwrap();
        let file_path = temp_file.path().to_string_lossy().to_string();

        let init_request = json!({
            "seq": 1,
            "type": "request",
            "command": "initialize",
            "arguments": {}
        });
        server.handle_request(init_request);

        let launch_request = json!({
            "seq": 2,
            "type": "request",
            "command": "launch",
            "arguments": {
                "program": file_path.clone()
            }
        });
        server.handle_request(launch_request);

        assert!(server.has_ast_for(&file_path));
    }

    #[test]
    fn test_ast_caching_for_javascript_file() {
        let server = DapServer::new();

        let mut temp_file = NamedTempFile::with_suffix(".js").unwrap();
        writeln!(temp_file, "const x = 42;").unwrap();
        let file_path = temp_file.path().to_string_lossy().to_string();

        let init_request = json!({
            "seq": 1,
            "type": "request",
            "command": "initialize",
            "arguments": {}
        });
        server.handle_request(init_request);

        let launch_request = json!({
            "seq": 2,
            "type": "request",
            "command": "launch",
            "arguments": {
                "program": file_path.clone()
            }
        });
        server.handle_request(launch_request);

        assert!(server.has_ast_for(&file_path));
    }

    #[test]
    fn test_breakpoints_cache_ast() {
        let server = DapServer::new();

        let mut temp_file = NamedTempFile::with_suffix(".rs").unwrap();
        writeln!(temp_file, "fn main() {{\n    let x = 42;\n}}").unwrap();
        let file_path = temp_file.path().to_string_lossy().to_string();

        let request = json!({
            "seq": 1,
            "type": "request",
            "command": "setBreakpoints",
            "arguments": {
                "source": {
                    "path": file_path.clone()
                },
                "breakpoints": [
                    { "line": 2 }
                ]
            }
        });

        server.handle_request(request);

        // AST should be cached after setting breakpoints
        assert!(server.has_ast_for(&file_path));
    }

    // Simulate Stop Tests

    #[test]
    fn test_simulate_stop_at_line() {
        let mut server = DapServer::new();

        server.simulate_stop_at_line("/path/to/file.rs", 42);

        assert_eq!(
            server.current_stopped_file(),
            Some("/path/to/file.rs".to_string())
        );
        assert_eq!(server.current_stopped_line(), Some(42));
    }

    #[test]
    fn test_simulate_stop_updates_location() {
        let mut server = DapServer::new();

        server.simulate_stop_at_line("/first/file.rs", 10);
        assert_eq!(
            server.current_stopped_file(),
            Some("/first/file.rs".to_string())
        );
        assert_eq!(server.current_stopped_line(), Some(10));

        server.simulate_stop_at_line("/second/file.rs", 20);
        assert_eq!(
            server.current_stopped_file(),
            Some("/second/file.rs".to_string())
        );
        assert_eq!(server.current_stopped_line(), Some(20));
    }

    // Get Variables At Line Tests

    #[test]
    fn test_get_variables_at_line_rust() {
        let server = DapServer::new();

        let mut temp_file = NamedTempFile::with_suffix(".rs").unwrap();
        writeln!(
            temp_file,
            r#"fn main() {{
    let x = 42;
    println!("{{}}", x);
}}"#
        )
        .unwrap();

        let result = server.get_variables_at_line(temp_file.path().to_str().unwrap(), 3);

        assert!(result.is_ok());
        let vars = result.unwrap();
        assert!(vars.iter().any(|v| v.name == "x"));
    }

    #[test]
    fn test_get_variables_at_line_typescript() {
        let server = DapServer::new();

        let mut temp_file = NamedTempFile::with_suffix(".ts").unwrap();
        writeln!(
            temp_file,
            r#"function main() {{
    const x = 42;
    console.log(x);
}}"#
        )
        .unwrap();

        let result = server.get_variables_at_line(temp_file.path().to_str().unwrap(), 3);

        assert!(result.is_ok());
        let vars = result.unwrap();
        assert!(vars.iter().any(|v| v.name == "x"));
    }

    #[test]
    fn test_get_variables_at_line_javascript() {
        let server = DapServer::new();

        let mut temp_file = NamedTempFile::with_suffix(".js").unwrap();
        writeln!(
            temp_file,
            r#"function main() {{
    const x = 42;
    console.log(x);
}}"#
        )
        .unwrap();

        let result = server.get_variables_at_line(temp_file.path().to_str().unwrap(), 3);

        assert!(result.is_ok());
    }

    #[test]
    fn test_get_variables_at_line_python() {
        let server = DapServer::new();

        let mut temp_file = NamedTempFile::with_suffix(".py").unwrap();
        writeln!(
            temp_file,
            r#"def main():
    x = 42
    print(x)"#
        )
        .unwrap();

        let result = server.get_variables_at_line(temp_file.path().to_str().unwrap(), 3);

        // Python support depends on python-ast feature
        #[cfg(not(feature = "python-ast"))]
        assert!(result.is_err());
        #[cfg(feature = "python-ast")]
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_variables_at_line_unsupported_language() {
        let server = DapServer::new();

        let mut temp_file = NamedTempFile::with_suffix(".xyz").unwrap();
        writeln!(temp_file, "some content").unwrap();

        let result = server.get_variables_at_line(temp_file.path().to_str().unwrap(), 1);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Could not detect language"));
    }

    #[test]
    fn test_get_variables_at_line_file_not_found() {
        let server = DapServer::new();

        let result = server.get_variables_at_line("/nonexistent/file.rs", 1);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to read file"));
    }

    // Recording Path Generation Tests

    #[test]
    fn test_generate_recording_path_without_dir() {
        let server = DapServer::new();

        // Without recording_dir, should return None
        let path = server.generate_recording_path();
        assert!(path.is_none());
    }

    #[test]
    fn test_generate_recording_path_with_dir() {
        let temp_dir = tempdir().unwrap();
        let server = DapServer::with_recording(Some(temp_dir.path().to_path_buf()));

        let path = server.generate_recording_path();
        assert!(path.is_some());

        let path = path.unwrap();
        assert!(path.to_string_lossy().contains("session-"));
        assert!(path.to_string_lossy().ends_with(".pmat"));
    }

    // Step Commands with Recording Tests

    #[test]
    fn test_step_commands_capture_snapshot() {
        let temp_dir = tempdir().unwrap();
        let server = DapServer::with_recording(Some(temp_dir.path().to_path_buf()));

        // Create a temp Rust file
        let mut temp_file = NamedTempFile::with_suffix(".rs").unwrap();
        writeln!(temp_file, "fn main() {{\n    let x = 42;\n}}").unwrap();
        let file_path = temp_file.path().to_string_lossy().to_string();

        // Initialize and launch
        let init_request = json!({
            "seq": 1,
            "type": "request",
            "command": "initialize",
            "arguments": {}
        });
        server.handle_request(init_request);

        let launch_request = json!({
            "seq": 2,
            "type": "request",
            "command": "launch",
            "arguments": {
                "program": file_path
            }
        });
        server.handle_request(launch_request);

        // Next step
        let next_request = json!({
            "seq": 3,
            "type": "request",
            "command": "next",
            "arguments": {"threadId": 1}
        });
        let response = server.handle_request(next_request);
        assert_eq!(response["success"], true);

        // StepIn
        let step_in_request = json!({
            "seq": 4,
            "type": "request",
            "command": "stepIn",
            "arguments": {"threadId": 1}
        });
        let response = server.handle_request(step_in_request);
        assert_eq!(response["success"], true);

        // StepOut
        let step_out_request = json!({
            "seq": 5,
            "type": "request",
            "command": "stepOut",
            "arguments": {"threadId": 1}
        });
        let response = server.handle_request(step_out_request);
        assert_eq!(response["success"], true);
    }

    // Sequence Number Tests

    #[test]
    fn test_sequence_number_increments() {
        let server = DapServer::new();

        let seq1 = server.next_seq();
        let seq2 = server.next_seq();
        let seq3 = server.next_seq();

        assert_eq!(seq1, 1);
        assert_eq!(seq2, 2);
        assert_eq!(seq3, 3);
    }

    #[test]
    fn test_response_sequence_numbers() {
        let server = DapServer::new();

        let request1 = json!({
            "seq": 1,
            "type": "request",
            "command": "threads",
            "arguments": {}
        });
        let response1 = server.handle_request(request1);

        let request2 = json!({
            "seq": 2,
            "type": "request",
            "command": "threads",
            "arguments": {}
        });
        let response2 = server.handle_request(request2);

        // Response seq should increment
        assert!(response2["seq"].as_i64().unwrap() > response1["seq"].as_i64().unwrap());
    }

    // Default Capabilities Tests

    #[test]
    fn test_default_capabilities() {
        let server = DapServer::new();

        let request = json!({
            "seq": 1,
            "type": "request",
            "command": "initialize",
            "arguments": {}
        });
        let response = server.handle_request(request);

        let body = response["body"].as_object().unwrap();

        // Verify specific capabilities
        assert_eq!(body["supportsConfigurationDoneRequest"], true);
        assert_eq!(body["supportsConditionalBreakpoints"], true);
        assert_eq!(body["supportsTerminateDebuggee"], true);
        assert_eq!(body["supportsTerminateRequest"], true);

        // Verify unsupported capabilities are false
        assert_eq!(body["supportsFunctionBreakpoints"], false);
        assert_eq!(body["supportsStepBack"], false);
        assert_eq!(body["supportsRestartRequest"], false);
    }

    // Full Session Lifecycle Tests

    #[test]
    fn test_full_debug_session_lifecycle() {
        let server = DapServer::new();

        // Create temp file
        let mut temp_file = NamedTempFile::with_suffix(".rs").unwrap();
        writeln!(temp_file, "fn main() {{\n    let x = 42;\n}}").unwrap();
        let file_path = temp_file.path().to_string_lossy().to_string();

        // 1. Initialize
        let init_request = json!({
            "seq": 1,
            "type": "request",
            "command": "initialize",
            "arguments": {"adapterId": "pmat"}
        });
        let response = server.handle_request(init_request);
        assert_eq!(response["success"], true);
        assert!(server.is_initialized());

        // 2. Launch
        let launch_request = json!({
            "seq": 2,
            "type": "request",
            "command": "launch",
            "arguments": {"program": file_path.clone()}
        });
        let response = server.handle_request(launch_request);
        assert_eq!(response["success"], true);
        assert!(server.is_running());

        // 3. ConfigurationDone
        let config_done_request = json!({
            "seq": 3,
            "type": "request",
            "command": "configurationDone",
            "arguments": {}
        });
        let response = server.handle_request(config_done_request);
        assert_eq!(response["success"], true);

        // 4. Set breakpoints
        let bp_request = json!({
            "seq": 4,
            "type": "request",
            "command": "setBreakpoints",
            "arguments": {
                "source": {"path": file_path.clone()},
                "breakpoints": [{"line": 2}]
            }
        });
        let response = server.handle_request(bp_request);
        assert_eq!(response["success"], true);

        // 5. Get threads
        let threads_request = json!({
            "seq": 5,
            "type": "request",
            "command": "threads",
            "arguments": {}
        });
        let response = server.handle_request(threads_request);
        assert_eq!(response["success"], true);

        // 6. Continue
        let continue_request = json!({
            "seq": 6,
            "type": "request",
            "command": "continue",
            "arguments": {"threadId": 1}
        });
        let response = server.handle_request(continue_request);
        assert_eq!(response["success"], true);

        // 7. Disconnect
        let disconnect_request = json!({
            "seq": 7,
            "type": "request",
            "command": "disconnect",
            "arguments": {}
        });
        let response = server.handle_request(disconnect_request);
        assert_eq!(response["success"], true);
        assert!(!server.is_running());
    }

    #[test]
    fn test_full_session_with_recording() {
        let temp_dir = tempdir().unwrap();
        let server = DapServer::with_recording(Some(temp_dir.path().to_path_buf()));

        // Create temp file
        let mut temp_file = NamedTempFile::with_suffix(".rs").unwrap();
        writeln!(temp_file, "fn main() {{\n    let x = 42;\n}}").unwrap();
        let file_path = temp_file.path().to_string_lossy().to_string();

        // Initialize
        let init_request = json!({
            "seq": 1,
            "type": "request",
            "command": "initialize",
            "arguments": {}
        });
        server.handle_request(init_request);

        // Launch
        let launch_request = json!({
            "seq": 2,
            "type": "request",
            "command": "launch",
            "arguments": {"program": file_path}
        });
        server.handle_request(launch_request);

        // Step commands
        let next_request = json!({
            "seq": 3,
            "type": "request",
            "command": "next",
            "arguments": {"threadId": 1}
        });
        server.handle_request(next_request);

        // Terminate (should save recording)
        let terminate_request = json!({
            "seq": 4,
            "type": "request",
            "command": "terminate",
            "arguments": {}
        });
        let response = server.handle_request(terminate_request);
        assert_eq!(response["success"], true);
    }

    // Edge Cases Tests

    #[test]
    fn test_multiple_initialize_calls() {
        let server = DapServer::new();

        for i in 1..=3 {
            let request = json!({
                "seq": i,
                "type": "request",
                "command": "initialize",
                "arguments": {}
            });
            let response = server.handle_request(request);
            assert_eq!(response["success"], true);
        }

        assert!(server.is_initialized());
    }

    #[test]
    fn test_commands_before_initialize() {
        let server = DapServer::new();

        // Threads command should still work (returns default thread)
        let request = json!({
            "seq": 1,
            "type": "request",
            "command": "threads",
            "arguments": {}
        });
        let response = server.handle_request(request);
        assert_eq!(response["success"], true);
    }

    #[test]
    fn test_empty_request_arguments() {
        let server = DapServer::new();

        // Most commands should handle empty arguments gracefully
        let commands = [
            "threads",
            "stackTrace",
            "continue",
            "next",
            "stepIn",
            "stepOut",
            "pause",
        ];

        for (i, cmd) in commands.iter().enumerate() {
            let request = json!({
                "seq": (i + 1) as i64,
                "type": "request",
                "command": cmd,
                "arguments": {}
            });
            let response = server.handle_request(request);
            assert_eq!(response["success"], true, "Command {} should succeed", cmd);
        }
    }

    #[test]
    fn test_large_sequence_numbers() {
        let server = DapServer::new();

        let request = json!({
            "seq": i64::MAX - 1,
            "type": "request",
            "command": "threads",
            "arguments": {}
        });
        let response = server.handle_request(request);

        assert_eq!(response["success"], true);
        assert_eq!(response["request_seq"], i64::MAX - 1);
    }

    #[test]
    fn test_breakpoints_with_conditions() {
        let server = DapServer::new();

        let request = json!({
            "seq": 1,
            "type": "request",
            "command": "setBreakpoints",
            "arguments": {
                "source": {"path": "/test/file.rs"},
                "breakpoints": [
                    { "line": 10, "condition": "x > 5" },
                    { "line": 20, "hitCondition": "3" },
                    { "line": 30, "logMessage": "Hit line 30" }
                ]
            }
        });

        let response = server.handle_request(request);
        assert_eq!(response["success"], true);
    }

    #[test]
    fn test_launch_nonexistent_file() {
        let server = DapServer::new();

        let init_request = json!({
            "seq": 1,
            "type": "request",
            "command": "initialize",
            "arguments": {}
        });
        server.handle_request(init_request);

        let launch_request = json!({
            "seq": 2,
            "type": "request",
            "command": "launch",
            "arguments": {
                "program": "/nonexistent/path/to/program.rs"
            }
        });
        let response = server.handle_request(launch_request);

        // Launch should succeed even if file doesn't exist (runtime error later)
        assert_eq!(response["success"], true);
    }
}
