#![cfg(feature = "dap")]

// DEBUG-002: DAP Server CLI Handler Tests
// Sprint 74 - RED Phase
//
// Tests for `pmat debug serve` command implementation
// These tests drive the creation of async DAP server infrastructure

use pmat::services::dap::DapServer;
use tokio::net::TcpListener;
use tokio::time::{timeout, Duration};

// RED Test 1: Handler function exists and reports itself unimplemented
//
// This used to spawn `handle_debug_serve` and comment "Server should be running
// now" — it never was; the handler bailed immediately and the task's Err was
// discarded, so the test passed on a handler that binds no socket at all. The
// handler now exits the process with DEBUG_UNIMPLEMENTED_EXIT_CODE, so awaiting
// it here would take the test binary down. Pin the signature and the honest
// diagnostic instead.
#[test]
fn test_debug_serve_handler_exists() {
    // Building the future type-checks the signature without running it.
    let _future = pmat::cli::handlers::debug_handlers::handle_debug_serve(
        15678,
        "127.0.0.1".to_string(),
        None,
    );

    let mut buf = Vec::new();
    pmat::cli::handlers::debug_handlers::write_debug_unimplemented_message(
        &mut buf,
        "serve",
        "DEBUG-002",
        "host=127.0.0.1 port=15678 record_dir=None",
    )
    .expect("write");
    let s = String::from_utf8(buf).expect("utf8");
    assert!(s.contains("DEBUG-002"), "got: {s}");
    assert!(s.contains("not implemented"), "got: {s}");
    assert_eq!(
        pmat::cli::handlers::debug_handlers::DEBUG_UNIMPLEMENTED_EXIT_CODE,
        2,
        "unimplemented must exit 2 (misuse), not 1"
    );
}

// RED Test 2: DAP server can start on specified port
#[tokio::test]
async fn test_dap_server_starts_on_port() {
    // This test drives the creation of DapServer::run() method
    // The server should bind to a port and be ready to accept connections

    let server = DapServer::new();
    let port = 15679;

    // Spawn server in background task
    let server_handle =
        tokio::spawn(async move { server.run(port, "127.0.0.1".to_string()).await });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Verify port is listening by attempting to connect
    let connect_result = timeout(
        Duration::from_millis(500),
        TcpListener::bind(format!("127.0.0.1:{}", port)),
    )
    .await;

    // Port should be in use (connection attempt fails with "already in use")
    assert!(
        connect_result.is_err() || connect_result.unwrap().is_err(),
        "Server should be listening on port {}",
        port
    );

    // Clean up
    server_handle.abort();
}

// RED Test 3: Server returns error when port is already in use
#[tokio::test]
async fn test_server_handles_port_in_use() {
    // This test ensures proper error handling for port conflicts

    let port = 15680;

    // Bind the port manually to simulate "already in use"
    let _listener = TcpListener::bind(format!("127.0.0.1:{}", port))
        .await
        .expect("Failed to bind test port");

    // Try to start DAP server on same port
    let server = DapServer::new();
    let result = server.run(port, "127.0.0.1".to_string()).await;

    // Should return an error (not panic)
    assert!(result.is_err(), "Should fail when port is already in use");

    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("address already in use")
            || error_msg.contains("in use")
            || error_msg.contains("bind"),
        "Error should indicate port/address conflict: {}",
        error_msg
    );
}

// RED Test 4: Handler can be stopped gracefully
#[tokio::test]
async fn test_server_graceful_shutdown() {
    // This test ensures the server can be stopped cleanly
    // Important for CLI use case where user hits Ctrl+C

    let server = DapServer::new();
    let port = 15681;

    // Start server
    let server_handle =
        tokio::spawn(async move { server.run(port, "127.0.0.1".to_string()).await });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Stop server by aborting the task
    server_handle.abort();

    // Wait a bit for cleanup
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Verify port is released by successfully binding to it
    let bind_result = TcpListener::bind(format!("127.0.0.1:{}", port)).await;

    assert!(
        bind_result.is_ok(),
        "Port should be released after server shutdown"
    );
}
