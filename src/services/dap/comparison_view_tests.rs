// ComparisonView tests - Helpers, enum tests, creation tests, navigation tests

/// Helper: Create test recording with specified snapshots
fn create_test_recording(program: &str, snapshot_count: usize) -> Recording {
    let mut recording = Recording::new(program.to_string(), vec![]);
    for i in 0..snapshot_count {
        let mut variables = HashMap::new();
        variables.insert("counter".to_string(), serde_json::json!(i));
        variables.insert("name".to_string(), serde_json::json!(format!("item_{}", i)));

        let stack_frames = vec![StackFrame {
            name: format!("func_{}", i),
            file: Some("test.rs".to_string()),
            line: Some(10 + i as u32),
            locals: HashMap::new(),
        }];

        let snapshot = Snapshot {
            frame_id: i as u64,
            timestamp_relative_ms: (i * 100) as u32,
            variables,
            stack_frames,
            instruction_pointer: 0x1000 + (i as u64 * 8),
            memory_snapshot: None,
        };
        recording.add_snapshot(snapshot);
    }
    recording
}

/// Helper: Create recording with specific variables at each frame
fn create_recording_with_vars(
    program: &str,
    frames: Vec<HashMap<String, serde_json::Value>>,
) -> Recording {
    let mut recording = Recording::new(program.to_string(), vec![]);
    for (i, variables) in frames.into_iter().enumerate() {
        let snapshot = Snapshot {
            frame_id: i as u64,
            timestamp_relative_ms: (i * 100) as u32,
            variables,
            stack_frames: vec![StackFrame {
                name: "main".to_string(),
                file: Some("test.rs".to_string()),
                line: Some(i as u32),
                locals: HashMap::new(),
            }],
            instruction_pointer: 0x1000 + (i as u64 * 8),
            memory_snapshot: None,
        };
        recording.add_snapshot(snapshot);
    }
    recording
}

// ========================================================================
// SyncMode Tests
// ========================================================================

#[test]
fn test_sync_mode_equality() {
    assert_eq!(SyncMode::ByFrame, SyncMode::ByFrame);
    assert_eq!(SyncMode::ByTimestamp, SyncMode::ByTimestamp);
    assert_eq!(SyncMode::ByLocation, SyncMode::ByLocation);
    assert_ne!(SyncMode::ByFrame, SyncMode::ByTimestamp);
    assert_ne!(SyncMode::ByTimestamp, SyncMode::ByLocation);
}

#[test]
fn test_sync_mode_clone() {
    let mode = SyncMode::ByFrame;
    let cloned = mode;
    assert_eq!(mode, cloned);
}

#[test]
fn test_sync_mode_debug() {
    let mode = SyncMode::ByFrame;
    let debug_str = format!("{:?}", mode);
    assert!(debug_str.contains("ByFrame"));
}

#[test]
fn test_sync_mode_serialization() {
    let mode = SyncMode::ByTimestamp;
    let json = serde_json::to_string(&mode).unwrap();
    let deserialized: SyncMode = serde_json::from_str(&json).unwrap();
    assert_eq!(mode, deserialized);
}

// ========================================================================
// DiffStatus Tests
// ========================================================================

#[test]
fn test_diff_status_variants() {
    assert_eq!(DiffStatus::Same, DiffStatus::Same);
    assert_eq!(DiffStatus::Modified, DiffStatus::Modified);
    assert_eq!(DiffStatus::Added, DiffStatus::Added);
    assert_eq!(DiffStatus::Removed, DiffStatus::Removed);
}

#[test]
fn test_diff_status_inequality() {
    assert_ne!(DiffStatus::Same, DiffStatus::Modified);
    assert_ne!(DiffStatus::Added, DiffStatus::Removed);
}

#[test]
fn test_diff_status_clone() {
    let status = DiffStatus::Modified;
    let cloned = status.clone();
    assert_eq!(status, cloned);
}

#[test]
fn test_diff_status_debug() {
    let status = DiffStatus::Added;
    let debug_str = format!("{:?}", status);
    assert!(debug_str.contains("Added"));
}

#[test]
fn test_diff_status_serialization() {
    let status = DiffStatus::Removed;
    let json = serde_json::to_string(&status).unwrap();
    let deserialized: DiffStatus = serde_json::from_str(&json).unwrap();
    assert_eq!(status, deserialized);
}

// ========================================================================
// ComparisonView Creation Tests
// ========================================================================

#[test]
fn test_comparison_view_creation() {
    let recording_a = create_test_recording("program_a", 5);
    let recording_b = create_test_recording("program_b", 3);

    let view = ComparisonView::new(recording_a, recording_b);

    assert_eq!(view.total_frames_a(), 5);
    assert_eq!(view.total_frames_b(), 3);
    assert_eq!(view.current_frame_a(), 0);
    assert_eq!(view.current_frame_b(), 0);
}

