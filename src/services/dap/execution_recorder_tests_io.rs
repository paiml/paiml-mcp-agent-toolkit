// ExecutionRecorder tests - snapshot conversion, file I/O, and integration tests
// Split from execution_recorder.rs

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests_io {
    use super::*;
    use std::io::Cursor;

    // ========================================================================
    // Convert Snapshot Tests
    // ========================================================================

    #[test]
    fn test_convert_to_recording_snapshot() {
        // Create an ExecutionSnapshot
        let mut variables = HashMap::new();
        variables.insert("x".to_string(), serde_json::json!(42));

        let exec_snapshot = ExecutionSnapshot {
            timestamp: 1_000_000_000, // 1 second in nanoseconds
            sequence: 5,
            variables: variables.clone(),
            call_stack: vec![StackFrame {
                id: 1,
                name: "main".to_string(),
                source: Some(super::super::types::Source {
                    name: Some("test.rs".to_string()),
                    path: Some("/path/to/test.rs".to_string()),
                }),
                line: 42,
                column: 0,
            }],
            location: SourceLocation {
                file: "test.rs".to_string(),
                line: 42,
                column: Some(0),
            },
            delta: None,
        };

        let recording_snapshot =
            ExecutionRecorder::<std::io::Sink>::convert_to_recording_snapshot(&exec_snapshot);

        // Verify conversion
        assert_eq!(recording_snapshot.frame_id, 5); // sequence
        assert_eq!(recording_snapshot.timestamp_relative_ms, 1000); // 1_000_000_000ns = 1000ms (1 second)
        assert_eq!(recording_snapshot.variables, variables);
        assert_eq!(recording_snapshot.stack_frames.len(), 1);
        assert_eq!(recording_snapshot.stack_frames[0].name, "main");
        assert_eq!(
            recording_snapshot.stack_frames[0].file,
            Some("/path/to/test.rs".to_string())
        );
        assert_eq!(recording_snapshot.stack_frames[0].line, Some(42));
    }

    #[test]
    fn test_convert_snapshot_negative_line() {
        let exec_snapshot = ExecutionSnapshot {
            timestamp: 0,
            sequence: 0,
            variables: HashMap::new(),
            call_stack: vec![StackFrame {
                id: 1,
                name: "test".to_string(),
                source: None,
                line: -1, // Negative line
                column: 0,
            }],
            location: SourceLocation {
                file: "test.rs".to_string(),
                line: 1,
                column: None,
            },
            delta: None,
        };

        let recording_snapshot =
            ExecutionRecorder::<std::io::Sink>::convert_to_recording_snapshot(&exec_snapshot);

        // Negative line should not convert
        assert_eq!(recording_snapshot.stack_frames[0].line, None);
    }

    #[test]
    fn test_convert_snapshot_empty_call_stack() {
        let exec_snapshot = ExecutionSnapshot {
            timestamp: 0,
            sequence: 0,
            variables: HashMap::new(),
            call_stack: vec![],
            location: SourceLocation {
                file: "test.rs".to_string(),
                line: 1,
                column: None,
            },
            delta: None,
        };

        let recording_snapshot =
            ExecutionRecorder::<std::io::Sink>::convert_to_recording_snapshot(&exec_snapshot);

        assert!(recording_snapshot.stack_frames.is_empty());
    }

    #[test]
    fn test_convert_snapshot_no_source() {
        let exec_snapshot = ExecutionSnapshot {
            timestamp: 0,
            sequence: 0,
            variables: HashMap::new(),
            call_stack: vec![StackFrame {
                id: 1,
                name: "anonymous".to_string(),
                source: None,
                line: 10,
                column: 0,
            }],
            location: SourceLocation {
                file: "test.rs".to_string(),
                line: 1,
                column: None,
            },
            delta: None,
        };

        let recording_snapshot =
            ExecutionRecorder::<std::io::Sink>::convert_to_recording_snapshot(&exec_snapshot);

        assert_eq!(recording_snapshot.stack_frames[0].file, None);
    }

    // ========================================================================
    // Save/Load File Tests
    // ========================================================================

    #[test]
    fn test_save_to_file_empty() {
        let dap = Arc::new(Mutex::new(DapServer::new()));
        let recorder = ExecutionRecorder::new(dap);

        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("test_recorder_empty.json");

        let result = recorder.save_to_file(temp_file.to_str().unwrap());
        assert!(result.is_ok());

        // Clean up
        let _ = std::fs::remove_file(&temp_file);
    }

    #[test]
    fn test_save_to_file_invalid_path() {
        let dap = Arc::new(Mutex::new(DapServer::new()));
        let recorder = ExecutionRecorder::new(dap);

        let result = recorder.save_to_file("/nonexistent/directory/file.json");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to write file"));
    }

    #[test]
    fn test_load_from_file_nonexistent() {
        let result = ExecutionRecorder::load_from_file("/nonexistent/file.json");
        assert!(result.is_err());
        assert!(result.err().unwrap().contains("Failed to read file"));
    }

    #[test]
    fn test_load_from_file_invalid_json() {
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("test_recorder_invalid.json");

        // Write invalid JSON
        std::fs::write(&temp_file, "not valid json").unwrap();

        let result = ExecutionRecorder::load_from_file(temp_file.to_str().unwrap());
        assert!(result.is_err());
        assert!(result.err().unwrap().contains("Failed to deserialize"));

        // Clean up
        let _ = std::fs::remove_file(&temp_file);
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let dap = Arc::new(Mutex::new(DapServer::new()));
        let recorder = ExecutionRecorder::new(dap);

        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("test_recorder_roundtrip.json");

        // Save empty recording
        recorder.save_to_file(temp_file.to_str().unwrap()).unwrap();

        // Load it back
        let loaded = ExecutionRecorder::load_from_file(temp_file.to_str().unwrap()).unwrap();
        assert_eq!(loaded.snapshot_count(), 0);
        assert!(!loaded.is_recording());

        // Clean up
        let _ = std::fs::remove_file(&temp_file);
    }

    // ========================================================================
    // Integration Tests
    // ========================================================================

    #[test]
    fn test_recorder_lifecycle_without_writer() {
        let dap = Arc::new(Mutex::new(DapServer::new()));
        let mut recorder = ExecutionRecorder::new(dap);

        // Start recording
        recorder.start_recording();
        assert!(recorder.is_recording());

        // Try to capture (will fail due to no stopped location)
        let _ = recorder.capture_snapshot();

        // Stop recording
        recorder.stop_recording();
        assert!(!recorder.is_recording());

        // Finalize
        let result = recorder.finalize();
        assert!(result.is_ok());
    }

    #[test]
    fn test_recorder_lifecycle_with_writer() {
        let buffer = Cursor::new(Vec::new());
        let dap = Arc::new(Mutex::new(DapServer::new()));

        let mut recorder = ExecutionRecorder::with_writer(
            buffer,
            "test_program".to_string(),
            vec!["--verbose".to_string()],
            dap,
        )
        .unwrap();

        // Add environment
        recorder.add_environment("DEBUG", "1");

        // Start recording
        recorder.start_recording();
        assert!(recorder.is_recording());

        // Stop recording
        recorder.stop_recording();
        assert!(!recorder.is_recording());

        // Finalize
        let result = recorder.finalize();
        assert!(result.is_ok());
    }

    #[test]
    fn test_multiple_environment_variables() {
        let buffer = Cursor::new(Vec::new());
        let dap = Arc::new(Mutex::new(DapServer::new()));

        let mut recorder =
            ExecutionRecorder::with_writer(buffer, "program".to_string(), vec![], dap).unwrap();

        // Add multiple environment variables
        recorder.add_environment("PATH", "/usr/bin:/usr/local/bin");
        recorder.add_environment("HOME", "/home/user");
        recorder.add_environment("SHELL", "/bin/bash");
        recorder.add_environment("LANG", "en_US.UTF-8");
        recorder.add_environment("TERM", "xterm-256color");

        // Finalize should work
        let result = recorder.finalize();
        assert!(result.is_ok());
    }
}
