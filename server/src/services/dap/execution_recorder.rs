// TRACE-005: Execution Recording Infrastructure
// Sprint 72 - GREEN Phase
//
// Implements execution recording that captures program state at each step.

use super::server::DapServer;
use super::types::{ExecutionSnapshot, SourceLocation, StackFrame};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// Execution Recorder manages recording of program execution state
pub struct ExecutionRecorder {
    /// All snapshots in chronological order
    snapshots: Vec<ExecutionSnapshot>,
    /// Current recording state
    is_recording: bool,
    /// Integration with DAP server
    dap_server: Arc<Mutex<DapServer>>,
}

impl ExecutionRecorder {
    /// Create a new execution recorder
    pub fn new(dap_server: Arc<Mutex<DapServer>>) -> Self {
        Self {
            snapshots: Vec::new(),
            is_recording: false,
            dap_server,
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

        self.snapshots.push(snapshot.clone());

        Ok(snapshot)
    }

    /// Save recording to file
    pub fn save_to_file(&self, path: &str) -> Result<(), String> {
        let json = serde_json::to_string_pretty(&self.snapshots)
            .map_err(|e| format!("Failed to serialize: {}", e))?;

        std::fs::write(path, json)
            .map_err(|e| format!("Failed to write file: {}", e))?;

        Ok(())
    }

    /// Load recording from file
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
