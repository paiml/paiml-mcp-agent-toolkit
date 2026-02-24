// ExecutionRecorder tests - core and state tests
// Split from execution_recorder.rs

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_recorder_creation() {
        let dap = Arc::new(Mutex::new(DapServer::new()));
        let recorder = ExecutionRecorder::new(dap);

        assert!(!recorder.is_recording());
        assert_eq!(recorder.snapshot_count(), 0);
    }

    #[test]
    fn test_start_stop_recording() {
        let dap = Arc::new(Mutex::new(DapServer::new()));
        let mut recorder = ExecutionRecorder::new(dap);

        recorder.start_recording();
        assert!(recorder.is_recording());

        recorder.stop_recording();
        assert!(!recorder.is_recording());
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;
    use std::io::Cursor;

    // ========================================================================
    // ExecutionRecorder Creation Tests
    // ========================================================================

    #[test]
    fn test_recorder_new_memory_only() {
        let dap = Arc::new(Mutex::new(DapServer::new()));
        let recorder = ExecutionRecorder::new(dap);

        assert!(!recorder.is_recording());
        assert_eq!(recorder.snapshot_count(), 0);
    }

    #[test]
    fn test_recorder_with_writer_creation() {
        let buffer = Cursor::new(Vec::new());
        let dap = Arc::new(Mutex::new(DapServer::new()));

        let recorder = ExecutionRecorder::with_writer(
            buffer,
            "test_program".to_string(),
            vec!["--flag".to_string()],
            dap,
        );

        assert!(recorder.is_ok());
        let recorder = recorder.unwrap();
        assert!(!recorder.is_recording());
        assert_eq!(recorder.snapshot_count(), 0);
    }

    #[test]
    fn test_recorder_with_writer_empty_args() {
        let buffer = Cursor::new(Vec::new());
        let dap = Arc::new(Mutex::new(DapServer::new()));

        let recorder =
            ExecutionRecorder::with_writer(buffer, "program".to_string(), vec![], dap).unwrap();

        assert!(!recorder.is_recording());
    }

    // ========================================================================
    // Recording State Tests
    // ========================================================================

    #[test]
    fn test_start_recording_sets_state() {
        let dap = Arc::new(Mutex::new(DapServer::new()));
        let mut recorder = ExecutionRecorder::new(dap);

        assert!(!recorder.is_recording());
        recorder.start_recording();
        assert!(recorder.is_recording());
    }

    #[test]
    fn test_stop_recording_clears_state() {
        let dap = Arc::new(Mutex::new(DapServer::new()));
        let mut recorder = ExecutionRecorder::new(dap);

        recorder.start_recording();
        assert!(recorder.is_recording());

        recorder.stop_recording();
        assert!(!recorder.is_recording());
    }

    #[test]
    fn test_multiple_start_stop_cycles() {
        let dap = Arc::new(Mutex::new(DapServer::new()));
        let mut recorder = ExecutionRecorder::new(dap);

        for _ in 0..5 {
            recorder.start_recording();
            assert!(recorder.is_recording());
            recorder.stop_recording();
            assert!(!recorder.is_recording());
        }
    }

    #[test]
    fn test_is_recording_returns_correct_state() {
        let dap = Arc::new(Mutex::new(DapServer::new()));
        let mut recorder = ExecutionRecorder::new(dap);

        // Initially not recording
        assert!(!recorder.is_recording());

        // After start
        recorder.start_recording();
        assert!(recorder.is_recording());

        // After stop
        recorder.stop_recording();
        assert!(!recorder.is_recording());
    }

    // ========================================================================
    // Snapshot Count Tests
    // ========================================================================

    #[test]
    fn test_snapshot_count_initially_zero() {
        let dap = Arc::new(Mutex::new(DapServer::new()));
        let recorder = ExecutionRecorder::new(dap);

        assert_eq!(recorder.snapshot_count(), 0);
    }

    // ========================================================================
    // Capture Snapshot Tests
    // ========================================================================

    #[test]
    fn test_capture_snapshot_requires_recording() {
        let dap = Arc::new(Mutex::new(DapServer::new()));
        let mut recorder = ExecutionRecorder::new(dap);

        // Not recording - should fail
        let result = recorder.capture_snapshot();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Not recording"));
    }

    #[test]
    fn test_capture_snapshot_requires_stopped_location() {
        let dap = Arc::new(Mutex::new(DapServer::new()));
        let mut recorder = ExecutionRecorder::new(dap);

        recorder.start_recording();

        // No stopped location set - should fail
        let result = recorder.capture_snapshot();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No file currently stopped at"));
    }

    // ========================================================================
    // Add Environment Tests
    // ========================================================================

    #[test]
    fn test_add_environment_with_writer() {
        let buffer = Cursor::new(Vec::new());
        let dap = Arc::new(Mutex::new(DapServer::new()));

        let mut recorder =
            ExecutionRecorder::with_writer(buffer, "program".to_string(), vec![], dap).unwrap();

        // Should not panic
        recorder.add_environment("PATH", "/usr/bin");
        recorder.add_environment("HOME", "/home/user");
    }

    #[test]
    fn test_add_environment_without_writer() {
        let dap = Arc::new(Mutex::new(DapServer::new()));
        let mut recorder = ExecutionRecorder::new(dap);

        // Should not panic even without writer
        recorder.add_environment("KEY", "value");
    }

    #[test]
    fn test_add_environment_multiple_entries() {
        let buffer = Cursor::new(Vec::new());
        let dap = Arc::new(Mutex::new(DapServer::new()));

        let mut recorder =
            ExecutionRecorder::with_writer(buffer, "program".to_string(), vec![], dap).unwrap();

        recorder.add_environment("VAR1", "value1");
        recorder.add_environment("VAR2", "value2");
        recorder.add_environment("VAR3", "value3");

        // Still functional
        assert!(!recorder.is_recording());
    }

    // ========================================================================
    // Finalize Tests
    // ========================================================================

    #[test]
    fn test_finalize_without_writer() {
        let dap = Arc::new(Mutex::new(DapServer::new()));
        let recorder = ExecutionRecorder::new(dap);

        // Finalize should succeed even without writer
        let result = recorder.finalize();
        assert!(result.is_ok());
    }

    #[test]
    fn test_finalize_with_writer_no_snapshots() {
        let buffer = Cursor::new(Vec::new());
        let dap = Arc::new(Mutex::new(DapServer::new()));

        let recorder =
            ExecutionRecorder::with_writer(buffer, "program".to_string(), vec![], dap).unwrap();

        // Finalize empty recording
        let result = recorder.finalize();
        assert!(result.is_ok());
    }
}
