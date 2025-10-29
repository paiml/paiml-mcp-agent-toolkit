// TRACE-001: DAP Protocol Server Implementation
// RED Phase Tests - Sprint 71
//
// Debug Adapter Protocol (DAP) server foundation tests
// These tests define the specification for PMAT's DAP server

use serde_json::{json, Value};

// Test fixtures and helpers
fn create_initialize_request() -> Value {
    json!({
        "seq": 1,
        "type": "request",
        "command": "initialize",
        "arguments": {
            "clientID": "vscode",
            "adapterID": "pmat-debug",
            "linesStartAt1": true,
            "columnsStartAt1": true,
            "pathFormat": "path",
            "supportsVariableType": true,
            "supportsVariablePaging": false
        }
    })
}

fn create_launch_request(program: &str) -> Value {
    json!({
        "seq": 2,
        "type": "request",
        "command": "launch",
        "arguments": {
            "program": program,
            "stopOnEntry": true,
            "cwd": "${workspaceFolder}"
        }
    })
}

fn create_invalid_request() -> Value {
    json!({
        "seq": 999,
        "type": "request",
        "command": "nonexistent_command",
        "arguments": {}
    })
}

// RED Phase Test 1: DAP Server Initialization
#[test]
fn test_dap_server_initialization() {
    // This test will fail until DapServer is implemented

    // Create a new DAP server instance
    let server = pmat::services::dap::DapServer::new();

    // Send initialize request
    let init_request = create_initialize_request();
    let response = server.handle_request(init_request);

    // Verify response structure
    assert_eq!(response["type"], "response");
    assert_eq!(response["command"], "initialize");
    assert_eq!(response["success"], true);
    assert_eq!(response["request_seq"], 1);

    // Verify capabilities are returned
    let body = &response["body"];
    assert!(body.is_object(), "Response body should be an object");
    assert!(body["supportsConfigurationDoneRequest"].as_bool().unwrap_or(false));

    // Server should be in initialized state
    assert!(server.is_initialized(), "Server should be initialized after initialize request");
}

// RED Phase Test 2: DAP Launch Request
#[test]
fn test_dap_server_launch_request() {
    // This test will fail until DapServer launch is implemented

    let server = pmat::services::dap::DapServer::new();

    // Initialize first
    let init_request = create_initialize_request();
    server.handle_request(init_request);

    // Send configurationDone
    let config_done = json!({
        "seq": 2,
        "type": "request",
        "command": "configurationDone"
    });
    server.handle_request(config_done);

    // Send launch request
    let launch_request = create_launch_request("tests/fixtures/sample.rs");
    let response = server.handle_request(launch_request);

    // Verify response
    assert_eq!(response["type"], "response");
    assert_eq!(response["command"], "launch");
    assert_eq!(response["success"], true);

    // Server should have loaded the program
    assert!(server.has_program_loaded(), "Server should have program loaded");
    assert_eq!(
        server.current_program().unwrap(),
        "tests/fixtures/sample.rs",
        "Server should track current program"
    );
}

// RED Phase Test 3: Invalid Request Handling
#[test]
fn test_dap_server_handles_invalid_request() {
    // This test will fail until error handling is implemented

    let server = pmat::services::dap::DapServer::new();

    // Send invalid request
    let invalid_request = create_invalid_request();
    let response = server.handle_request(invalid_request);

    // Verify error response
    assert_eq!(response["type"], "response");
    assert_eq!(response["command"], "nonexistent_command");
    assert_eq!(response["success"], false);

    // Error message should be present
    let message = response["message"].as_str().unwrap_or("");
    assert!(!message.is_empty(), "Error message should be present");
    assert!(
        message.contains("unknown") || message.contains("not supported"),
        "Error message should indicate unknown command"
    );
}

// RED Phase Test 4: Request Sequence Tracking
#[test]
fn test_dap_server_tracks_request_sequence() {
    // This test ensures request/response seq matching

    let server = pmat::services::dap::DapServer::new();

    // Send multiple requests
    let req1 = json!({"seq": 1, "type": "request", "command": "initialize", "arguments": {}});
    let req2 = json!({"seq": 2, "type": "request", "command": "disconnect", "arguments": {}});
    let req3 = json!({"seq": 3, "type": "request", "command": "terminate", "arguments": {}});

    let resp1 = server.handle_request(req1);
    let resp2 = server.handle_request(req2);
    let resp3 = server.handle_request(req3);

    // Each response should have request_seq matching the request
    assert_eq!(resp1["request_seq"], 1);
    assert_eq!(resp2["request_seq"], 2);
    assert_eq!(resp3["request_seq"], 3);

    // Response seq should be unique and incrementing
    assert!(resp1["seq"].as_i64().unwrap() > 0);
    assert!(resp2["seq"].as_i64().unwrap() > resp1["seq"].as_i64().unwrap());
    assert!(resp3["seq"].as_i64().unwrap() > resp2["seq"].as_i64().unwrap());
}

