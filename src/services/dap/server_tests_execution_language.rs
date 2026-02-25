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

