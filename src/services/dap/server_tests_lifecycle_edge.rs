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
