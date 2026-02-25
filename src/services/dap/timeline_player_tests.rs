// Tests for TimelinePlayer

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::dap::recording::StackFrame;
    use std::collections::HashMap;

    /// Helper: Create test recording with N snapshots
    fn create_test_recording(snapshot_count: usize) -> Recording {
        let mut recording = Recording::new("test_program".to_string(), vec!["--test".to_string()]);

        for i in 0..snapshot_count {
            let mut variables = HashMap::new();
            variables.insert("test_var".to_string(), serde_json::json!(i));

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

    #[test]
    fn test_new_initializes_at_frame_zero() {
        let recording = create_test_recording(10);
        let player = TimelinePlayer::new(recording);

        assert_eq!(player.current_frame(), 0);
        assert_eq!(player.total_frames(), 10);
        assert_eq!(player.playback_speed(), 1.0);
        assert!(!player.is_playing());
    }

    #[test]
    fn test_next_frame_navigation() {
        let recording = create_test_recording(5);
        let mut player = TimelinePlayer::new(recording);

        assert_eq!(player.current_frame(), 0);

        let snapshot = player.next_frame().unwrap();
        assert_eq!(snapshot.frame_id, 1);
        assert_eq!(player.current_frame(), 1);

        let snapshot = player.next_frame().unwrap();
        assert_eq!(snapshot.frame_id, 2);
        assert_eq!(player.current_frame(), 2);
    }

    #[test]
    fn test_prev_frame_navigation() {
        let recording = create_test_recording(5);
        let mut player = TimelinePlayer::new(recording);

        // Move to frame 2
        player.next_frame();
        player.next_frame();
        assert_eq!(player.current_frame(), 2);

        // Move back
        let snapshot = player.prev_frame().unwrap();
        assert_eq!(snapshot.frame_id, 1);
        assert_eq!(player.current_frame(), 1);

        let snapshot = player.prev_frame().unwrap();
        assert_eq!(snapshot.frame_id, 0);
        assert_eq!(player.current_frame(), 0);

        // At start, returns None
        assert!(player.prev_frame().is_none());
    }

    #[test]
    fn test_jump_to_valid_frame() {
        let recording = create_test_recording(100);
        let mut player = TimelinePlayer::new(recording);

        player.jump_to(50).unwrap();
        assert_eq!(player.current_frame(), 50);

        player.jump_to(0).unwrap();
        assert_eq!(player.current_frame(), 0);

        player.jump_to(99).unwrap();
        assert_eq!(player.current_frame(), 99);
    }

    #[test]
    fn test_jump_to_out_of_bounds() {
        let recording = create_test_recording(10);
        let mut player = TimelinePlayer::new(recording);

        let result = player.jump_to(10);
        assert!(result.is_err());
        assert_eq!(player.current_frame(), 0); // Unchanged

        let result = player.jump_to(100);
        assert!(result.is_err());
        assert_eq!(player.current_frame(), 0); // Unchanged
    }

    #[test]
    fn test_play_pause_control() {
        let recording = create_test_recording(10);
        let mut player = TimelinePlayer::new(recording);

        assert!(!player.is_playing());

        player.play();
        assert!(player.is_playing());

        player.pause();
        assert!(!player.is_playing());

        // Can toggle multiple times
        player.play();
        assert!(player.is_playing());
        player.pause();
        assert!(!player.is_playing());
    }

    #[test]
    fn test_playback_speed() {
        let recording = create_test_recording(10);
        let mut player = TimelinePlayer::new(recording);

        assert_eq!(player.playback_speed(), 1.0);

        player.set_speed(0.5);
        assert_eq!(player.playback_speed(), 0.5);

        player.set_speed(2.0);
        assert_eq!(player.playback_speed(), 2.0);

        player.set_speed(10.0);
        assert_eq!(player.playback_speed(), 10.0);
    }

    #[test]
    fn test_current_snapshot() {
        let recording = create_test_recording(10);
        let mut player = TimelinePlayer::new(recording);

        let snapshot = player.current_snapshot();
        assert_eq!(snapshot.frame_id, 0);

        player.next_frame();
        let snapshot = player.current_snapshot();
        assert_eq!(snapshot.frame_id, 1);

        player.jump_to(5).unwrap();
        let snapshot = player.current_snapshot();
        assert_eq!(snapshot.frame_id, 5);
    }
}
