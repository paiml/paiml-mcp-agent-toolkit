use pmat::mcp_server::{McpServer, StateManager};
use pmat::models::refactor::RefactorConfig;
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Integration tests for the MCP server functionality
/// These tests verify the complete MCP server workflow including:
/// - State management
/// - Session lifecycle
/// - Message handling
/// - Snapshot persistence

#[tokio::test]
async fn test_state_manager_lifecycle() {
    use tempfile::tempdir;
    let temp_dir = tempdir().unwrap();
    let mut manager = StateManager::with_temp_dir(temp_dir.path());

    // Test initial state
    assert!(
        manager.get_state().is_err(),
        "Should not have state initially"
    );

    // Test session start
    let targets = vec![PathBuf::from("test.rs")];
    let config = RefactorConfig::default();

    manager
        .start_session(targets.clone(), config.clone())
        .unwrap();

    // Verify state exists after start
    let state = manager.get_state().unwrap();
    assert_eq!(state.targets.len(), 1);
    assert_eq!(state.targets[0], PathBuf::from("test.rs"));
    assert_eq!(state.config.target_complexity, config.target_complexity);

    // Test state advancement
    manager.advance().unwrap();
    let state_after_advance = manager.get_state().unwrap();
    // State should still be valid after advance
    assert_eq!(state_after_advance.targets.len(), 1);

    // Test session stop
    manager.stop_session().unwrap();
    assert!(
        manager.get_state().is_err(),
        "Should not have state after stop"
    );
}

#[tokio::test]
async fn test_state_manager_multiple_sessions() {
    use tempfile::tempdir;
    let temp_dir = tempdir().unwrap();
    let mut manager = StateManager::with_temp_dir(temp_dir.path());

    // Start first session
    let targets1 = vec![PathBuf::from("file1.rs")];
    manager
        .start_session(targets1, RefactorConfig::default())
        .unwrap();

    let state1 = manager.get_state().unwrap();
    assert_eq!(state1.targets[0], PathBuf::from("file1.rs"));

    // Stop first session
    manager.stop_session().unwrap();

    // Start second session with different targets
    let targets2 = vec![PathBuf::from("file2.rs"), PathBuf::from("file3.rs")];
    manager
        .start_session(targets2, RefactorConfig::default())
        .unwrap();

    let state2 = manager.get_state().unwrap();
    assert_eq!(state2.targets.len(), 2);
    assert_eq!(state2.targets[0], PathBuf::from("file2.rs"));
    assert_eq!(state2.targets[1], PathBuf::from("file3.rs"));

    manager.stop_session().unwrap();
}

#[tokio::test]
async fn test_state_manager_error_conditions() {
    use tempfile::tempdir;
    let temp_dir = tempdir().unwrap();
    let mut manager = StateManager::with_temp_dir(temp_dir.path());

    // Test advance without session
    assert!(
        manager.advance().is_err(),
        "Should fail to advance without session"
    );

    // Test stop without session
    assert!(
        manager.stop_session().is_err(),
        "Should fail to stop without session"
    );

    // Test starting session twice
    let targets = vec![PathBuf::from("test.rs")];
    manager
        .start_session(targets.clone(), RefactorConfig::default())
        .unwrap();

    let result = manager.start_session(targets, RefactorConfig::default());
    assert!(result.is_err(), "Should fail to start session twice");

    manager.stop_session().unwrap();
}

#[tokio::test]
async fn test_mcp_server_creation() {
    let _server = McpServer::new();
    // Verify server can be created without errors
    // The actual run() method would require stdin/stdout mocking for full testing
}

#[tokio::test]
async fn test_handler_parameter_parsing() {
    use pmat::mcp_server::handlers::{
        handle_refactor_get_state, handle_refactor_start, handle_refactor_stop,
    };

    let state_manager = Arc::new(Mutex::new(StateManager::new()));

    // Test refactor.start with valid parameters
    let start_params = json!({
        "targets": ["src/main.rs", "src/lib.rs"],
        "config": {
            "target_complexity": 10,
            "remove_satd": true,
            "max_function_lines": 50
        }
    });

    let start_result = handle_refactor_start(&state_manager, start_params).await;
    assert!(start_result.is_ok(), "Should handle valid start parameters");

    // Test refactor.getState
    let state_result = handle_refactor_get_state(&state_manager).await;
    assert!(state_result.is_ok(), "Should return current state");

    // Test refactor.stop
    let stop_result = handle_refactor_stop(&state_manager).await;
    assert!(stop_result.is_ok(), "Should stop session successfully");
}

#[tokio::test]
async fn test_handler_invalid_parameters() {
    use pmat::mcp_server::handlers::handle_refactor_start;

    let state_manager = Arc::new(Mutex::new(StateManager::new()));

    // Test with missing targets
    let invalid_params = json!({
        "config": {
            "target_complexity": 10
        }
    });

    let result = handle_refactor_start(&state_manager, invalid_params).await;
    assert!(result.is_err(), "Should fail with missing targets");

    // Test with invalid targets format
    let invalid_targets = json!({
        "targets": "not_an_array"
    });

    let result = handle_refactor_start(&state_manager, invalid_targets).await;
    assert!(result.is_err(), "Should fail with invalid targets format");

    // Test with empty targets
    let empty_targets = json!({
        "targets": []
    });

    let _result = handle_refactor_start(&state_manager, empty_targets).await;
    // This should actually succeed as empty targets might be valid
    // The validation would depend on business logic
}

