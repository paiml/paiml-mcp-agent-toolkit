// TRACE-009: Variable Diff Highlighting Tests
// Sprint 73 - RED Phase
//
// Integration tests for visual comparison of variables between snapshots.
// Tests drive implementation of diff computation and colored visualization.

use pmat::services::dap::types::{ExecutionSnapshot, SourceLocation, StackFrame};
use std::collections::HashMap;

// Helper function to create a snapshot with specific variables
fn create_snapshot_with_vars(
    sequence: usize,
    vars: HashMap<String, serde_json::Value>,
) -> ExecutionSnapshot {
    ExecutionSnapshot {
        timestamp: 1000000 + (sequence as u64 * 1000),
        sequence,
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
    }
}

// RED Test 1: Detect changed variables
#[test]
fn test_detect_changed_variables() {
    let mut vars1 = HashMap::new();
    vars1.insert("x".to_string(), serde_json::json!(10));
    vars1.insert("y".to_string(), serde_json::json!(20));
    let snapshot1 = create_snapshot_with_vars(0, vars1);

    let mut vars2 = HashMap::new();
    vars2.insert("x".to_string(), serde_json::json!(15));
    vars2.insert("y".to_string(), serde_json::json!(20));
    let snapshot2 = create_snapshot_with_vars(1, vars2);

    let diff = pmat::services::dap::VariableDiff::compute(&snapshot1, &snapshot2);

    // Variable "x" changed from 10 to 15
    assert_eq!(diff.changed.len(), 1, "Should detect 1 changed variable");
    assert!(
        diff.changed.contains_key("x"),
        "Changed variables should include 'x'"
    );
    let x_change = &diff.changed["x"];
    assert_eq!(x_change.old_value, serde_json::json!(10));
    assert_eq!(x_change.new_value, serde_json::json!(15));

    // Variable "y" unchanged (should not appear in diff)
    assert_eq!(diff.unchanged.len(), 1, "Should detect 1 unchanged variable");
    assert!(
        diff.unchanged.iter().any(|s| s == "y"),
        "Unchanged variables should include 'y'"
    );
}

// RED Test 2: Detect new variables
#[test]
fn test_detect_new_variables() {
    let mut vars1 = HashMap::new();
    vars1.insert("x".to_string(), serde_json::json!(10));
    let snapshot1 = create_snapshot_with_vars(0, vars1);

    let mut vars2 = HashMap::new();
    vars2.insert("x".to_string(), serde_json::json!(10));
    vars2.insert("z".to_string(), serde_json::json!(30));
    let snapshot2 = create_snapshot_with_vars(1, vars2);

    let diff = pmat::services::dap::VariableDiff::compute(&snapshot1, &snapshot2);

    // New variable "z" added
    assert_eq!(diff.added.len(), 1, "Should detect 1 new variable");
    assert!(diff.added.contains_key("z"), "New variables should include 'z'");
    assert_eq!(diff.added["z"], serde_json::json!(30));
}

// RED Test 3: Detect removed variables
#[test]
fn test_detect_removed_variables() {
    let mut vars1 = HashMap::new();
    vars1.insert("x".to_string(), serde_json::json!(10));
    vars1.insert("y".to_string(), serde_json::json!(20));
    let snapshot1 = create_snapshot_with_vars(0, vars1);

    let mut vars2 = HashMap::new();
    vars2.insert("x".to_string(), serde_json::json!(10));
    let snapshot2 = create_snapshot_with_vars(1, vars2);

    let diff = pmat::services::dap::VariableDiff::compute(&snapshot1, &snapshot2);

    // Variable "y" removed
    assert_eq!(diff.removed.len(), 1, "Should detect 1 removed variable");
    assert!(
        diff.removed.contains_key("y"),
        "Removed variables should include 'y'"
    );
    assert_eq!(diff.removed["y"], serde_json::json!(20));
}

// RED Test 4: Render diff as colored text
#[test]
fn test_render_diff_colored() {
    let mut vars1 = HashMap::new();
    vars1.insert("x".to_string(), serde_json::json!(10));
    vars1.insert("y".to_string(), serde_json::json!(20));
    let snapshot1 = create_snapshot_with_vars(0, vars1);

    let mut vars2 = HashMap::new();
    vars2.insert("x".to_string(), serde_json::json!(15));
    vars2.insert("z".to_string(), serde_json::json!(30));
    let snapshot2 = create_snapshot_with_vars(1, vars2);

    let diff = pmat::services::dap::VariableDiff::compute(&snapshot1, &snapshot2);
    let output = diff.render_colored();

    // Should contain ANSI color codes
    assert!(
        output.contains("\x1b["),
        "Colored output should contain ANSI escape codes"
    );

    // Should show changed variables
    assert!(output.contains("x"), "Output should show changed variable 'x'");

    // Should show added variables
    assert!(output.contains("z"), "Output should show added variable 'z'");

    // Should show removed variables
    assert!(output.contains("y"), "Output should show removed variable 'y'");
}

// RED Test 5: Side-by-side diff display
#[test]
fn test_side_by_side_diff() {
    let mut vars1 = HashMap::new();
    vars1.insert("x".to_string(), serde_json::json!(10));
    vars1.insert("y".to_string(), serde_json::json!(20));
    let snapshot1 = create_snapshot_with_vars(0, vars1);

    let mut vars2 = HashMap::new();
    vars2.insert("x".to_string(), serde_json::json!(15));
    vars2.insert("z".to_string(), serde_json::json!(30));
    let snapshot2 = create_snapshot_with_vars(1, vars2);

    let diff = pmat::services::dap::VariableDiff::compute(&snapshot1, &snapshot2);
    let output = diff.render_side_by_side();

    // Should show both snapshot labels
    assert!(
        output.contains("Snapshot #0") || output.contains("Before"),
        "Output should label first snapshot"
    );
    assert!(
        output.contains("Snapshot #1") || output.contains("After"),
        "Output should label second snapshot"
    );

    // Should show old and new values
    assert!(output.contains("10"), "Output should show old value of x");
    assert!(output.contains("15"), "Output should show new value of x");
}

