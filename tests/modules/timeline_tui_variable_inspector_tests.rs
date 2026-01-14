#![cfg(feature = "dap")]

// Sprint 78: TUI-003 RED phase - Variable Inspector View Tests
//
// Tests for scrollable variable inspector in timeline TUI.
// These tests verify:
// - Variable list management
// - Scroll state (offset, viewport)
// - Selection and highlighting
// - Navigation (up/down, page up/down)
// - Viewport bounds enforcement

use pmat::services::dap::timeline_tui::VariableInspectorView;

// ============================================================================
// Test 1: VariableInspectorView Creation
// ============================================================================

#[test]
fn test_variable_inspector_creation() {
    // RED: Should create empty variable inspector
    let inspector = VariableInspectorView::new();

    assert_eq!(inspector.variable_count(), 0);
    assert_eq!(inspector.scroll_offset(), 0);
}

#[test]
fn test_variable_inspector_with_variables() {
    // RED: Should create inspector with variable list
    let variables = vec![
        ("x".to_string(), "42".to_string()),
        ("y".to_string(), "hello".to_string()),
        ("z".to_string(), "[1, 2, 3]".to_string()),
    ];

    let inspector = VariableInspectorView::from_variables(variables);

    assert_eq!(inspector.variable_count(), 3);
}

// ============================================================================
// Test 2: Scroll Offset Management
// ============================================================================

#[test]
fn test_set_scroll_offset() {
    // RED: Should update scroll offset
    let mut inspector = VariableInspectorView::new();
    inspector.add_variable("x".to_string(), "42".to_string());
    inspector.add_variable("y".to_string(), "100".to_string());

    inspector.set_scroll_offset(1);

    assert_eq!(inspector.scroll_offset(), 1);
}

#[test]
fn test_scroll_offset_bounds() {
    // RED: Should enforce scroll bounds (0 to variable_count-1)
    let mut inspector = VariableInspectorView::new();
    inspector.add_variable("x".to_string(), "42".to_string());

    inspector.set_scroll_offset(10); // Out of bounds

    // Should clamp to valid range
    assert_eq!(inspector.scroll_offset(), 0);
}

// ============================================================================
// Test 3: Variable Navigation
// ============================================================================

#[test]
fn test_scroll_down() {
    // RED: Should scroll down one line
    let mut inspector = VariableInspectorView::from_variables(vec![
        ("a".to_string(), "1".to_string()),
        ("b".to_string(), "2".to_string()),
        ("c".to_string(), "3".to_string()),
    ]);

    inspector.scroll_down();
    assert_eq!(inspector.scroll_offset(), 1);

    inspector.scroll_down();
    assert_eq!(inspector.scroll_offset(), 2);
}

#[test]
fn test_scroll_up() {
    // RED: Should scroll up one line
    let mut inspector = VariableInspectorView::from_variables(vec![
        ("a".to_string(), "1".to_string()),
        ("b".to_string(), "2".to_string()),
        ("c".to_string(), "3".to_string()),
    ]);
    inspector.set_scroll_offset(2);

    inspector.scroll_up();

    assert_eq!(inspector.scroll_offset(), 1);
}

#[test]
fn test_scroll_up_at_top() {
    // RED: Should not scroll past top
    let mut inspector =
        VariableInspectorView::from_variables(vec![("a".to_string(), "1".to_string())]);

    inspector.scroll_up();

    assert_eq!(inspector.scroll_offset(), 0);
}

#[test]
fn test_scroll_down_at_bottom() {
    // RED: Should not scroll past bottom
    let mut inspector = VariableInspectorView::from_variables(vec![
        ("a".to_string(), "1".to_string()),
        ("b".to_string(), "2".to_string()),
    ]);
    inspector.set_scroll_offset(1);

    inspector.scroll_down();

    assert_eq!(inspector.scroll_offset(), 1);
}

// ============================================================================
// Test 4: Page Navigation
// ============================================================================

