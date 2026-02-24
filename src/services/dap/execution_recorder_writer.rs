// ExecutionRecorder writer and conversion methods
// Split from execution_recorder.rs - writer construction, environment, finalize, snapshot conversion

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
