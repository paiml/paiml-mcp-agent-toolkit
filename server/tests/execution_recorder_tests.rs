// TRACE-005: Execution Recording Infrastructure Tests
// Sprint 72 - RED Phase
//
// Integration tests for execution recording that captures program state at each step.
// Tests drive implementation of ExecutionRecorder and ExecutionSnapshot types.

use pmat::services::dap::{DapServer, ExecutionRecorder};
use std::sync::{Arc, Mutex};

// RED Test 1: Create recorder
#[test]
fn test_create_execution_recorder() {
    let dap_server = Arc::new(Mutex::new(DapServer::new()));
    let recorder = ExecutionRecorder::new(dap_server);

    assert!(
        !recorder.is_recording(),
        "New recorder should not be recording"
    );
    assert_eq!(recorder.snapshot_count(), 0, "New recorder should have 0 snapshots");
}

// RED Test 2: Start recording
#[test]
fn test_start_recording() {
    let dap_server = Arc::new(Mutex::new(DapServer::new()));
    let mut recorder = ExecutionRecorder::new(dap_server);

    recorder.start_recording();
    assert!(recorder.is_recording(), "Should be recording after start");
}

// RED Test 3: Stop recording
#[test]
fn test_stop_recording() {
    let dap_server = Arc::new(Mutex::new(DapServer::new()));
    let mut recorder = ExecutionRecorder::new(dap_server);

    recorder.start_recording();
    recorder.stop_recording();

    assert!(!recorder.is_recording(), "Should not be recording after stop");
}

// RED Test 4: Capture snapshot
#[test]
fn test_capture_snapshot() {
    let mut dap_server = DapServer::new();

    // Initialize and launch
    let init_request = serde_json::json!({
        "seq": 1,
        "type": "request",
        "command": "initialize",
        "arguments": {"clientID": "test", "adapterID": "pmat"}
    });
    dap_server.handle_request(init_request);

    let launch_request = serde_json::json!({
        "seq": 2,
        "type": "request",
        "command": "launch",
        "arguments": {"program": "tests/fixtures/sample.rs"}
    });
    dap_server.handle_request(launch_request);

    dap_server.simulate_stop_at_line("tests/fixtures/sample.rs", 3);

    let dap = Arc::new(Mutex::new(dap_server));
    let mut recorder = ExecutionRecorder::new(dap);

    recorder.start_recording();
    let snapshot = recorder.capture_snapshot().expect("Should capture snapshot");

    assert_eq!(snapshot.sequence, 0, "First snapshot should have sequence 0");
    assert!(snapshot.timestamp > 0, "Snapshot should have valid timestamp");
    assert_eq!(
        snapshot.location.line, 3,
        "Snapshot should capture correct line"
    );
}

// RED Test 5: Multiple snapshots
#[test]
fn test_multiple_snapshots() {
    let dap_server = DapServer::new();

    // Initialize and launch
    let init_request = serde_json::json!({
        "seq": 1,
        "type": "request",
        "command": "initialize",
        "arguments": {"clientID": "test", "adapterID": "pmat"}
    });
    dap_server.handle_request(init_request);

    let launch_request = serde_json::json!({
        "seq": 2,
        "type": "request",
        "command": "launch",
        "arguments": {"program": "tests/fixtures/sample.rs"}
    });
    dap_server.handle_request(launch_request);

    let dap = Arc::new(Mutex::new(dap_server));
    let mut recorder = ExecutionRecorder::new(dap.clone());

    recorder.start_recording();

    // Capture 3 snapshots at different lines
    for i in 2..=4 {
        {
            let mut server = dap.lock().unwrap();
            server.simulate_stop_at_line("tests/fixtures/sample.rs", i);
        }
        recorder.capture_snapshot().expect("Should capture snapshot");
    }

    assert_eq!(
        recorder.snapshot_count(),
        3,
        "Should have captured 3 snapshots"
    );
}

// RED Test 6: Snapshot contains variables
#[test]
fn test_snapshot_captures_variables() {
    let mut dap_server = DapServer::new();

    // Initialize and launch
    let init_request = serde_json::json!({
        "seq": 1,
        "type": "request",
        "command": "initialize",
        "arguments": {"clientID": "test", "adapterID": "pmat"}
    });
    dap_server.handle_request(init_request);

    let launch_request = serde_json::json!({
        "seq": 2,
        "type": "request",
        "command": "launch",
        "arguments": {"program": "tests/fixtures/sample.rs"}
    });
    dap_server.handle_request(launch_request);

    dap_server.simulate_stop_at_line("tests/fixtures/sample.rs", 3);

    let dap = Arc::new(Mutex::new(dap_server));
    let mut recorder = ExecutionRecorder::new(dap);

    recorder.start_recording();
    let snapshot = recorder.capture_snapshot().expect("Should capture snapshot");

    assert!(
        !snapshot.variables.is_empty(),
        "Snapshot should capture variables"
    );
}

