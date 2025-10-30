//! TIMELINE-002: Timeline UI Integration Tests
//! Sprint 77 - RED Phase
//!
//! Tests for integrating TimelinePlayer with Timeline UI for interactive
//! recording playback visualization.
//!
//! ## Requirements (TIMELINE-002)
//!
//! The Timeline UI must:
//! 1. Accept TimelinePlayer as input source
//! 2. Render progress bar showing current frame / total frames
//! 3. Display current snapshot variables in variable panel
//! 4. Display stack trace in stack panel
//! 5. Handle keyboard controls: ← → (prev/next), Space (play/pause), J (jump)
//! 6. Show frame counter: "Frame 42/100 | 1250ms | main.rs:10"
//! 7. Update UI state when navigation occurs
//! 8. Support playback mode with timer-based advancement
//!
//! ## Test Strategy
//!
//! These tests drive the design of Timeline UI + TimelinePlayer integration:
//! - RED Phase: All tests fail with assert!(false)
//! - GREEN Phase: Minimal implementation to pass tests
//! - REFACTOR Phase: Improve design while keeping tests green

/// RED Test 1: Timeline UI accepts TimelinePlayer
///
/// Requirement: TimelineUI must be constructible from a TimelinePlayer
///
/// Expected behavior:
/// - TimelineUI::from_player(player) creates UI
/// - UI has access to player state
/// - Initial state matches player (frame 0, not playing)
#[test]
fn test_timeline_ui_accepts_player() {
    use pmat::services::dap::{TimelinePlayer, TimelineUI};

    let recording = create_test_recording(10);
    let player = TimelinePlayer::new(recording);
    let ui = TimelineUI::from_player(player);

    assert_eq!(ui.current_frame(), 0);
    assert!(!ui.is_playing());
}

/// RED Test 2: Progress bar shows correct position
///
/// Requirement: UI must display progress as "Frame X/Y"
///
/// Expected behavior:
/// - progress_text() returns "Frame 0/10" initially
/// - Updates after navigation: "Frame 5/10"
/// - Shows percentage or visual bar
#[test]
fn test_progress_bar_display() {
    use pmat::services::dap::{TimelinePlayer, TimelineUI};

    let recording = create_test_recording(100);
    let player = TimelinePlayer::new(recording);
    let mut ui = TimelineUI::from_player(player);

    assert_eq!(ui.progress_text(), "Frame 0/100");

    ui.next_frame().unwrap();
    assert_eq!(ui.progress_text(), "Frame 1/100");

    ui.jump_to(50).unwrap();
    assert_eq!(ui.progress_text(), "Frame 50/100");
}

/// RED Test 3: Variables panel shows current snapshot
///
/// Requirement: UI must display variables from current snapshot
///
/// Expected behavior:
/// - variables() returns HashMap from current snapshot
/// - Updates when frame changes
/// - Empty when no variables in snapshot
#[test]
fn test_variables_panel_display() {
    use pmat::services::dap::{TimelinePlayer, TimelineUI};

    let recording = create_test_recording(10);
    let player = TimelinePlayer::new(recording);
    let mut ui = TimelineUI::from_player(player);

    // Frame 0 variables
    let vars = ui.current_variables();
    assert!(vars.contains_key("test_var"));
    assert_eq!(vars["test_var"], serde_json::json!(0));

    // Navigate to frame 5
    ui.jump_to(5).unwrap();
    let vars = ui.current_variables();
    assert_eq!(vars["test_var"], serde_json::json!(5));
}

/// RED Test 4: Stack panel shows current stack frames
///
/// Requirement: UI must display stack trace from current snapshot
///
/// Expected behavior:
/// - stack_frames() returns Vec<StackFrame> from current snapshot
/// - Shows function names, files, line numbers
/// - Updates when frame changes
#[test]
fn test_stack_panel_display() {
    // This test drives the requirement for stack trace display
    // Will implement in GREEN phase:
    //
    // use pmat::services::dap::{TimelinePlayer, Recording};
    // use pmat::services::dap::timeline_ui::TimelineUI;
    //
    // let recording = create_test_recording(10);
    // let player = TimelinePlayer::new(recording);
    // let mut ui = TimelineUI::from_player(player);
    //
    // let stack = ui.current_stack_frames();
    // assert_eq!(stack.len(), 1);
    // assert_eq!(stack[0].name, "test_function_0");
    // assert_eq!(stack[0].file, Some("test.rs".to_string()));
    //
    // ui.next_frame();
    // let stack = ui.current_stack_frames();
    // assert_eq!(stack[0].name, "test_function_1");

    use pmat::services::dap::{TimelinePlayer, TimelineUI};

    let recording = create_test_recording(10);
    let player = TimelinePlayer::new(recording);
    let mut ui = TimelineUI::from_player(player);

    let stack = ui.current_stack_frames();
    assert_eq!(stack.len(), 1);
    assert_eq!(stack[0].name, "test_function_0");
    assert_eq!(stack[0].file, Some("test.rs".to_string()));

    ui.next_frame().unwrap();
    let stack = ui.current_stack_frames();
    assert_eq!(stack[0].name, "test_function_1");
}

/// RED Test 5: Right arrow (→) advances frame
///
/// Requirement: Pressing → must advance to next frame
///
/// Expected behavior:
/// - handle_key('→') advances current_frame by 1
/// - Returns Ok(()) on success
/// - Returns Err at end of recording
#[test]
fn test_right_arrow_advances() {
    use pmat::services::dap::{TimelinePlayer, TimelineUI};

    let recording = create_test_recording(10);
    let player = TimelinePlayer::new(recording);
    let mut ui = TimelineUI::from_player(player);

    assert_eq!(ui.current_frame(), 0);

    ui.handle_key('→').unwrap();
    assert_eq!(ui.current_frame(), 1);

    ui.handle_key('→').unwrap();
    assert_eq!(ui.current_frame(), 2);
}

