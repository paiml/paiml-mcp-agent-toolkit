// DAP (Debug Adapter Protocol) Server Implementation
// Sprint 71 - TRACE-001: DAP Protocol Server Implementation
//
// This implements a Debug Adapter Protocol server for PMAT
// allowing integration with VSCode and other DAP-compatible debuggers

use super::types::*;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

/// Server state enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerState {
    Uninitialized,
    Initialized,
    Running,
    Stopped,
}

/// DAP Server structure
#[derive(Debug)]
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
}

impl DapServer {
    /// Create a new DAP server instance
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(ServerState::Uninitialized)),
            response_seq: Arc::new(Mutex::new(0)),
            capabilities: Self::default_capabilities(),
            program: Arc::new(Mutex::new(None)),
            breakpoints: Arc::new(Mutex::new(HashMap::new())),
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
        let mut seq = self.response_seq.lock().unwrap();
        *seq += 1;
        *seq
    }

    /// Check if server is initialized
    pub fn is_initialized(&self) -> bool {
        let state = self.state.lock().unwrap();
        *state != ServerState::Uninitialized
    }

    /// Check if server is running
    pub fn is_running(&self) -> bool {
        let state = self.state.lock().unwrap();
        *state == ServerState::Running
    }

    /// Check if program is loaded
    pub fn has_program_loaded(&self) -> bool {
        let program = self.program.lock().unwrap();
        program.is_some()
    }

    /// Get current program path
    pub fn current_program(&self) -> Option<String> {
        let program = self.program.lock().unwrap();
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
        let mut state = self.state.lock().unwrap();
        *state = ServerState::Initialized;
        drop(state);

        let seq = self.next_seq();
        let response = DapResponse::success(
            request.seq,
            seq,
            request.command,
            Some(serde_json::to_value(&self.capabilities).unwrap()),
        );

        serde_json::to_value(&response).unwrap()
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
                return serde_json::to_value(&response).unwrap();
            }
        };

        // Store program
        let mut program = self.program.lock().unwrap();
        *program = Some(args.program.clone());
        drop(program);

        // Update state
        let mut state = self.state.lock().unwrap();
        *state = ServerState::Running;
        drop(state);

        let seq = self.next_seq();
        let response = DapResponse::success(request.seq, seq, request.command, None);

        serde_json::to_value(&response).unwrap()
    }

    /// Handle configurationDone request
    fn handle_configuration_done(&self, request: DapRequest) -> Value {
        let seq = self.next_seq();
        let response = DapResponse::success(request.seq, seq, request.command, None);
        serde_json::to_value(&response).unwrap()
    }

    /// Handle disconnect request
    fn handle_disconnect(&self, request: DapRequest) -> Value {
        let mut state = self.state.lock().unwrap();
        *state = ServerState::Stopped;
        drop(state);

        let seq = self.next_seq();
        let response = DapResponse::success(request.seq, seq, request.command, None);
        serde_json::to_value(&response).unwrap()
    }

    /// Handle terminate request
    fn handle_terminate(&self, request: DapRequest) -> Value {
        let mut state = self.state.lock().unwrap();
        *state = ServerState::Stopped;
        drop(state);

        let seq = self.next_seq();
        let response = DapResponse::success(request.seq, seq, request.command, None);
        serde_json::to_value(&response).unwrap()
    }

    /// Handle setBreakpoints request
    fn handle_set_breakpoints(&self, request: DapRequest) -> Value {
        // Parse setBreakpoints arguments
        let args: SetBreakpointsArguments = match serde_json::from_value(request.arguments.clone()) {
            Ok(args) => args,
            Err(e) => {
                let seq = self.next_seq();
                let response = DapResponse::error(
                    request.seq,
                    seq,
                    request.command,
                    format!("Invalid setBreakpoints arguments: {}", e),
                );
                return serde_json::to_value(&response).unwrap();
            }
        };

        // Store breakpoints
        let source_path = args.source.path.unwrap_or_else(|| "unknown".to_string());
        let mut breakpoints_map = self.breakpoints.lock().unwrap();

        if let Some(bps) = args.breakpoints {
            let lines: HashSet<i64> = bps.iter().map(|bp| bp.line).collect();
            breakpoints_map.insert(source_path, lines);
        } else {
            breakpoints_map.remove(&source_path);
        }
        drop(breakpoints_map);

        let seq = self.next_seq();
        let response = DapResponse::success(
            request.seq,
            seq,
            request.command,
            Some(json!({"breakpoints": []})),
        );

        serde_json::to_value(&response).unwrap()
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
        serde_json::to_value(&response).unwrap()
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
        serde_json::to_value(&response).unwrap()
    }

    /// Handle scopes request
    fn handle_scopes(&self, request: DapRequest) -> Value {
        let seq = self.next_seq();
        let response = DapResponse::success(
            request.seq,
            seq,
            request.command,
            Some(json!({"scopes": []})),
        );
        serde_json::to_value(&response).unwrap()
    }

    /// Handle variables request
    fn handle_variables(&self, request: DapRequest) -> Value {
        let seq = self.next_seq();
        let response = DapResponse::success(
            request.seq,
            seq,
            request.command,
            Some(json!({"variables": []})),
        );
        serde_json::to_value(&response).unwrap()
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
        serde_json::to_value(&response).unwrap()
    }

    /// Handle next request (step over)
    fn handle_next(&self, request: DapRequest) -> Value {
        let seq = self.next_seq();
        let response = DapResponse::success(request.seq, seq, request.command, None);
        serde_json::to_value(&response).unwrap()
    }

    /// Handle stepIn request
    fn handle_step_in(&self, request: DapRequest) -> Value {
        let seq = self.next_seq();
        let response = DapResponse::success(request.seq, seq, request.command, None);
        serde_json::to_value(&response).unwrap()
    }

    /// Handle stepOut request
    fn handle_step_out(&self, request: DapRequest) -> Value {
        let seq = self.next_seq();
        let response = DapResponse::success(request.seq, seq, request.command, None);
        serde_json::to_value(&response).unwrap()
    }

    /// Handle pause request
    fn handle_pause(&self, request: DapRequest) -> Value {
        let seq = self.next_seq();
        let response = DapResponse::success(request.seq, seq, request.command, None);
        serde_json::to_value(&response).unwrap()
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
        serde_json::to_value(&response).unwrap()
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
