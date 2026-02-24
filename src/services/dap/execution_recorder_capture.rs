// ExecutionRecorder capture and file I/O methods
// Split from execution_recorder.rs - snapshot capture, save/load, memory-only constructor

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