#[test]
fn test_comparison_view_empty_recordings() {
    let recording_a = Recording::new("empty_a".to_string(), vec![]);
    let recording_b = Recording::new("empty_b".to_string(), vec![]);

    let view = ComparisonView::new(recording_a, recording_b);

    assert_eq!(view.total_frames_a(), 0);
    assert_eq!(view.total_frames_b(), 0);
    assert!(view.recording_a_exhausted());
    assert!(view.recording_b_exhausted());
}

#[test]
fn test_comparison_view_default_sync_mode() {
    let recording_a = create_test_recording("a", 2);
    let recording_b = create_test_recording("b", 2);

    let view = ComparisonView::new(recording_a, recording_b);
    assert_eq!(view.sync_mode(), SyncMode::ByFrame);
}

// ========================================================================
// Frame Navigation Tests
// ========================================================================

#[test]
fn test_total_frames_min_max() {
    let recording_a = create_test_recording("a", 10);
    let recording_b = create_test_recording("b", 5);

    let view = ComparisonView::new(recording_a, recording_b);

    assert_eq!(view.total_frames_min(), 5);
    assert_eq!(view.total_frames_max(), 10);
}

#[test]
fn test_next_frame_navigation() {
    let recording_a = create_test_recording("a", 3);
    let recording_b = create_test_recording("b", 3);

    let mut view = ComparisonView::new(recording_a, recording_b);

    assert_eq!(view.current_frame_a(), 0);
    assert_eq!(view.current_frame_b(), 0);

    view.next_frame().unwrap();
    assert_eq!(view.current_frame_a(), 1);
    assert_eq!(view.current_frame_b(), 1);
}

#[test]
fn test_prev_frame_navigation() {
    let recording_a = create_test_recording("a", 5);
    let recording_b = create_test_recording("b", 5);

    let mut view = ComparisonView::new(recording_a, recording_b);

    // Move forward first
    view.next_frame().unwrap();
    view.next_frame().unwrap();
    assert_eq!(view.current_frame_a(), 2);

    // Now move back
    view.prev_frame().unwrap();
    assert_eq!(view.current_frame_a(), 1);
}

#[test]
fn test_prev_frame_at_start_error() {
    let recording_a = create_test_recording("a", 2);
    let recording_b = create_test_recording("b", 2);

    let mut view = ComparisonView::new(recording_a, recording_b);

    let result = view.prev_frame();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("at start"));
}

#[test]
fn test_next_frame_at_end_error() {
    let recording_a = create_test_recording("a", 2);
    let recording_b = create_test_recording("b", 2);

    let mut view = ComparisonView::new(recording_a, recording_b);

    // Move to last frame
    view.next_frame().unwrap();

    // Try to move past end
    let result = view.next_frame();
    assert!(result.is_err());
}

#[test]
fn test_jump_to_valid_frame() {
    let recording_a = create_test_recording("a", 10);
    let recording_b = create_test_recording("b", 10);

    let mut view = ComparisonView::new(recording_a, recording_b);

    view.jump_to(5).unwrap();
    assert_eq!(view.current_frame_a(), 5);
    assert_eq!(view.current_frame_b(), 5);
}

#[test]
fn test_jump_to_out_of_bounds_error() {
    let recording_a = create_test_recording("a", 3);
    let recording_b = create_test_recording("b", 3);

    let mut view = ComparisonView::new(recording_a, recording_b);

    let result = view.jump_to(100);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("out of bounds"));
}

#[test]
fn test_jump_to_asymmetric_recordings() {
    let recording_a = create_test_recording("a", 10);
    let recording_b = create_test_recording("b", 5);

    let mut view = ComparisonView::new(recording_a, recording_b);

    // Jump to frame within B's range
    view.jump_to(3).unwrap();
    assert_eq!(view.current_frame_a(), 3);
    assert_eq!(view.current_frame_b(), 3);

    // Jump to frame outside B's range but within A's range
    view.jump_to(7).unwrap();
    assert_eq!(view.current_frame_a(), 7);
}

// ========================================================================
// Exhaustion Tests
// ========================================================================

#[test]
fn test_recording_exhausted_detection() {
    let recording_a = create_test_recording("a", 2);
    let recording_b = create_test_recording("b", 4);

    let mut view = ComparisonView::new(recording_a, recording_b);

    assert!(!view.recording_a_exhausted());
    assert!(!view.recording_b_exhausted());

    // Move to end of A
    view.next_frame().unwrap();
    assert!(view.recording_a_exhausted());
    assert!(!view.recording_b_exhausted());
}

// ========================================================================
// Sync Mode Tests
// ========================================================================

#[test]
fn test_set_sync_mode() {
    let recording_a = create_test_recording("a", 2);
    let recording_b = create_test_recording("b", 2);

    let mut view = ComparisonView::new(recording_a, recording_b);

    assert_eq!(view.sync_mode(), SyncMode::ByFrame);

    view.set_sync_mode(SyncMode::ByTimestamp);
    assert_eq!(view.sync_mode(), SyncMode::ByTimestamp);

    view.set_sync_mode(SyncMode::ByLocation);
    assert_eq!(view.sync_mode(), SyncMode::ByLocation);
}