/// RED Test 6: Left arrow (←) moves back
///
/// Requirement: Pressing ← must move to previous frame
///
/// Expected behavior:
/// - handle_key('←') decrements current_frame by 1
/// - Returns Ok(()) on success
/// - Returns Err at start of recording
#[test]
fn test_left_arrow_moves_back() {
    use pmat::services::dap::{TimelinePlayer, TimelineUI};

    let recording = create_test_recording(10);
    let player = TimelinePlayer::new(recording);
    let mut ui = TimelineUI::from_player(player);

    // Move forward then back
    ui.jump_to(5).unwrap();
    assert_eq!(ui.current_frame(), 5);

    ui.handle_key('←').unwrap();
    assert_eq!(ui.current_frame(), 4);

    ui.handle_key('←').unwrap();
    assert_eq!(ui.current_frame(), 3);
}

/// RED Test 7: Space toggles play/pause
///
/// Requirement: Pressing Space must toggle playback state
///
/// Expected behavior:
/// - handle_key(' ') toggles is_playing
/// - First press: starts playback
/// - Second press: pauses playback
#[test]
fn test_space_toggles_playback() {
    use pmat::services::dap::{TimelinePlayer, TimelineUI};

    let recording = create_test_recording(10);
    let player = TimelinePlayer::new(recording);
    let mut ui = TimelineUI::from_player(player);

    assert!(!ui.is_playing());

    ui.handle_key(' ').unwrap();
    assert!(ui.is_playing());

    ui.handle_key(' ').unwrap();
    assert!(!ui.is_playing());
}

/// RED Test 8: J prompts for frame number and jumps
///
/// Requirement: Pressing J must allow jumping to specific frame
///
/// Expected behavior:
/// - handle_key('j') enters jump mode
/// - User can input frame number
/// - Jump to specified frame
#[test]
fn test_jump_key_handling() {
    use pmat::services::dap::{TimelinePlayer, TimelineUI};

    let recording = create_test_recording(100);
    let player = TimelinePlayer::new(recording);
    let mut ui = TimelineUI::from_player(player);

    // Direct jump method (keyboard input handled by UI layer)
    ui.jump_to(50).unwrap();
    assert_eq!(ui.current_frame(), 50);

    ui.jump_to(0).unwrap();
    assert_eq!(ui.current_frame(), 0);

    // Out of bounds fails
    assert!(ui.jump_to(200).is_err());
}

/// RED Test 9: Frame counter updates on navigation
///
/// Requirement: Frame counter must show "Frame X/Y | Timestamp | Location"
///
/// Expected behavior:
/// - frame_info() returns formatted string
/// - Includes frame number, timestamp, file:line
/// - Updates when frame changes
#[test]
fn test_frame_counter_display() {
    use pmat::services::dap::{TimelinePlayer, TimelineUI};

    let recording = create_test_recording(10);
    let player = TimelinePlayer::new(recording);
    let mut ui = TimelineUI::from_player(player);

    let info = ui.frame_info();
    assert!(info.contains("Frame 0/10"));
    assert!(info.contains("0ms")); // timestamp_relative_ms
    assert!(info.contains("test.rs:10")); // file:line

    ui.next_frame().unwrap();
    let info = ui.frame_info();
    assert!(info.contains("Frame 1/10"));
    assert!(info.contains("100ms"));
    assert!(info.contains("test.rs:11"));
}

/// RED Test 10: UI updates during playback mode
///
/// Requirement: When playing, UI must auto-advance frames
///
/// Expected behavior:
/// - tick() method advances frame when playing
/// - Respects playback_speed
/// - Stops at end of recording
#[test]
fn test_playback_auto_advance() {
    use pmat::services::dap::{TimelinePlayer, TimelineUI};

    let recording = create_test_recording(10);
    let player = TimelinePlayer::new(recording);
    let mut ui = TimelineUI::from_player(player);

    // Start playback
    ui.play();
    assert!(ui.is_playing());
    assert_eq!(ui.current_frame(), 0);

    // Tick advances frame when playing
    ui.tick();
    assert_eq!(ui.current_frame(), 1);

    ui.tick();
    assert_eq!(ui.current_frame(), 2);

    // Pause stops advancement
    ui.pause();
    ui.tick();
    assert_eq!(ui.current_frame(), 2); // Unchanged
}

// ============================================================================
// Test Helpers
// ============================================================================

/// Helper: Create test recording with N snapshots
#[allow(dead_code)]
fn create_test_recording(snapshot_count: usize) -> pmat::services::dap::recording::Recording {
    use pmat::services::dap::recording::{Recording, Snapshot, StackFrame};
    use std::collections::HashMap;

    let mut recording = Recording::new("test_program".to_string(), vec!["--test".to_string()]);

    for i in 0..snapshot_count {
        let mut variables = HashMap::new();
        variables.insert("test_var".to_string(), serde_json::json!(i));
        variables.insert("counter".to_string(), serde_json::json!(i * 10));

        let stack_frames = vec![StackFrame {
            name: format!("test_function_{}", i),
            file: Some("test.rs".to_string()),
            line: Some(10 + i as u32),
            locals: HashMap::new(),
        }];

        let snapshot = Snapshot {
            frame_id: i as u64,
            timestamp_relative_ms: (i * 100) as u32,
            variables,
            stack_frames,
            instruction_pointer: 0x401000 + (i as u64 * 0x10),
            memory_snapshot: None,
        };

        recording.add_snapshot(snapshot);
    }

    recording
}
