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

