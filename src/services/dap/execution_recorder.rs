// TRACE-005: Execution Recording Infrastructure
// Sprint 72 - GREEN Phase: In-memory snapshot capture
// Sprint 76 - GREEN Phase: CAPTURE-001 RecordingWriter Integration
//
// Implements execution recording that captures program state at each step.
// Sprint 72 provided in-memory snapshot storage for time-travel debugging.
// Sprint 76 adds optional persistence to .pmat files via RecordingWriter.
//
// Integration Modes:
// 1. Memory-Only: ExecutionRecorder::new() - Sprint 72 backward compatible
// 2. Streaming to File: ExecutionRecorder::with_writer() - Sprint 76 persistence
//
// The recorder is generic over Write trait, enabling flexible output:
// - File::create("session.pmat") for file persistence
// - Cursor::new(Vec::new()) for in-memory .pmat generation
// - TcpStream for network streaming (future)

use super::recording::{RecordingWriter, Snapshot};
use super::server::DapServer;
use super::types::{ExecutionSnapshot, SourceLocation, StackFrame};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// Execution Recorder manages recording of program execution state
///
/// Sprint 76: Now supports optional persistence via RecordingWriter<W>
pub struct ExecutionRecorder<W: Write = std::io::Sink> {
    /// All snapshots in chronological order (in-memory)
    snapshots: Vec<ExecutionSnapshot>,
    /// Current recording state
    is_recording: bool,
    /// Integration with DAP server
    dap_server: Arc<Mutex<DapServer>>,
    /// Optional recording writer for persistence (Sprint 76)
    writer: Option<RecordingWriter<W>>,
}

impl<W: Write> ExecutionRecorder<W> {
    /// Create a new execution recorder with RecordingWriter for persistence
    ///
    /// Sprint 76 - CAPTURE-001: This enables automatic snapshot writing to .pmat files
    ///
    /// # Arguments
    /// * `writer` - Any type implementing Write trait (File, Cursor, TcpStream, etc.)
    /// * `program` - Program name for recording metadata
    /// * `args` - Command-line arguments for recording metadata
    /// * `dap_server` - DAP server for capturing execution state
    ///
    /// # Example
    /// ```rust,no_run
    /// use std::fs::File;
    /// use std::sync::{Arc, Mutex};
    /// use pmat::services::dap::{ExecutionRecorder, DapServer};
    ///
    /// let file = File::create("session.pmat").expect("internal error");
    /// let dap = Arc::new(Mutex::new(DapServer::new()));
    /// let mut recorder = ExecutionRecorder::with_writer(
    ///     file,
    ///     "my_program".to_string(),
    ///     vec!["arg1".to_string(), "arg2".to_string()],
    ///     dap,
    /// ).expect("internal error");
    ///
    /// recorder.start_recording();
    /// // ... capture snapshots during execution ...
    /// recorder.finalize().expect("internal error");
    /// ```
    pub fn with_writer(
        writer: W,
        program: String,
        args: Vec<String>,
        dap_server: Arc<Mutex<DapServer>>,
    ) -> Result<Self> {
        let recording_writer = RecordingWriter::new(writer, program, args)
            .context("Failed to create RecordingWriter")?;

        Ok(Self {
            snapshots: Vec::new(),
            is_recording: false,
            dap_server,
            writer: Some(recording_writer),
        })
    }

    /// Add environment variable to recording metadata
    ///
    /// Sprint 76 - CAPTURE-001: Enriches recording metadata
    pub fn add_environment(&mut self, key: impl Into<String>, value: impl Into<String>) {
        if let Some(ref mut writer) = self.writer {
            writer.add_environment(key, value);
        }
    }

    /// Finalize the recording (must be called to complete .pmat file)
    ///
    /// Sprint 76 - CAPTURE-001: Completes the recording and flushes to disk
    pub fn finalize(self) -> Result<()> {
        if let Some(writer) = self.writer {
            writer.finalize().context("Failed to finalize recording")?;
        }
        Ok(())
    }

    /// Convert ExecutionSnapshot (Sprint 72) to Snapshot (Sprint 75)
    ///
    /// Maps between in-memory snapshot format and .pmat file format
    fn convert_to_recording_snapshot(exec_snapshot: &ExecutionSnapshot) -> Snapshot {
        // Convert Sprint 72 StackFrame to Sprint 75 StackFrame
        let stack_frames = exec_snapshot
            .call_stack
            .iter()
            .map(|frame| {
                let file = frame.source.as_ref().and_then(|s| s.path.clone());
                let line = if frame.line >= 0 {
                    Some(frame.line as u32)
                } else {
                    None
                };

                super::recording::StackFrame {
                    name: frame.name.clone(),
                    file,
                    line,
                    locals: HashMap::new(), // Could extract from variables if needed
                }
            })
            .collect();

        // Calculate timestamp_relative_ms (convert nanoseconds to milliseconds)
        let timestamp_relative_ms = (exec_snapshot.timestamp / 1_000_000) as u32;

        // Use sequence as frame_id
        let frame_id = exec_snapshot.sequence as u64;

        // Instruction pointer: use a placeholder (could extract from location)
        let instruction_pointer = 0u64; // TODO: Could derive from actual IP if available

        Snapshot {
            frame_id,
            timestamp_relative_ms,
            variables: exec_snapshot.variables.clone(),
            stack_frames,
            instruction_pointer,
            memory_snapshot: None, // Not captured in Sprint 72
        }
    }

