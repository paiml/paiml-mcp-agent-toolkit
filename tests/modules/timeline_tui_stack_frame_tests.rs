#![cfg(feature = "dap")]

// Sprint 78: TUI-004 RED phase - Stack Frame Navigator Tests
//
// Tests for interactive stack frame navigation in timeline TUI.
// These tests verify:
// - Stack frame list management
// - Current frame selection state
// - Navigation (up/down through frames)
// - Frame information display
// - Selection highlighting
// - Bounds enforcement

use pmat::services::dap::timeline_tui::StackFrameNavigator;

// ============================================================================
// Test 1: StackFrameNavigator Creation
// ============================================================================

#[test]
fn test_stack_frame_navigator_creation() {
    // RED: Should create empty stack frame navigator
    let navigator = StackFrameNavigator::new();

    assert_eq!(navigator.frame_count(), 0);
    assert_eq!(navigator.selected_index(), 0);
}

#[test]
fn test_stack_frame_navigator_with_frames() {
    // RED: Should create navigator with stack frames
    let frames = vec![
        ("main".to_string(), "main.rs".to_string(), 10),
        ("process".to_string(), "lib.rs".to_string(), 42),
        ("helper".to_string(), "utils.rs".to_string(), 100),
    ];

    let navigator = StackFrameNavigator::from_frames(frames);

    assert_eq!(navigator.frame_count(), 3);
}

// ============================================================================
// Test 2: Frame Selection Management
// ============================================================================

#[test]
fn test_set_selected_index() {
    // RED: Should update selected frame index
    let frames = vec![
        ("main".to_string(), "main.rs".to_string(), 10),
        ("process".to_string(), "lib.rs".to_string(), 42),
    ];
    let mut navigator = StackFrameNavigator::from_frames(frames);

    navigator.set_selected_index(1);

    assert_eq!(navigator.selected_index(), 1);
}

#[test]
fn test_selection_bounds_enforcement() {
    // RED: Should enforce selection bounds (0 to frame_count-1)
    let frames = vec![("main".to_string(), "main.rs".to_string(), 10)];
    let mut navigator = StackFrameNavigator::from_frames(frames);

    navigator.set_selected_index(10); // Out of bounds

    // Should clamp to valid range
    assert_eq!(navigator.selected_index(), 0);
}

// ============================================================================
// Test 3: Frame Navigation
// ============================================================================

#[test]
fn test_select_next_frame() {
    // RED: Should move selection down one frame
    let frames = vec![
        ("main".to_string(), "main.rs".to_string(), 10),
        ("process".to_string(), "lib.rs".to_string(), 42),
        ("helper".to_string(), "utils.rs".to_string(), 100),
    ];
    let mut navigator = StackFrameNavigator::from_frames(frames);

    navigator.select_next();
    assert_eq!(navigator.selected_index(), 1);

    navigator.select_next();
    assert_eq!(navigator.selected_index(), 2);
}

#[test]
fn test_select_previous_frame() {
    // RED: Should move selection up one frame
    let frames = vec![
        ("main".to_string(), "main.rs".to_string(), 10),
        ("process".to_string(), "lib.rs".to_string(), 42),
        ("helper".to_string(), "utils.rs".to_string(), 100),
    ];
    let mut navigator = StackFrameNavigator::from_frames(frames);
    navigator.set_selected_index(2);

    navigator.select_previous();

    assert_eq!(navigator.selected_index(), 1);
}

#[test]
fn test_select_next_at_bottom() {
    // RED: Should not move past bottom frame
    let frames = vec![
        ("main".to_string(), "main.rs".to_string(), 10),
        ("process".to_string(), "lib.rs".to_string(), 42),
    ];
    let mut navigator = StackFrameNavigator::from_frames(frames);
    navigator.set_selected_index(1);

    navigator.select_next();

    assert_eq!(navigator.selected_index(), 1);
}

#[test]
fn test_select_previous_at_top() {
    // RED: Should not move past top frame
    let frames = vec![("main".to_string(), "main.rs".to_string(), 10)];
    let mut navigator = StackFrameNavigator::from_frames(frames);

    navigator.select_previous();

    assert_eq!(navigator.selected_index(), 0);
}

// ============================================================================
// Test 4: Frame Information Access
// ============================================================================

#[test]
fn test_get_frame_at_index() {
    // RED: Should retrieve frame by index
    let frames = vec![
        ("main".to_string(), "main.rs".to_string(), 10),
        ("process".to_string(), "lib.rs".to_string(), 42),
    ];
    let navigator = StackFrameNavigator::from_frames(frames);

    let frame = navigator.get_frame(1);

    assert!(frame.is_some());
    let (func, file, line) = frame.unwrap();
    assert_eq!(func, "process");
    assert_eq!(file, "lib.rs");
    assert_eq!(line, 42);
}

#[test]
fn test_get_frame_out_of_bounds() {
    // RED: Should return None for invalid index
    let frames = vec![("main".to_string(), "main.rs".to_string(), 10)];
    let navigator = StackFrameNavigator::from_frames(frames);

    let frame = navigator.get_frame(10);

    assert_eq!(frame, None);
}

