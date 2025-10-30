// Sprint 78: TUI-005 RED phase - Keyboard Shortcut Handler Tests
//
// Tests for keyboard shortcut system in timeline TUI.
// These tests verify:
// - Action enum for all TUI actions
// - Keyboard event routing to actions
// - Action handler registration
// - Integration between EventLoop and handlers
// - All key bindings work correctly

use pmat::services::dap::timeline_tui::{
    KeyboardHandler, TuiAction, KeyCode, TerminalEvent,
};

// ============================================================================
// Test 1: TuiAction Enum
// ============================================================================

#[test]
fn test_tui_action_enum_variants() {
    // RED: Should have action variants for all TUI operations
    let actions = vec![
        TuiAction::NextFrame,
        TuiAction::PreviousFrame,
        TuiAction::TogglePlayback,
        TuiAction::JumpToStart,
        TuiAction::JumpToEnd,
        TuiAction::Quit,
        TuiAction::ScrollDown,
        TuiAction::ScrollUp,
        TuiAction::SelectNextFrame,
        TuiAction::SelectPreviousFrame,
    ];

    assert_eq!(actions.len(), 10);
}

// ============================================================================
// Test 2: KeyboardHandler Creation
// ============================================================================

#[test]
fn test_keyboard_handler_creation() {
    // RED: Should create keyboard handler with default bindings
    let handler = KeyboardHandler::new();

    assert!(handler.has_default_bindings());
}

#[test]
fn test_keyboard_handler_with_custom_bindings() {
    // RED: Should support custom key bindings
    let mut handler = KeyboardHandler::new();
    handler.bind_key(KeyCode::Char('n'), TuiAction::NextFrame);

    assert!(handler.is_key_bound(KeyCode::Char('n')));
}

// ============================================================================
// Test 3: Key-to-Action Mapping
// ============================================================================

#[test]
fn test_map_right_arrow_to_next_frame() {
    // RED: Right arrow should map to NextFrame
    let handler = KeyboardHandler::new();

    let action = handler.get_action(KeyCode::Right);

    assert_eq!(action, Some(TuiAction::NextFrame));
}

#[test]
fn test_map_left_arrow_to_previous_frame() {
    // RED: Left arrow should map to PreviousFrame
    let handler = KeyboardHandler::new();

    let action = handler.get_action(KeyCode::Left);

    assert_eq!(action, Some(TuiAction::PreviousFrame));
}

#[test]
fn test_map_space_to_toggle_playback() {
    // RED: Space should map to TogglePlayback
    let handler = KeyboardHandler::new();

    let action = handler.get_action(KeyCode::Char(' '));

    assert_eq!(action, Some(TuiAction::TogglePlayback));
}

#[test]
fn test_map_q_to_quit() {
    // RED: 'q' should map to Quit
    let handler = KeyboardHandler::new();

    let action = handler.get_action(KeyCode::Char('q'));

    assert_eq!(action, Some(TuiAction::Quit));
}

#[test]
fn test_map_home_to_jump_start() {
    // RED: Home should map to JumpToStart
    let handler = KeyboardHandler::new();

    let action = handler.get_action(KeyCode::Home);

    assert_eq!(action, Some(TuiAction::JumpToStart));
}

#[test]
fn test_map_end_to_jump_end() {
    // RED: End should map to JumpToEnd
    let handler = KeyboardHandler::new();

    let action = handler.get_action(KeyCode::End);

    assert_eq!(action, Some(TuiAction::JumpToEnd));
}

// ============================================================================
// Test 4: Unmapped Keys
// ============================================================================

#[test]
fn test_unmapped_key_returns_none() {
    // RED: Unmapped keys should return None
    let handler = KeyboardHandler::new();

    let action = handler.get_action(KeyCode::Char('x'));

    assert_eq!(action, None);
}

// ============================================================================
// Test 5: Custom Key Binding
// ============================================================================

#[test]
fn test_bind_custom_key() {
    // RED: Should allow binding custom keys
    let mut handler = KeyboardHandler::new();

    handler.bind_key(KeyCode::Char('n'), TuiAction::NextFrame);
    let action = handler.get_action(KeyCode::Char('n'));

    assert_eq!(action, Some(TuiAction::NextFrame));
}

#[test]
fn test_rebind_existing_key() {
    // RED: Should allow rebinding existing keys
    let mut handler = KeyboardHandler::new();

    handler.bind_key(KeyCode::Right, TuiAction::Quit);
    let action = handler.get_action(KeyCode::Right);

    assert_eq!(action, Some(TuiAction::Quit));
}

