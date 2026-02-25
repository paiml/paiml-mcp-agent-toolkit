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

