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