#[test]
fn test_unbind_key() {
    // RED: Should allow unbinding keys
    let mut handler = KeyboardHandler::new();

    handler.unbind_key(KeyCode::Char('q'));
    let action = handler.get_action(KeyCode::Char('q'));

    assert_eq!(action, None);
}

// ============================================================================
// Test 6: Event-to-Action Conversion
// ============================================================================

#[test]
fn test_convert_keyboard_event_to_action() {
    // RED: Should convert TerminalEvent to TuiAction
    let handler = KeyboardHandler::new();
    let event = TerminalEvent::Key(KeyCode::Right);

    let action = handler.handle_event(&event);

    assert_eq!(action, Some(TuiAction::NextFrame));
}

#[test]
fn test_ignore_non_keyboard_events() {
    // RED: Should return None for non-keyboard events
    let handler = KeyboardHandler::new();
    let event = TerminalEvent::Resize(80, 24);

    let action = handler.handle_event(&event);

    assert_eq!(action, None);
}

// ============================================================================
// Test 7: Action Description
// ============================================================================

#[test]
fn test_action_description() {
    // RED: Actions should have human-readable descriptions
    assert_eq!(TuiAction::NextFrame.description(), "Next frame");
    assert_eq!(TuiAction::PreviousFrame.description(), "Previous frame");
    assert_eq!(TuiAction::TogglePlayback.description(), "Play/Pause");
    assert_eq!(TuiAction::Quit.description(), "Quit");
}

// ============================================================================
// Test 8: Key Bindings List
// ============================================================================

#[test]
fn test_list_all_bindings() {
    // RED: Should list all current key bindings
    let handler = KeyboardHandler::new();

    let bindings = handler.list_bindings();

    assert!(bindings.len() >= 6); // At least 6 default bindings
}

#[test]
fn test_binding_format() {
    // RED: Bindings should be formatted as (key, action) pairs
    let handler = KeyboardHandler::new();

    let bindings = handler.list_bindings();
    let has_right_arrow = bindings
        .iter()
        .any(|(k, a)| *k == KeyCode::Right && *a == TuiAction::NextFrame);

    assert!(has_right_arrow);
}

// ============================================================================
// Test 9: Help Text Generation
// ============================================================================

#[test]
fn test_generate_help_text() {
    // RED: Should generate help text for all bindings
    let handler = KeyboardHandler::new();

    let help = handler.generate_help_text();

    assert!(help.contains("→")); // Right arrow
    assert!(help.contains("←")); // Left arrow
    assert!(help.contains("Space")); // Space bar
    assert!(help.contains("q")); // Quit key
}

// ============================================================================
// Test 10: Multiple Actions Per Key (Not Supported)
// ============================================================================

#[test]
fn test_rebinding_replaces_action() {
    // RED: Binding same key twice should replace first binding
    let mut handler = KeyboardHandler::new();

    handler.bind_key(KeyCode::Char('x'), TuiAction::NextFrame);
    handler.bind_key(KeyCode::Char('x'), TuiAction::PreviousFrame);

    let action = handler.get_action(KeyCode::Char('x'));
    assert_eq!(action, Some(TuiAction::PreviousFrame));
}

// ============================================================================
// Test 11: Case Sensitivity
// ============================================================================

#[test]
fn test_key_bindings_case_sensitive() {
    // RED: 'Q' and 'q' should be different keys
    let mut handler = KeyboardHandler::new();

    handler.bind_key(KeyCode::Char('Q'), TuiAction::NextFrame);

    let lower_action = handler.get_action(KeyCode::Char('q'));
    let upper_action = handler.get_action(KeyCode::Char('Q'));

    assert_ne!(lower_action, upper_action);
}

// ============================================================================
// Test 12: Default Bindings Completeness
// ============================================================================

#[test]
fn test_default_bindings_include_navigation() {
    // RED: Default bindings should include all navigation keys
    let handler = KeyboardHandler::new();

    assert!(handler.is_key_bound(KeyCode::Right));
    assert!(handler.is_key_bound(KeyCode::Left));
    assert!(handler.is_key_bound(KeyCode::Home));
    assert!(handler.is_key_bound(KeyCode::End));
}

#[test]
fn test_default_bindings_include_playback() {
    // RED: Default bindings should include playback control
    let handler = KeyboardHandler::new();

    assert!(handler.is_key_bound(KeyCode::Char(' ')));
}

#[test]
fn test_default_bindings_include_quit() {
    // RED: Default bindings should include quit
    let handler = KeyboardHandler::new();

    assert!(handler.is_key_bound(KeyCode::Char('q')));
}
