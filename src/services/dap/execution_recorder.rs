#![cfg_attr(coverage_nightly, coverage(off))]
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
    include!("execution_recorder_writer.rs");
    include!("execution_recorder_capture.rs");

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

include!("execution_recorder_tests.rs");
include!("execution_recorder_tests_io.rs");
