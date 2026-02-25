// Timeline UI tests
// Split from timeline_ui.rs for modularity (include!() pattern)

#[cfg_attr(coverage_nightly, coverage(off))]
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
    fn test_timeline_ui_creation() {
        let snapshots = vec![create_test_snapshot(0), create_test_snapshot(1)];
        let ui = TimelineUI::new(snapshots);

        assert_eq!(ui.current_position(), 0);
    }

    #[test]
    fn test_basic_navigation() {
        let snapshots = vec![
            create_test_snapshot(0),
            create_test_snapshot(1),
            create_test_snapshot(2),
        ];
        let mut ui = TimelineUI::new(snapshots);

        ui.handle_key('→').unwrap();
        assert_eq!(ui.current_position(), 1);

        ui.handle_key('→').unwrap();
        assert_eq!(ui.current_position(), 2);

        ui.handle_key('←').unwrap();
        assert_eq!(ui.current_position(), 1);
    }

    #[test]
    fn test_jump_to() {
        let snapshots = vec![
            create_test_snapshot(0),
            create_test_snapshot(1),
            create_test_snapshot(2),
            create_test_snapshot(3),
            create_test_snapshot(4),
        ];
        let mut ui = TimelineUI::new(snapshots);

        ui.jump_to(3).unwrap();
        assert_eq!(ui.current_position(), 3);
    }
}