#[test]
fn test_get_selected_frame() {
    // RED: Should retrieve currently selected frame
    let frames = vec![
        ("main".to_string(), "main.rs".to_string(), 10),
        ("process".to_string(), "lib.rs".to_string(), 42),
    ];
    let mut navigator = StackFrameNavigator::from_frames(frames);
    navigator.set_selected_index(1);

    let frame = navigator.get_selected_frame();

    assert!(frame.is_some());
    let (func, file, line) = frame.unwrap();
    assert_eq!(func, "process");
    assert_eq!(file, "lib.rs");
    assert_eq!(line, 42);
}

// ============================================================================
// Test 5: Frame Formatting
// ============================================================================

#[test]
fn test_format_frame_line() {
    // RED: Should format frame as "function @ file:line"
    let frames = vec![("main".to_string(), "main.rs".to_string(), 10)];
    let navigator = StackFrameNavigator::from_frames(frames);

    let line = navigator.format_frame_line(0);

    assert_eq!(line, Some("main @ main.rs:10".to_string()));
}

#[test]
fn test_format_frame_line_out_of_bounds() {
    // RED: Should return None for invalid index
    let frames = vec![("main".to_string(), "main.rs".to_string(), 10)];
    let navigator = StackFrameNavigator::from_frames(frames);

    let line = navigator.format_frame_line(10);

    assert_eq!(line, None);
}

#[test]
fn test_format_frame_with_selection_marker() {
    // RED: Should add selection marker (▶) to selected frame
    let frames = vec![
        ("main".to_string(), "main.rs".to_string(), 10),
        ("process".to_string(), "lib.rs".to_string(), 42),
    ];
    let mut navigator = StackFrameNavigator::from_frames(frames);
    navigator.set_selected_index(1);

    let line = navigator.format_frame_with_marker(1);

    assert_eq!(line, Some("▶ process @ lib.rs:42".to_string()));
}

#[test]
fn test_format_frame_without_selection_marker() {
    // RED: Should add spacing (  ) to non-selected frames
    let frames = vec![
        ("main".to_string(), "main.rs".to_string(), 10),
        ("process".to_string(), "lib.rs".to_string(), 42),
    ];
    let mut navigator = StackFrameNavigator::from_frames(frames);
    navigator.set_selected_index(1);

    let line = navigator.format_frame_with_marker(0);

    assert_eq!(line, Some("  main @ main.rs:10".to_string()));
}

// ============================================================================
// Test 6: All Frames Rendering
// ============================================================================

#[test]
fn test_render_all_frames() {
    // RED: Should format all frames with selection markers
    let frames = vec![
        ("main".to_string(), "main.rs".to_string(), 10),
        ("process".to_string(), "lib.rs".to_string(), 42),
        ("helper".to_string(), "utils.rs".to_string(), 100),
    ];
    let mut navigator = StackFrameNavigator::from_frames(frames);
    navigator.set_selected_index(1);

    let rendered = navigator.render_all_frames();

    assert_eq!(rendered.len(), 3);
    assert_eq!(rendered[0], "  main @ main.rs:10");
    assert_eq!(rendered[1], "▶ process @ lib.rs:42");
    assert_eq!(rendered[2], "  helper @ utils.rs:100");
}

// ============================================================================
// Test 7: Frame Addition
// ============================================================================

#[test]
fn test_add_frame() {
    // RED: Should add frame to navigator
    let mut navigator = StackFrameNavigator::new();

    navigator.add_frame("main".to_string(), "main.rs".to_string(), 10);

    assert_eq!(navigator.frame_count(), 1);
}

#[test]
fn test_add_multiple_frames() {
    // RED: Should maintain frame order
    let mut navigator = StackFrameNavigator::new();

    navigator.add_frame("main".to_string(), "main.rs".to_string(), 10);
    navigator.add_frame("process".to_string(), "lib.rs".to_string(), 42);

    assert_eq!(navigator.frame_count(), 2);
    let frame = navigator.get_frame(1);
    assert!(frame.is_some());
    let (func, _, _) = frame.unwrap();
    assert_eq!(func, "process");
}

// ============================================================================
// Test 8: Empty State
// ============================================================================

#[test]
fn test_empty_navigator_selection() {
    // RED: Should handle selection with no frames
    let mut navigator = StackFrameNavigator::new();

    navigator.select_next();
    navigator.select_previous();

    assert_eq!(navigator.selected_index(), 0);
}

#[test]
fn test_empty_navigator_get_selected_frame() {
    // RED: Should return None when no frames exist
    let navigator = StackFrameNavigator::new();

    let frame = navigator.get_selected_frame();

    assert_eq!(frame, None);
}

// ============================================================================
// Test 9: Selection State Query
// ============================================================================

#[test]
fn test_is_frame_selected() {
    // RED: Should check if frame is selected
    let frames = vec![
        ("main".to_string(), "main.rs".to_string(), 10),
        ("process".to_string(), "lib.rs".to_string(), 42),
    ];
    let mut navigator = StackFrameNavigator::from_frames(frames);
    navigator.set_selected_index(1);

    assert!(!navigator.is_frame_selected(0));
    assert!(navigator.is_frame_selected(1));
}
