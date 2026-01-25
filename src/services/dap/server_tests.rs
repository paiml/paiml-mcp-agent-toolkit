// Tests for DAP server
// Extracted for file health compliance (CB-040)

use super::*;

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
