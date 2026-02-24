// ComparisonView tests - Variable diff, divergence, render split, export JSON

// ========================================================================
// Variable Diff Tests
// ========================================================================

#[test]
fn test_variable_diff_same_values() {
    let mut vars = HashMap::new();
    vars.insert("x".to_string(), serde_json::json!(42));

    // Need at least 2 frames so recording is not immediately exhausted
    let recording_a = create_recording_with_vars("a", vec![vars.clone(), vars.clone()]);
    let recording_b = create_recording_with_vars("b", vec![vars.clone(), vars]);

    let view = ComparisonView::new(recording_a, recording_b);
    let diff = view.variable_diff();

    assert_eq!(diff.get("x"), Some(&DiffStatus::Same));
}

#[test]
fn test_variable_diff_modified_values() {
    let mut vars_a = HashMap::new();
    vars_a.insert("x".to_string(), serde_json::json!(1));

    let mut vars_b = HashMap::new();
    vars_b.insert("x".to_string(), serde_json::json!(2));

    // Need at least 2 frames so recording is not immediately exhausted
    let recording_a = create_recording_with_vars("a", vec![vars_a.clone(), vars_a]);
    let recording_b = create_recording_with_vars("b", vec![vars_b.clone(), vars_b]);

    let view = ComparisonView::new(recording_a, recording_b);
    let diff = view.variable_diff();

    assert_eq!(diff.get("x"), Some(&DiffStatus::Modified));
}

#[test]
fn test_variable_diff_added_variable() {
    let mut vars_a = HashMap::new();
    vars_a.insert("x".to_string(), serde_json::json!(1));

    let mut vars_b = HashMap::new();
    vars_b.insert("x".to_string(), serde_json::json!(1));
    vars_b.insert("y".to_string(), serde_json::json!(2));

    // Need at least 2 frames so recording is not immediately exhausted
    let recording_a = create_recording_with_vars("a", vec![vars_a.clone(), vars_a]);
    let recording_b = create_recording_with_vars("b", vec![vars_b.clone(), vars_b]);

    let view = ComparisonView::new(recording_a, recording_b);
    let diff = view.variable_diff();

    assert_eq!(diff.get("x"), Some(&DiffStatus::Same));
    assert_eq!(diff.get("y"), Some(&DiffStatus::Added));
}

#[test]
fn test_variable_diff_removed_variable() {
    let mut vars_a = HashMap::new();
    vars_a.insert("x".to_string(), serde_json::json!(1));
    vars_a.insert("y".to_string(), serde_json::json!(2));

    let mut vars_b = HashMap::new();
    vars_b.insert("x".to_string(), serde_json::json!(1));

    // Need at least 2 frames so recording is not immediately exhausted
    let recording_a = create_recording_with_vars("a", vec![vars_a.clone(), vars_a]);
    let recording_b = create_recording_with_vars("b", vec![vars_b.clone(), vars_b]);

    let view = ComparisonView::new(recording_a, recording_b);
    let diff = view.variable_diff();

    assert_eq!(diff.get("x"), Some(&DiffStatus::Same));
    assert_eq!(diff.get("y"), Some(&DiffStatus::Removed));
}

#[test]
fn test_variable_diff_exhausted_a() {
    let vars = HashMap::new();
    let recording_a = Recording::new("a".to_string(), vec![]);
    let recording_b = create_recording_with_vars("b", vec![vars]);

    let view = ComparisonView::new(recording_a, recording_b);
    let diff = view.variable_diff();

    // All variables in B should be marked as Added
    assert!(diff.is_empty() || diff.values().all(|s| *s == DiffStatus::Added));
}

#[test]
fn test_variable_diff_exhausted_b() {
    let mut vars = HashMap::new();
    vars.insert("x".to_string(), serde_json::json!(1));

    // Need at least 2 frames so recording A is not immediately exhausted
    let recording_a = create_recording_with_vars("a", vec![vars.clone(), vars]);
    let recording_b = Recording::new("b".to_string(), vec![]);

    let view = ComparisonView::new(recording_a, recording_b);
    let diff = view.variable_diff();

    assert_eq!(diff.get("x"), Some(&DiffStatus::Removed));
}

#[test]
fn test_variable_diff_both_exhausted() {
    let recording_a = Recording::new("a".to_string(), vec![]);
    let recording_b = Recording::new("b".to_string(), vec![]);

    let view = ComparisonView::new(recording_a, recording_b);
    let diff = view.variable_diff();

    assert!(diff.is_empty());
}

// ========================================================================
// Divergence Point Tests
// ========================================================================

#[test]
fn test_find_divergence_point_no_divergence() {
    let recording_a = create_test_recording("a", 5);
    let recording_b = create_test_recording("b", 5);

    let view = ComparisonView::new(recording_a, recording_b);
    // Recordings have same structure, no divergence
    let divergence = view.find_divergence_point();
    assert!(divergence.is_none());
}