#[test]
fn test_page_down() {
    // RED: Should scroll down by viewport height
    let mut inspector = VariableInspectorView::from_variables(vec![
        ("a".to_string(), "1".to_string()),
        ("b".to_string(), "2".to_string()),
        ("c".to_string(), "3".to_string()),
        ("d".to_string(), "4".to_string()),
        ("e".to_string(), "5".to_string()),
    ]);
    inspector.set_viewport_height(2);

    inspector.page_down();

    assert_eq!(inspector.scroll_offset(), 2);
}

#[test]
fn test_page_up() {
    // RED: Should scroll up by viewport height
    let mut inspector = VariableInspectorView::from_variables(vec![
        ("a".to_string(), "1".to_string()),
        ("b".to_string(), "2".to_string()),
        ("c".to_string(), "3".to_string()),
        ("d".to_string(), "4".to_string()),
    ]);
    inspector.set_viewport_height(2);
    inspector.set_scroll_offset(3);

    inspector.page_up();

    assert_eq!(inspector.scroll_offset(), 1);
}

// ============================================================================
// Test 5: Viewport Management
// ============================================================================

#[test]
fn test_viewport_height() {
    // RED: Should get/set viewport height
    let mut inspector = VariableInspectorView::new();

    inspector.set_viewport_height(10);

    assert_eq!(inspector.viewport_height(), 10);
}

#[test]
fn test_visible_range() {
    // RED: Should calculate visible variable range
    let mut inspector = VariableInspectorView::from_variables(vec![
        ("a".to_string(), "1".to_string()),
        ("b".to_string(), "2".to_string()),
        ("c".to_string(), "3".to_string()),
        ("d".to_string(), "4".to_string()),
        ("e".to_string(), "5".to_string()),
    ]);
    inspector.set_viewport_height(3);
    inspector.set_scroll_offset(1);

    let range = inspector.visible_range();

    assert_eq!(range, (1, 4)); // offset=1, height=3 -> indices 1,2,3
}

// ============================================================================
// Test 6: Variable Access
// ============================================================================

#[test]
fn test_get_variable_at_index() {
    // RED: Should retrieve variable by index
    let inspector = VariableInspectorView::from_variables(vec![
        ("x".to_string(), "42".to_string()),
        ("y".to_string(), "100".to_string()),
    ]);

    let var = inspector.get_variable(1);

    assert_eq!(var, Some((&"y".to_string(), &"100".to_string())));
}

#[test]
fn test_get_variable_out_of_bounds() {
    // RED: Should return None for invalid index
    let inspector =
        VariableInspectorView::from_variables(vec![("x".to_string(), "42".to_string())]);

    let var = inspector.get_variable(10);

    assert_eq!(var, None);
}

// ============================================================================
// Test 7: Variable Formatting
// ============================================================================

#[test]
fn test_format_variable_line() {
    // RED: Should format variable as "name: value"
    let inspector =
        VariableInspectorView::from_variables(vec![("counter".to_string(), "42".to_string())]);

    let line = inspector.format_line(0);

    assert_eq!(line, Some("counter: 42".to_string()));
}

#[test]
fn test_format_visible_lines() {
    // RED: Should format all visible lines
    let mut inspector = VariableInspectorView::from_variables(vec![
        ("a".to_string(), "1".to_string()),
        ("b".to_string(), "2".to_string()),
        ("c".to_string(), "3".to_string()),
    ]);
    inspector.set_viewport_height(2);
    inspector.set_scroll_offset(1);

    let lines = inspector.visible_lines();

    assert_eq!(lines, vec!["b: 2", "c: 3"]);
}

// ============================================================================
// Test 8: Empty State
// ============================================================================

#[test]
fn test_empty_inspector_scroll() {
    // RED: Should handle scrolling with no variables
    let mut inspector = VariableInspectorView::new();

    inspector.scroll_down();
    inspector.scroll_up();

    assert_eq!(inspector.scroll_offset(), 0);
}
