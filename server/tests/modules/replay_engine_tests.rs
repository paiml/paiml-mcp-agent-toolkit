#![cfg(feature = "dap")]

// TRACE-007: Replay Engine with Forward/Backward Navigation Tests
// Sprint 72 - RED Phase
//
// Integration tests for time-travel debugging replay engine.
// Tests drive implementation of forward/backward navigation through execution snapshots.

use pmat::services::dap::types::{ExecutionSnapshot, SourceLocation, StackFrame};
use std::collections::HashMap;

// Helper function to create a test recording with 10 snapshots
fn create_test_recording() -> Vec<ExecutionSnapshot> {
    let mut snapshots = Vec::new();

    for i in 0..10 {
        let mut vars = HashMap::new();
        vars.insert("counter".to_string(), serde_json::json!(i));
        vars.insert("step".to_string(), serde_json::json!(format!("step_{}", i)));

        snapshots.push(ExecutionSnapshot {
            timestamp: 1000000 + (i as u64 * 1000),
            sequence: i,
            variables: vars,
            call_stack: vec![StackFrame {
                id: 1,
                name: "main".to_string(),
                source: None,
                line: (10 + i) as i64,
                column: 0,
            }],
            location: SourceLocation {
                file: "test.rs".to_string(),
                line: 10 + i,
                column: Some(0),
            },
            delta: None,
        });
    }

    snapshots
}

// Helper function to create a large recording for performance testing
fn create_large_recording(count: usize) -> Vec<ExecutionSnapshot> {
    let mut snapshots = Vec::new();

    for i in 0..count {
        let mut vars = HashMap::new();
        vars.insert("i".to_string(), serde_json::json!(i));

        snapshots.push(ExecutionSnapshot {
            timestamp: 1000000 + (i as u64 * 1000),
            sequence: i,
            variables: vars,
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
        });
    }

    snapshots
}

// RED Test 1: Create replay engine from recording
#[test]
fn test_create_replay_engine() {
    let recording = create_test_recording();
    let engine = pmat::services::dap::ReplayEngine::from_recording(recording);

    assert_eq!(engine.current_position(), 0, "Should start at position 0");
    assert_eq!(
        engine.total_snapshots(),
        10,
        "Should have 10 total snapshots"
    );
}

// RED Test 2: Step forward
#[test]
fn test_step_forward() {
    let recording = create_test_recording();
    let mut engine = pmat::services::dap::ReplayEngine::from_recording(recording);

    engine.step_forward().unwrap();
    assert_eq!(engine.current_position(), 1, "Should be at position 1");

    engine.step_forward().unwrap();
    assert_eq!(engine.current_position(), 2, "Should be at position 2");
}

// RED Test 3: Step backward
#[test]
fn test_step_backward() {
    let recording = create_test_recording();
    let mut engine = pmat::services::dap::ReplayEngine::from_recording(recording);

    engine.goto(5).unwrap();
    engine.step_backward().unwrap();

    assert_eq!(engine.current_position(), 4, "Should be at position 4");
}

// RED Test 4: Jump to specific position
#[test]
fn test_goto_position() {
    let recording = create_test_recording();
    let mut engine = pmat::services::dap::ReplayEngine::from_recording(recording);

    engine.goto(7).unwrap();
    assert_eq!(engine.current_position(), 7, "Should be at position 7");
}

// RED Test 5: Get current snapshot
#[test]
fn test_get_current_snapshot() {
    let recording = create_test_recording();
    let mut engine = pmat::services::dap::ReplayEngine::from_recording(recording);

    engine.goto(3).unwrap();
    let snapshot = engine.current_snapshot();

    assert_eq!(
        snapshot.sequence, 3,
        "Current snapshot should have sequence 3"
    );
}

// RED Test 6: Cannot step backward from beginning
#[test]
fn test_cannot_step_backward_from_start() {
    let recording = create_test_recording();
    let mut engine = pmat::services::dap::ReplayEngine::from_recording(recording);

    let result = engine.step_backward();
    assert!(
        result.is_err(),
        "Should not be able to step backward from position 0"
    );
}

// RED Test 7: Cannot step forward from end
#[test]
fn test_cannot_step_forward_from_end() {
    let recording = create_test_recording();
    let mut engine = pmat::services::dap::ReplayEngine::from_recording(recording);

    engine.goto(9).unwrap(); // Last position
    let result = engine.step_forward();
    assert!(
        result.is_err(),
        "Should not be able to step forward from last position"
    );
}

// RED Test 8: Replay latency <50ms
#[test]
fn test_replay_performance() {
    let recording = create_large_recording(1000); // 1000 snapshots
    let mut engine = pmat::services::dap::ReplayEngine::from_recording(recording);

    let start = std::time::Instant::now();
    engine.step_forward().unwrap();
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 50,
        "Step forward should be <50ms, got {}ms",
        elapsed.as_millis()
    );
}

// RED Test 9: Multiple backward steps (continuity test)
#[test]
fn test_multiple_backward_steps() {
    let recording = create_test_recording();
    let mut engine = pmat::services::dap::ReplayEngine::from_recording(recording);

    // Jump to middle
    engine.goto(7).unwrap();

    // Step backward multiple times
    engine.step_backward().unwrap();
    assert_eq!(engine.current_position(), 6);

    engine.step_backward().unwrap();
    assert_eq!(engine.current_position(), 5);

    engine.step_backward().unwrap();
    assert_eq!(engine.current_position(), 4);

    // Verify snapshot data matches expected position
    let snapshot = engine.current_snapshot();
    assert_eq!(snapshot.sequence, 4);
}

// RED Test 10: Jump to out-of-bounds position handling
#[test]
fn test_goto_out_of_bounds() {
    let recording = create_test_recording();
    let mut engine = pmat::services::dap::ReplayEngine::from_recording(recording);

    // Try to jump beyond end
    let result = engine.goto(100);
    assert!(
        result.is_err(),
        "Should not be able to jump to position 100 (out of bounds)"
    );

    // Verify position unchanged
    assert_eq!(
        engine.current_position(),
        0,
        "Position should remain at 0 after failed jump"
    );
}