    /// Start recording execution
    pub fn start_recording(&mut self) {
        self.is_recording = true;
    }

    /// Stop recording execution
    pub fn stop_recording(&mut self) {
        self.is_recording = false;
    }

    /// Check if currently recording
    pub fn is_recording(&self) -> bool {
        self.is_recording
    }

    /// Get the number of snapshots
    pub fn snapshot_count(&self) -> usize {
        self.snapshots.len()
    }

    /// Capture a snapshot of current execution state
    ///
    /// Sprint 76: Now also writes to RecordingWriter if present
    pub fn capture_snapshot(&mut self) -> Result<ExecutionSnapshot, String> {
        if !self.is_recording {
            return Err("Not recording".to_string());
        }

        let dap = self
            .dap_server
            .lock()
            .map_err(|e| format!("Failed to lock DAP server: {}", e))?;

        // Get current stopped file and line
        let stopped_file = dap
            .current_stopped_file()
            .ok_or_else(|| "No file currently stopped at".to_string())?;
        let stopped_line = dap
            .current_stopped_line()
            .ok_or_else(|| "No line currently stopped at".to_string())?;

        // Get variables at current line
        let variables_vec = dap
            .get_variables_at_line(&stopped_file, stopped_line)
            .map_err(|e| format!("Failed to get variables: {}", e))?;

        // Convert Vec<Variable> to HashMap
        let mut variables = HashMap::new();
        for var in variables_vec {
            variables.insert(
                var.name.clone(),
                serde_json::json!({
                    "value": var.value,
                    "type": var.type_info
                }),
            );
        }

        // Create placeholder call stack (simplified for now)
        let call_stack = vec![StackFrame {
            id: 1,
            name: "main".to_string(),
            source: Some(super::types::Source {
                name: Some(stopped_file.clone()),
                path: Some(stopped_file.clone()),
            }),
            line: stopped_line as i64,
            column: 0,
        }];

        // Create snapshot
        let snapshot = ExecutionSnapshot {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("internal error")
                .as_nanos() as u64,
            sequence: self.snapshots.len(),
            variables,
            call_stack,
            location: SourceLocation {
                file: stopped_file,
                line: stopped_line,
                column: Some(0),
            },
            delta: None, // Delta computation will be added in TRACE-006
        };

        // Sprint 76: Write to .pmat file if writer is present
        if let Some(ref mut writer) = self.writer {
            let recording_snapshot = Self::convert_to_recording_snapshot(&snapshot);
            writer
                .write_snapshot(&recording_snapshot)
                .map_err(|e| format!("Failed to write snapshot to recording: {}", e))?;
        }

        self.snapshots.push(snapshot.clone());

        Ok(snapshot)
    }

    /// Save recording to file (Sprint 72 JSON format - deprecated, use .pmat instead)
    pub fn save_to_file(&self, path: &str) -> Result<(), String> {
        let json = serde_json::to_string_pretty(&self.snapshots)
            .map_err(|e| format!("Failed to serialize: {}", e))?;

        std::fs::write(path, json).map_err(|e| format!("Failed to write file: {}", e))?;

        Ok(())
    }
}

impl ExecutionRecorder<std::io::Sink> {
    /// Create a new memory-only execution recorder (Sprint 72 backward compatibility)
    ///
    /// This maintains backward compatibility with existing code that doesn't need persistence
    pub fn new(dap_server: Arc<Mutex<DapServer>>) -> Self {
        Self {
            snapshots: Vec::new(),
            is_recording: false,
            dap_server,
            writer: None,
        }
    }

    /// Load recording from file (Sprint 72 JSON format)
    pub fn load_from_file(path: &str) -> Result<Self, String> {
        let json =
            std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

        let snapshots: Vec<ExecutionSnapshot> =
            serde_json::from_str(&json).map_err(|e| format!("Failed to deserialize: {}", e))?;

        // Create a dummy DAP server for loaded recordings
        let dap_server = Arc::new(Mutex::new(DapServer::new()));

        Ok(Self {
            snapshots,
            is_recording: false,
            dap_server,
            writer: None,
        })
    }
}

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
