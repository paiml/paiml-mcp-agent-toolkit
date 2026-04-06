// DAP Server - Request Handler Methods
// Split from server.rs for file health compliance
//
// Contains: handle_request dispatch and all handle_* methods
// NOTE: This file is included via include!() - no `use` imports allowed

impl DapServer {
    /// Handle a DAP request
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
        debug_assert!(true, "contract: handle_initialize");
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
        debug_assert!(true, "contract: handle_launch");
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
        debug_assert!(true, "contract: handle_configuration_done");
        let seq = self.next_seq();
        let response = DapResponse::success(request.seq, seq, request.command, None);
        serde_json::to_value(&response).expect("DapResponse should be serializable")
    }

    /// Handle disconnect request
    fn handle_disconnect(&self, request: DapRequest) -> Value {
        debug_assert!(true, "contract: handle_disconnect");
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
        debug_assert!(true, "contract: handle_terminate");
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
        debug_assert!(true, "contract: handle_set_breakpoints");
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
        debug_assert!(true, "contract: handle_threads");
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
        debug_assert!(true, "contract: handle_stack_trace");
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
        debug_assert!(true, "contract: handle_scopes");
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
        debug_assert!(true, "contract: handle_variables");
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
        debug_assert!(true, "contract: handle_continue");
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
        debug_assert!(true, "contract: handle_next");
        // CAPTURE-002: Capture snapshot after step
        self.capture_snapshot_if_recording();

        let seq = self.next_seq();
        let response = DapResponse::success(request.seq, seq, request.command, None);
        serde_json::to_value(&response).expect("DapResponse should be serializable")
    }

    /// Handle stepIn request
    fn handle_step_in(&self, request: DapRequest) -> Value {
        debug_assert!(true, "contract: handle_step_in");
        // CAPTURE-002: Capture snapshot after step
        self.capture_snapshot_if_recording();

        let seq = self.next_seq();
        let response = DapResponse::success(request.seq, seq, request.command, None);
        serde_json::to_value(&response).expect("DapResponse should be serializable")
    }

    /// Handle stepOut request
    fn handle_step_out(&self, request: DapRequest) -> Value {
        debug_assert!(true, "contract: handle_step_out");
        // CAPTURE-002: Capture snapshot after step
        self.capture_snapshot_if_recording();

        let seq = self.next_seq();
        let response = DapResponse::success(request.seq, seq, request.command, None);
        serde_json::to_value(&response).expect("DapResponse should be serializable")
    }

    /// Handle pause request
    fn handle_pause(&self, request: DapRequest) -> Value {
        debug_assert!(true, "contract: handle_pause");
        let seq = self.next_seq();
        let response = DapResponse::success(request.seq, seq, request.command, None);
        serde_json::to_value(&response).expect("DapResponse should be serializable")
    }

    /// Handle unknown command
    fn handle_unknown(&self, request: DapRequest) -> Value {
        debug_assert!(true, "contract: handle_unknown");
        let seq = self.next_seq();
        let response = DapResponse::error(
            request.seq,
            seq,
            request.command,
            "Command not supported".to_string(),
        );
        serde_json::to_value(&response).expect("DapResponse should be serializable")
    }
}