// RED Test 6: Deep object diff (nested structures)
#[test]
fn test_deep_object_diff() {
    let mut vars1 = HashMap::new();
    vars1.insert(
        "user".to_string(),
        serde_json::json!({
            "name": "Alice",
            "age": 30,
            "address": {
                "city": "Boston"
            }
        }),
    );
    let snapshot1 = create_snapshot_with_vars(0, vars1);

    let mut vars2 = HashMap::new();
    vars2.insert(
        "user".to_string(),
        serde_json::json!({
            "name": "Alice",
            "age": 31,
            "address": {
                "city": "Cambridge"
            }
        }),
    );
    let snapshot2 = create_snapshot_with_vars(1, vars2);

    let diff = pmat::services::dap::VariableDiff::compute(&snapshot1, &snapshot2);

    // Should detect nested changes
    assert!(
        diff.changed.contains_key("user"),
        "Should detect changes in nested object"
    );

    let output = diff.render_colored();
    // Should show nested field differences
    assert!(
        output.contains("age") || output.contains("31") || output.contains("30"),
        "Output should show nested changes"
    );
}

// RED Test 7: Array diff visualization
#[test]
fn test_array_diff_visualization() {
    let mut vars1 = HashMap::new();
    vars1.insert("items".to_string(), serde_json::json!([1, 2, 3]));
    let snapshot1 = create_snapshot_with_vars(0, vars1);

    let mut vars2 = HashMap::new();
    vars2.insert("items".to_string(), serde_json::json!([1, 2, 3, 4]));
    let snapshot2 = create_snapshot_with_vars(1, vars2);

    let diff = pmat::services::dap::VariableDiff::compute(&snapshot1, &snapshot2);

    // Should detect array changes
    assert!(
        diff.changed.contains_key("items"),
        "Should detect changes in array"
    );

    let output = diff.render_colored();
    // Should show array modification
    assert!(
        output.contains("items") || output.contains("4"),
        "Output should show array changes"
    );
}

// RED Test 8: Type change detection
#[test]
fn test_type_change_detection() {
    let mut vars1 = HashMap::new();
    vars1.insert("value".to_string(), serde_json::json!(42));
    let snapshot1 = create_snapshot_with_vars(0, vars1);

    let mut vars2 = HashMap::new();
    vars2.insert("value".to_string(), serde_json::json!("42"));
    let snapshot2 = create_snapshot_with_vars(1, vars2);

    let diff = pmat::services::dap::VariableDiff::compute(&snapshot1, &snapshot2);

    // Should detect type change (number -> string)
    assert!(
        diff.changed.contains_key("value"),
        "Should detect value type change"
    );

    let value_change = &diff.changed["value"];
    assert_eq!(value_change.old_value, serde_json::json!(42));
    assert_eq!(value_change.new_value, serde_json::json!("42"));

    // Should flag as type change
    assert!(
        value_change.type_changed,
        "Should flag when variable type changes"
    );
}

// RED Test 9: Diff statistics summary
#[test]
fn test_diff_statistics_summary() {
    let mut vars1 = HashMap::new();
    vars1.insert("a".to_string(), serde_json::json!(1));
    vars1.insert("b".to_string(), serde_json::json!(2));
    vars1.insert("c".to_string(), serde_json::json!(3));
    let snapshot1 = create_snapshot_with_vars(0, vars1);

    let mut vars2 = HashMap::new();
    vars2.insert("a".to_string(), serde_json::json!(10));
    vars2.insert("b".to_string(), serde_json::json!(2));
    vars2.insert("d".to_string(), serde_json::json!(4));
    let snapshot2 = create_snapshot_with_vars(1, vars2);

    let diff = pmat::services::dap::VariableDiff::compute(&snapshot1, &snapshot2);
    let stats = diff.get_statistics();

    // Should provide summary statistics
    assert_eq!(stats.changed_count, 1, "Should count changed variables");
    assert_eq!(stats.added_count, 1, "Should count added variables");
    assert_eq!(stats.removed_count, 1, "Should count removed variables");
    assert_eq!(stats.unchanged_count, 1, "Should count unchanged variables");
    assert_eq!(stats.total_variables_before, 3, "Should count total before");
    assert_eq!(stats.total_variables_after, 3, "Should count total after");
}

// RED Test 10: Export diff to JSON
#[test]
fn test_export_diff_to_json() {
    let mut vars1 = HashMap::new();
    vars1.insert("x".to_string(), serde_json::json!(10));
    let snapshot1 = create_snapshot_with_vars(0, vars1);

    let mut vars2 = HashMap::new();
    vars2.insert("x".to_string(), serde_json::json!(15));
    let snapshot2 = create_snapshot_with_vars(1, vars2);

    let diff = pmat::services::dap::VariableDiff::compute(&snapshot1, &snapshot2);
    let json_output = diff.to_json();

    // Should parse as valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&json_output)
        .expect("Diff JSON output should be valid JSON");

    // Should contain diff sections
    assert!(
        parsed["changed"].is_object() || parsed["changed"].is_array(),
        "JSON should contain 'changed' section"
    );
    assert!(
        parsed["added"].is_object() || parsed["added"].is_array(),
        "JSON should contain 'added' section"
    );
    assert!(
        parsed["removed"].is_object() || parsed["removed"].is_array(),
        "JSON should contain 'removed' section"
    );
}