// RED Test 7: Snapshot contains call stack
#[test]
fn test_snapshot_captures_call_stack() {
    let mut dap_server = DapServer::new();

    // Initialize and launch
    let init_request = serde_json::json!({
        "seq": 1,
        "type": "request",
        "command": "initialize",
        "arguments": {"clientID": "test", "adapterID": "pmat"}
    });
    dap_server.handle_request(init_request);

    let launch_request = serde_json::json!({
        "seq": 2,
        "type": "request",
        "command": "launch",
        "arguments": {"program": "tests/fixtures/complex.rs"}
    });
    dap_server.handle_request(launch_request);

    dap_server.simulate_stop_at_line("tests/fixtures/complex.rs", 11);

    let dap = Arc::new(Mutex::new(dap_server));
    let mut recorder = ExecutionRecorder::new(dap);

    recorder.start_recording();
    let snapshot = recorder.capture_snapshot().expect("Should capture snapshot");

    assert!(
        !snapshot.call_stack.is_empty(),
        "Snapshot should capture call stack"
    );
}

// RED Test 8: Save recording to file
#[test]
fn test_save_recording_to_file() {
    let dap_server = DapServer::new();

    // Initialize and launch
    let init_request = serde_json::json!({
        "seq": 1,
        "type": "request",
        "command": "initialize",
        "arguments": {"clientID": "test", "adapterID": "pmat"}
    });
    dap_server.handle_request(init_request);

    let launch_request = serde_json::json!({
        "seq": 2,
        "type": "request",
        "command": "launch",
        "arguments": {"program": "tests/fixtures/sample.rs"}
    });
    dap_server.handle_request(launch_request);

    let dap = Arc::new(Mutex::new(dap_server));
    let mut recorder = ExecutionRecorder::new(dap.clone());

    recorder.start_recording();

    // Capture 3 snapshots
    for i in 2..=4 {
        {
            let mut server = dap.lock().unwrap();
            server.simulate_stop_at_line("tests/fixtures/sample.rs", i);
        }
        recorder.capture_snapshot().expect("Should capture snapshot");
    }

    let output_path = "/tmp/test_recording.pmat";
    recorder
        .save_to_file(output_path)
        .expect("Should save to file");

    assert!(
        std::path::Path::new(output_path).exists(),
        "Recording file should exist"
    );

    // Cleanup
    let _ = std::fs::remove_file(output_path);
}

// RED Test 9: Load recording from file
#[test]
fn test_load_recording_from_file() {
    let dap_server = DapServer::new();

    // Initialize and launch
    let init_request = serde_json::json!({
        "seq": 1,
        "type": "request",
        "command": "initialize",
        "arguments": {"clientID": "test", "adapterID": "pmat"}
    });
    dap_server.handle_request(init_request);

    let launch_request = serde_json::json!({
        "seq": 2,
        "type": "request",
        "command": "launch",
        "arguments": {"program": "tests/fixtures/sample.rs"}
    });
    dap_server.handle_request(launch_request);

    let dap = Arc::new(Mutex::new(dap_server));
    let mut recorder = ExecutionRecorder::new(dap.clone());

    recorder.start_recording();

    // Capture 3 snapshots
    for i in 2..=4 {
        {
            let mut server = dap.lock().unwrap();
            server.simulate_stop_at_line("tests/fixtures/sample.rs", i);
        }
        recorder.capture_snapshot().expect("Should capture snapshot");
    }

    let file_path = "/tmp/test_load.pmat";
    recorder.save_to_file(file_path).expect("Should save to file");

    // Now load it
    let loaded_recorder =
        ExecutionRecorder::load_from_file(file_path).expect("Should load from file");

    assert_eq!(
        loaded_recorder.snapshot_count(),
        3,
        "Loaded recorder should have 3 snapshots"
    );

    // Cleanup
    let _ = std::fs::remove_file(file_path);
}

// RED Test 10: Cannot capture when not recording
#[test]
fn test_cannot_capture_when_not_recording() {
    let dap = Arc::new(Mutex::new(DapServer::new()));
    let mut recorder = ExecutionRecorder::new(dap);

    let result = recorder.capture_snapshot();
    assert!(
        result.is_err(),
        "Should fail when capturing without recording"
    );

    if let Err(msg) = result {
        assert!(
            msg.contains("not recording") || msg.contains("Not recording"),
            "Error should mention not recording state. Got: {}",
            msg
        );
    }
}
