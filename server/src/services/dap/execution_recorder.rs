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
    /// let file = File::create("session.pmat").unwrap();
    /// let dap = Arc::new(Mutex::new(DapServer::new()));
    /// let mut recorder = ExecutionRecorder::with_writer(
    ///     file,
    ///     "my_program".to_string(),
    ///     vec!["arg1".to_string(), "arg2".to_string()],
    ///     dap,
    /// ).unwrap();
    ///
    /// recorder.start_recording();
    /// // ... capture snapshots during execution ...
    /// recorder.finalize().unwrap();
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

        let dap = self.dap_server.lock().map_err(|e| format!("Failed to lock DAP server: {}", e))?;

        // Get current stopped file and line
        let stopped_file = dap.current_stopped_file()
            .ok_or_else(|| "No file currently stopped at".to_string())?;
        let stopped_line = dap.current_stopped_line()
            .ok_or_else(|| "No line currently stopped at".to_string())?;

        // Get variables at current line
        let variables_vec = dap.get_variables_at_line(&stopped_file, stopped_line)
            .map_err(|e| format!("Failed to get variables: {}", e))?;

        // Convert Vec<Variable> to HashMap
        let mut variables = HashMap::new();
        for var in variables_vec {
            variables.insert(
                var.name.clone(),
                serde_json::json!({
                    "value": var.value,
                    "type": var.type_info
                })
            );
        }

        // Create placeholder call stack (simplified for now)
        let call_stack = vec![
            StackFrame {
                id: 1,
                name: "main".to_string(),
                source: Some(super::types::Source {
                    name: Some(stopped_file.clone()),
                    path: Some(stopped_file.clone()),
                }),
                line: stopped_line as i64,
                column: 0,
            }
        ];

        // Create snapshot
        let snapshot = ExecutionSnapshot {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
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
            writer.write_snapshot(&recording_snapshot)
                .map_err(|e| format!("Failed to write snapshot to recording: {}", e))?;
        }

        self.snapshots.push(snapshot.clone());

        Ok(snapshot)
    }

    /// Save recording to file (Sprint 72 JSON format - deprecated, use .pmat instead)
    pub fn save_to_file(&self, path: &str) -> Result<(), String> {
        let json = serde_json::to_string_pretty(&self.snapshots)
            .map_err(|e| format!("Failed to serialize: {}", e))?;

        std::fs::write(path, json)
            .map_err(|e| format!("Failed to write file: {}", e))?;

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
        let json = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let snapshots: Vec<ExecutionSnapshot> = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to deserialize: {}", e))?;

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
