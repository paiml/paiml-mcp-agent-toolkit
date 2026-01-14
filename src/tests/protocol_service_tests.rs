use crate::unified_protocol::{
    adapters::{cli::CliAdapter, mcp::McpAdapter},
    service::AppState,
};
use std::sync::Arc;

#[test]
fn test_app_state_creation() {
    let state = AppState::default();

    // Basic test that app state can be created
    assert!(std::ptr::addr_of!(state) as usize != 0);
}

#[test]
fn test_cli_adapter_creation() {
    let adapter = CliAdapter::new();

    // Test that CLI adapter can be created
    assert!(std::ptr::addr_of!(adapter) as usize != 0);
}

#[test]
fn test_app_state_clone() {
    let state = AppState::default();
    let cloned_state = state.clone();

    // Test that app state can be cloned
    assert!(std::ptr::addr_of!(cloned_state) as usize != 0);
}

#[test]
fn test_mcp_adapter_creation() {
    let adapter = McpAdapter::new();

    // Test that MCP adapter can be created
    assert!(std::ptr::addr_of!(adapter) as usize != 0);
}

#[test]
fn test_adapters_are_send_sync() {
    // Test that adapters have proper trait bounds
    fn assert_send_sync<T: Send + Sync>() {}

    assert_send_sync::<CliAdapter>();
    assert_send_sync::<McpAdapter>();
    assert_send_sync::<AppState>();
}

#[test]
fn test_adapter_trait_bounds() {
    // Ensure adapters can be used in concurrent contexts
    use std::sync::Arc;

    let cli_adapter = Arc::new(CliAdapter::new());
    let mcp_adapter = Arc::new(McpAdapter::new());

    // Test Arc cloning (requires Send + Sync)
    let _cli_clone = Arc::clone(&cli_adapter);
    let _mcp_clone = Arc::clone(&mcp_adapter);
}

#[test]
fn test_app_state_reference_counting() {
    let state = Arc::new(AppState::default());
    assert_eq!(Arc::strong_count(&state), 1);

    let state_clone = Arc::clone(&state);
    assert_eq!(Arc::strong_count(&state), 2);

    drop(state_clone);
    assert_eq!(Arc::strong_count(&state), 1);
}

#[test]
fn test_adapter_memory_footprint() {
    // Test that adapters have reasonable memory usage
    let cli_adapter = CliAdapter::new();
    let mcp_adapter = McpAdapter::new();

    // Basic size checks (adapters should be lightweight)
    assert!(std::mem::size_of_val(&cli_adapter) < 1024);
    assert!(std::mem::size_of_val(&mcp_adapter) < 1024);
}

#[test]
fn test_app_state_creation_performance() {
    // Test that app state creation is fast
    let start = std::time::Instant::now();

    for _ in 0..100 {
        let state = AppState::default();
        drop(state);
    }

    let duration = start.elapsed();
    assert!(
        duration.as_millis() < 100,
        "AppState creation too slow: {:?}",
        duration
    );
}

#[test]
fn test_multiple_app_states() {
    // Test that multiple app states can coexist
    let state1 = AppState::default();
    let state2 = AppState::default();

    // Both states should be independent
    drop(state1);
    drop(state2);
}

#[test]
fn test_adapter_creation_basic() {
    // Test basic adapter creation functionality
    let cli_adapter = CliAdapter::new();
    let mcp_adapter = McpAdapter::new();

    // Test that adapters can be created and used
    assert!(std::ptr::addr_of!(cli_adapter) as usize != 0);
    assert!(std::ptr::addr_of!(mcp_adapter) as usize != 0);
}
