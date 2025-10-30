// TRACE-007: Replay Engine with Forward/Backward Navigation
// Sprint 72 - GREEN Phase
//
// Implements time-travel debugging replay engine for navigating through execution snapshots.

use super::types::ExecutionSnapshot;

/// Replay Engine manages forward/backward navigation through execution snapshots
pub struct ReplayEngine {
    /// All snapshots in chronological order
    snapshots: Vec<ExecutionSnapshot>,
    /// Current position in the recording (0-indexed)
    current_position: usize,
}

impl ReplayEngine {
    /// Create a new replay engine from a recording
    pub fn from_recording(snapshots: Vec<ExecutionSnapshot>) -> Self {
        Self {
            snapshots,
            current_position: 0,
        }
    }

    /// Get current position in the recording
    pub fn current_position(&self) -> usize {
        self.current_position
    }

    /// Get total number of snapshots
    pub fn total_snapshots(&self) -> usize {
        self.snapshots.len()
    }

    /// Step forward one snapshot
    pub fn step_forward(&mut self) -> Result<(), String> {
        if self.current_position >= self.snapshots.len() - 1 {
            return Err("Cannot step forward: already at last snapshot".to_string());
        }
        self.current_position += 1;
        Ok(())
    }

    /// Step backward one snapshot
    pub fn step_backward(&mut self) -> Result<(), String> {
        if self.current_position == 0 {
            return Err("Cannot step backward: already at first snapshot".to_string());
        }
        self.current_position -= 1;
        Ok(())
    }

    /// Jump to specific position
    pub fn goto(&mut self, position: usize) -> Result<(), String> {
        if position >= self.snapshots.len() {
            return Err(format!(
                "Cannot goto position {}: out of bounds (max: {})",
                position,
                self.snapshots.len() - 1
            ));
        }
        self.current_position = position;
        Ok(())
    }

    /// Get current snapshot
    pub fn current_snapshot(&self) -> &ExecutionSnapshot {
        &self.snapshots[self.current_position]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::dap::types::{SourceLocation, StackFrame};
    use std::collections::HashMap;

    fn create_test_snapshot(sequence: usize) -> ExecutionSnapshot {
        ExecutionSnapshot {
            timestamp: 1000000 + (sequence as u64 * 1000),
            sequence,
            variables: HashMap::new(),
            call_stack: vec![StackFrame {
                id: 1,
                name: "main".to_string(),
                source: None,
                line: 10,
                column: 0,
            }],
            location: SourceLocation {
                file: "test.rs".to_string(),
                line: 10,
                column: Some(0),
            },
            delta: None,
        }
    }

    #[test]
    fn test_replay_engine_basic_creation() {
        let snapshots = vec![create_test_snapshot(0), create_test_snapshot(1)];
        let engine = ReplayEngine::from_recording(snapshots);

        assert_eq!(engine.current_position(), 0);
        assert_eq!(engine.total_snapshots(), 2);
    }

    #[test]
    fn test_step_forward_basic() {
        let snapshots = vec![create_test_snapshot(0), create_test_snapshot(1)];
        let mut engine = ReplayEngine::from_recording(snapshots);

        engine.step_forward().unwrap();
        assert_eq!(engine.current_position(), 1);
    }

    #[test]
    fn test_step_backward_basic() {
        let snapshots = vec![create_test_snapshot(0), create_test_snapshot(1)];
        let mut engine = ReplayEngine::from_recording(snapshots);

        engine.goto(1).unwrap();
        engine.step_backward().unwrap();
        assert_eq!(engine.current_position(), 0);
    }
}