#[test]
fn test_find_divergence_point_variable_difference() {
    let mut vars_0 = HashMap::new();
    vars_0.insert("x".to_string(), serde_json::json!(1));

    let mut vars_1_a = HashMap::new();
    vars_1_a.insert("x".to_string(), serde_json::json!(2));

    let mut vars_1_b = HashMap::new();
    vars_1_b.insert("x".to_string(), serde_json::json!(999)); // Different!

    let recording_a = create_recording_with_vars("a", vec![vars_0.clone(), vars_1_a]);
    let recording_b = create_recording_with_vars("b", vec![vars_0, vars_1_b]);

    let view = ComparisonView::new(recording_a, recording_b);
    let divergence = view.find_divergence_point();

    assert_eq!(divergence, Some(1));
}

#[test]
fn test_find_divergence_point_empty_recordings() {
    let recording_a = Recording::new("a".to_string(), vec![]);
    let recording_b = Recording::new("b".to_string(), vec![]);

    let view = ComparisonView::new(recording_a, recording_b);
    assert!(view.find_divergence_point().is_none());
}

// ========================================================================
// Render Split View Tests
// ========================================================================

#[test]
fn test_render_split_basic() {
    let recording_a = create_test_recording("program_a", 5);
    let recording_b = create_test_recording("program_b", 3);

    let view = ComparisonView::new(recording_a, recording_b);
    let output = view.render_split();

    assert!(output.contains("Recording A: program_a"));
    assert!(output.contains("Recording B: program_b"));
    assert!(output.contains("Frame 0/5"));
    assert!(output.contains("Frame 0/3"));
}

#[test]
fn test_render_split_shows_end_markers() {
    let recording_a = create_test_recording("a", 2);
    let recording_b = create_test_recording("b", 2);

    let mut view = ComparisonView::new(recording_a, recording_b);
    view.next_frame().unwrap(); // Move to last frame

    let output = view.render_split();
    assert!(output.contains("END"));
}

#[test]
fn test_render_split_empty_recordings() {
    let recording_a = Recording::new("empty_a".to_string(), vec![]);
    let recording_b = Recording::new("empty_b".to_string(), vec![]);

    let view = ComparisonView::new(recording_a, recording_b);
    let output = view.render_split();

    assert!(output.contains("Recording A: empty_a"));
    assert!(output.contains("Recording B: empty_b"));
}

// ========================================================================
// Export Diff JSON Tests
// ========================================================================

#[test]
fn test_export_diff_json_structure() {
    let recording_a = create_test_recording("a", 3);
    let recording_b = create_test_recording("b", 3);

    let view = ComparisonView::new(recording_a, recording_b);
    let json = view.export_diff_json().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert!(parsed.get("metadata").is_some());
    assert!(parsed.get("frame_diffs").is_some());
}

#[test]
fn test_export_diff_json_metadata() {
    let recording_a = create_test_recording("prog_a", 5);
    let recording_b = create_test_recording("prog_b", 3);

    let view = ComparisonView::new(recording_a, recording_b);
    let json = view.export_diff_json().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    let metadata = parsed.get("metadata").unwrap();
    assert_eq!(metadata.get("recording_a_name").unwrap(), "prog_a");
    assert_eq!(metadata.get("recording_b_name").unwrap(), "prog_b");
    assert_eq!(metadata.get("recording_a_frames").unwrap(), 5);
    assert_eq!(metadata.get("recording_b_frames").unwrap(), 3);
}

#[test]
fn test_export_diff_json_frame_diffs() {
    let mut vars_a = HashMap::new();
    vars_a.insert("x".to_string(), serde_json::json!(1));

    let mut vars_b = HashMap::new();
    vars_b.insert("x".to_string(), serde_json::json!(2));

    let recording_a = create_recording_with_vars("a", vec![vars_a]);
    let recording_b = create_recording_with_vars("b", vec![vars_b]);

    let view = ComparisonView::new(recording_a, recording_b);
    let json = view.export_diff_json().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    let frame_diffs = parsed.get("frame_diffs").unwrap().as_array().unwrap();
    assert_eq!(frame_diffs.len(), 1);
    assert!(frame_diffs[0].get("variable_diff").is_some());
}

#[test]
fn test_export_diff_json_empty_recordings() {
    let recording_a = Recording::new("a".to_string(), vec![]);
    let recording_b = Recording::new("b".to_string(), vec![]);

    let view = ComparisonView::new(recording_a, recording_b);
    let json = view.export_diff_json().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    let frame_diffs = parsed.get("frame_diffs").unwrap().as_array().unwrap();
    assert!(frame_diffs.is_empty());
}

#[test]
fn test_export_diff_json_asymmetric_frames() {
    let recording_a = create_test_recording("a", 5);
    let recording_b = create_test_recording("b", 2);

    let view = ComparisonView::new(recording_a, recording_b);
    let json = view.export_diff_json().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    let frame_diffs = parsed.get("frame_diffs").unwrap().as_array().unwrap();
    // Should have diffs for max(5, 2) = 5 frames
    assert_eq!(frame_diffs.len(), 5);
}