// RED Phase Test 5: Server State Machine
#[test]
fn test_dap_server_state_machine() {
    // This test ensures proper state transitions

    let server = pmat::services::dap::DapServer::new();

    // Initial state should be uninitialized
    assert!(!server.is_initialized(), "Server should start uninitialized");
    assert!(!server.is_running(), "Server should not be running initially");

    // Initialize
    let init_req = create_initialize_request();
    server.handle_request(init_req);
    assert!(server.is_initialized(), "Server should be initialized");

    // Configuration done
    let config_done = json!({"seq": 2, "type": "request", "command": "configurationDone"});
    server.handle_request(config_done);

    // Launch
    let launch_req = create_launch_request("tests/fixtures/sample.rs");
    server.handle_request(launch_req);
    assert!(server.is_running(), "Server should be running after launch");

    // Disconnect
    let disconnect = json!({"seq": 4, "type": "request", "command": "disconnect"});
    server.handle_request(disconnect);
    assert!(!server.is_running(), "Server should not be running after disconnect");
}

// RED Phase Test 6: DAP Protocol Compliance
#[test]
fn test_dap_protocol_compliance() {
    // Ensure responses conform to DAP specification

    let server = pmat::services::dap::DapServer::new();
    let init_req = create_initialize_request();
    let response = server.handle_request(init_req);

    // All responses must have these fields
    assert!(response.get("seq").is_some(), "Response must have 'seq' field");
    assert!(response.get("type").is_some(), "Response must have 'type' field");
    assert!(response.get("request_seq").is_some(), "Response must have 'request_seq' field");
    assert!(response.get("success").is_some(), "Response must have 'success' field");
    assert!(response.get("command").is_some(), "Response must have 'command' field");

    // Type must be "response"
    assert_eq!(response["type"], "response", "Type must be 'response'");

    // Success must be boolean
    assert!(response["success"].is_boolean(), "Success must be boolean");
}

// RED Phase Test 7: Concurrent Request Handling
#[test]
fn test_dap_server_handles_concurrent_requests() {
    // Ensure server can handle requests in parallel (future requirement)

    use std::sync::{Arc, Mutex};
    use std::thread;

    let server = Arc::new(Mutex::new(pmat::services::dap::DapServer::new()));
    let mut handles = vec![];

    // Spawn 10 threads making initialize requests
    for i in 0..10 {
        let server_clone = Arc::clone(&server);
        let handle = thread::spawn(move || {
            let req = json!({
                "seq": i,
                "type": "request",
                "command": "initialize",
                "arguments": {}
            });

            let server = server_clone.lock().unwrap();
            let response = server.handle_request(req);
            assert_eq!(response["success"], true);
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }
}

// RED Phase Test 8: Memory Safety
#[test]
fn test_dap_server_memory_safety() {
    // Ensure server doesn't leak memory with repeated requests

    let server = pmat::services::dap::DapServer::new();

    // Send 1000 requests
    for i in 0..1000 {
        let req = json!({
            "seq": i,
            "type": "request",
            "command": "initialize",
            "arguments": {}
        });

        let _ = server.handle_request(req);
    }

    // If we get here without panicking, memory safety is maintained
    // (Rust's ownership system ensures this, but test documents the expectation)
}

// RED Phase Test 9: Error Recovery
#[test]
fn test_dap_server_error_recovery() {
    // Ensure server recovers from malformed requests

    let server = pmat::services::dap::DapServer::new();

    // Send malformed request (missing required fields)
    let malformed = json!({
        "type": "request",
        // Missing seq, command
    });

    let response = server.handle_request(malformed);
    assert_eq!(response["success"], false, "Malformed request should fail");

    // Server should still handle valid requests after error
    let valid_req = create_initialize_request();
    let valid_response = server.handle_request(valid_req);
    assert_eq!(valid_response["success"], true, "Server should recover from errors");
}

// RED Phase Test 10: Performance Requirements
#[test]
fn test_dap_server_performance() {
    // Ensure server handles requests within 100ms (per spec)

    use std::time::Instant;

    let server = pmat::services::dap::DapServer::new();
    let req = create_initialize_request();

    let start = Instant::now();
    let _ = server.handle_request(req);
    let duration = start.elapsed();

    assert!(
        duration.as_millis() < 100,
        "Request handling should take less than 100ms, took {:?}",
        duration
    );
}