#[tokio::test]
async fn test_snapshot_persistence() {
    use pmat::mcp_server::snapshots::SnapshotManager;
    use pmat::models::refactor::RefactorStateMachine;
    use tempfile::tempdir;

    let temp_dir = tempdir().unwrap();
    let _snapshot_path = temp_dir.path().join("test-snapshot.bin");

    // Create a custom snapshot manager with test path
    let snapshot_manager = SnapshotManager::with_path(temp_dir.path());

    // Create test state
    let config = RefactorConfig {
        target_complexity: 15,
        remove_satd: true,
        max_function_lines: 100,
        parallel_workers: 4,
        memory_limit_mb: 512,
        batch_size: 10,
        ..Default::default()
    };

    let state = RefactorStateMachine::new(
        vec![PathBuf::from("test1.rs"), PathBuf::from("test2.rs")],
        config,
    );

    // Test save
    snapshot_manager.save_snapshot(&state).unwrap();

    // Test load
    let loaded_state = snapshot_manager.load_snapshot().unwrap();

    // Verify state integrity
    assert_eq!(loaded_state.targets.len(), state.targets.len());
    assert_eq!(loaded_state.targets[0], state.targets[0]);
    assert_eq!(loaded_state.targets[1], state.targets[1]);
    assert_eq!(
        loaded_state.config.target_complexity,
        state.config.target_complexity
    );
    assert_eq!(loaded_state.config.remove_satd, state.config.remove_satd);
    assert_eq!(
        loaded_state.config.max_function_lines,
        state.config.max_function_lines
    );

    // Test remove
    snapshot_manager.remove_snapshot().unwrap();

    // Verify removal
    let load_result = snapshot_manager.load_snapshot();
    assert!(load_result.is_err(), "Should fail to load after removal");
}

#[tokio::test]
async fn test_serialization_format() {
    use pmat::mcp_server::capnp_conversion::{get_serialization_format, is_capnp_available};

    let format = get_serialization_format();
    let is_available = is_capnp_available();

    // Currently should be using JSON fallback
    assert_eq!(format, "JSON");
    assert!(!is_available);
}

#[tokio::test]
async fn test_complete_workflow() {
    // Test a complete MCP server workflow
    let state_manager = Arc::new(Mutex::new(StateManager::new()));

    // 1. Start refactoring session
    use pmat::mcp_server::handlers::handle_refactor_start;
    let start_params = json!({
        "targets": ["src/main.rs"],
        "config": {
            "target_complexity": 20,
            "remove_satd": false
        }
    });

    let start_result = handle_refactor_start(&state_manager, start_params)
        .await
        .unwrap();
    assert!(start_result.get("session_id").is_some());
    assert!(start_result.get("state").is_some());

    // 2. Get current state
    use pmat::mcp_server::handlers::handle_refactor_get_state;
    let state_result = handle_refactor_get_state(&state_manager).await.unwrap();
    assert!(state_result.is_object());

    // 3. Advance iteration
    use pmat::mcp_server::handlers::handle_refactor_next_iteration;
    let next_result = handle_refactor_next_iteration(&state_manager)
        .await
        .unwrap();
    assert!(next_result.is_object());

    // 4. Stop session
    use pmat::mcp_server::handlers::handle_refactor_stop;
    let stop_result = handle_refactor_stop(&state_manager).await.unwrap();
    assert_eq!(
        stop_result["message"],
        "Refactoring session stopped successfully"
    );

    // 5. Verify session is stopped
    let final_state_result = handle_refactor_get_state(&state_manager).await;
    assert!(
        final_state_result.is_err(),
        "Should have no state after stop"
    );
}

/// Test that demonstrates the MCP server handles concurrent requests properly
#[tokio::test]
async fn test_concurrent_access() {
    use tempfile::tempdir;
    let temp_dir = tempdir().unwrap();
    let state_manager = Arc::new(Mutex::new(StateManager::with_temp_dir(temp_dir.path())));

    // Start a session
    let start_params = json!({
        "targets": ["test.rs"],
        "config": {}
    });

    use pmat::mcp_server::handlers::{handle_refactor_get_state, handle_refactor_start};
    handle_refactor_start(&state_manager, start_params)
        .await
        .unwrap();

    // Spawn multiple concurrent state reads
    let mut handles = Vec::new();
    for _ in 0..5 {
        let manager_clone = Arc::clone(&state_manager);
        let handle =
            tokio::spawn(async move { handle_refactor_get_state(&manager_clone).await.unwrap() });
        handles.push(handle);
    }

    // Wait for all to complete
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_object());
    }

    // Clean up
    use pmat::mcp_server::handlers::handle_refactor_stop;
    handle_refactor_stop(&state_manager).await.unwrap();
}
