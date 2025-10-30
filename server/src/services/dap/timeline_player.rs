//! TIMELINE-001: TimelinePlayer State Management
//! Sprint 77 - GREEN Phase
//!
//! TimelinePlayer manages recording playback state and provides navigation controls
//! for the Timeline UI. It wraps a Recording and tracks the current frame position,
//! playback state, and speed.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │ TimelinePlayer                                               │
//! │                                                               │
//! │  ┌─────────────┐         ┌──────────────┐                   │
//! │  │  Recording  │────────▶│  Snapshots   │                   │
//! │  │             │         │  [0..N]      │                   │
//! │  └─────────────┘         └──────────────┘                   │
//! │                                  ▲                           │
//! │                                  │                           │
//! │                          ┌───────┴────────┐                 │
//! │                          │ current_frame  │                 │
//! │                          │   (position)   │                 │
//! │                          └────────────────┘                 │
//! │                                                               │
//! │  Navigation Methods:                                         │
//! │  • next_frame() ──────▶ Advance forward                     │
//! │  • prev_frame() ──────▶ Move backward                       │
//! │  • jump_to(N)   ──────▶ Random access                       │
//! │                                                               │
//! │  Playback Control:                                           │
//! │  • play()       ──────▶ Enable auto-advance                 │
//! │  • pause()      ──────▶ Disable auto-advance                │
//! │  • set_speed()  ──────▶ Adjust playback rate                │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```rust,no_run
//! use pmat::services::dap::{Recording, TimelinePlayer};
//! use std::path::PathBuf;
//!
//! // Load recording from file
//! let recording = Recording::load_from_file(&PathBuf::from("session.pmat"))?;
//!
//! // Create player
//! let mut player = TimelinePlayer::new(recording);
//!
//! // Navigate through frames
//! player.next_frame();        // Advance to frame 1
//! player.prev_frame();        // Back to frame 0
//! player.jump_to(50)?;        // Jump to frame 50
//!
//! // Playback control
//! player.play();              // Start auto-advance mode
//! player.set_speed(2.0);      // 2x speed
//! player.pause();             // Stop auto-advance
//!
//! // Access current state
//! let snapshot = player.current_snapshot();
//! let position = player.current_frame();
//! let total = player.total_frames();
//! # Ok::<(), anyhow::Error>(())
//! ```

use super::recording::{Recording, Snapshot};
use anyhow::Result;

/// TimelinePlayer manages recording playback state and navigation
///
/// Wraps a Recording and provides:
/// - Frame position tracking (current_frame)
/// - Navigation controls (next, prev, jump)
/// - Playback state (play, pause, speed)
/// - Current snapshot access for UI rendering
#[derive(Debug)]
pub struct TimelinePlayer {
    /// The recording being played back
    recording: Recording,

    /// Current frame position (0-indexed)
    current_frame: usize,

    /// Total number of frames in recording
    total_frames: usize,

    /// Playback speed multiplier (1.0 = normal speed)
    playback_speed: f64,

    /// Whether playback is currently active
    is_playing: bool,
}

impl TimelinePlayer {
    /// Create a new TimelinePlayer from a Recording
    ///
    /// Initializes the player at frame 0 with default playback settings:
    /// - Speed: 1.0x (normal speed)
    /// - Playing: false (paused)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use pmat::services::dap::{Recording, TimelinePlayer};
    /// use std::path::PathBuf;
    ///
    /// let recording = Recording::load_from_file(&PathBuf::from("session.pmat"))?;
    /// let player = TimelinePlayer::new(recording);
    ///
    /// assert_eq!(player.current_frame(), 0);
    /// assert_eq!(player.playback_speed(), 1.0);
    /// assert!(!player.is_playing());
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn new(recording: Recording) -> Self {
        let total_frames = recording.snapshot_count();

        Self {
            recording,
            current_frame: 0,
            total_frames,
            playback_speed: 1.0,
            is_playing: false,
        }
    }

    /// Get the current frame position (0-indexed)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use pmat::services::dap::{Recording, TimelinePlayer};
    /// # let recording = Recording::new("test".to_string(), vec![]);
    /// let mut player = TimelinePlayer::new(recording);
    /// assert_eq!(player.current_frame(), 0);
    ///
    /// player.next_frame();
    /// assert_eq!(player.current_frame(), 1);
    /// ```
    pub fn current_frame(&self) -> usize {
        self.current_frame
    }

    /// Get the total number of frames in the recording
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use pmat::services::dap::{Recording, TimelinePlayer};
    /// # let mut recording = Recording::new("test".to_string(), vec![]);
    /// # for _ in 0..10 { recording.add_snapshot(Default::default()); }
    /// let player = TimelinePlayer::new(recording);
    /// assert_eq!(player.total_frames(), 10);
    /// ```
    pub fn total_frames(&self) -> usize {
        self.total_frames
    }

    /// Advance to the next frame
    ///
    /// Returns Some(&Snapshot) if there is a next frame, or None if already at the end.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use pmat::services::dap::{Recording, TimelinePlayer};
    /// # let mut recording = Recording::new("test".to_string(), vec![]);
    /// # for _ in 0..3 { recording.add_snapshot(Default::default()); }
    /// let mut player = TimelinePlayer::new(recording);
    ///
    /// let frame1 = player.next_frame().unwrap();  // Move to frame 1
    /// let frame2 = player.next_frame().unwrap();  // Move to frame 2
    /// let at_end = player.next_frame();           // None (already at end)
    /// assert!(at_end.is_none());
    /// ```
    pub fn next_frame(&mut self) -> Option<&Snapshot> {
        if self.current_frame < self.total_frames - 1 {
            self.current_frame += 1;
            Some(&self.recording.snapshots()[self.current_frame])
        } else {
            None
        }
    }

    /// Move back to the previous frame
    ///
    /// Returns Some(&Snapshot) if there is a previous frame, or None if already at frame 0.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use pmat::services::dap::{Recording, TimelinePlayer};
    /// # let mut recording = Recording::new("test".to_string(), vec![]);
    /// # for _ in 0..3 { recording.add_snapshot(Default::default()); }
    /// let mut player = TimelinePlayer::new(recording);
    ///
    /// player.jump_to(2).unwrap();               // Start at frame 2
    /// let frame1 = player.prev_frame().unwrap();  // Move to frame 1
    /// let frame0 = player.prev_frame().unwrap();  // Move to frame 0
    /// let at_start = player.prev_frame();         // None (already at start)
    /// assert!(at_start.is_none());
    /// ```
    pub fn prev_frame(&mut self) -> Option<&Snapshot> {
        if self.current_frame > 0 {
            self.current_frame -= 1;
            Some(&self.recording.snapshots()[self.current_frame])
        } else {
            None
        }
    }

    /// Jump to a specific frame by index
    ///
    /// Returns Ok(&Snapshot) if the frame index is valid, or an error if out of bounds.
    ///
    /// # Errors
    ///
    /// Returns an error if `frame >= total_frames`.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use pmat::services::dap::{Recording, TimelinePlayer};
    /// # let mut recording = Recording::new("test".to_string(), vec![]);
    /// # for _ in 0..100 { recording.add_snapshot(Default::default()); }
    /// let mut player = TimelinePlayer::new(recording);
    ///
    /// player.jump_to(50)?;   // Jump to middle
    /// assert_eq!(player.current_frame(), 50);
    ///
    /// player.jump_to(0)?;    // Jump to start
    /// assert_eq!(player.current_frame(), 0);
    ///
    /// let result = player.jump_to(1000);
    /// assert!(result.is_err());  // Out of bounds
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn jump_to(&mut self, frame: usize) -> Result<&Snapshot> {
        if frame < self.total_frames {
            self.current_frame = frame;
            Ok(&self.recording.snapshots()[frame])
        } else {
            anyhow::bail!(
                "Frame {} out of bounds (total: {})",
                frame,
                self.total_frames
            )
        }
    }

    /// Get a reference to the current snapshot
    ///
    /// Returns a reference to the Snapshot at the current frame position.
    /// This is the primary method for UI rendering.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use pmat::services::dap::{Recording, TimelinePlayer};
    /// # let mut recording = Recording::new("test".to_string(), vec![]);
    /// # for _ in 0..10 { recording.add_snapshot(Default::default()); }
    /// let mut player = TimelinePlayer::new(recording);
    ///
    /// let snapshot = player.current_snapshot();
    /// // Access snapshot data for UI rendering
    /// println!("Variables: {:?}", snapshot.variables);
    /// println!("Stack: {:?}", snapshot.stack_frames);
    /// ```
    pub fn current_snapshot(&self) -> &Snapshot {
        &self.recording.snapshots()[self.current_frame]
    }

    /// Start auto-advance playback
    ///
    /// Sets `is_playing` to true. The UI layer is responsible for implementing
    /// timer-based frame advancement at the current playback speed.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use pmat::services::dap::{Recording, TimelinePlayer};
    /// # let recording = Recording::new("test".to_string(), vec![]);
    /// let mut player = TimelinePlayer::new(recording);
    ///
    /// assert!(!player.is_playing());
    /// player.play();
    /// assert!(player.is_playing());
    /// ```
    pub fn play(&mut self) {
        self.is_playing = true;
    }

    /// Stop auto-advance playback
    ///
    /// Sets `is_playing` to false, pausing frame advancement.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use pmat::services::dap::{Recording, TimelinePlayer};
    /// # let recording = Recording::new("test".to_string(), vec![]);
    /// let mut player = TimelinePlayer::new(recording);
    ///
    /// player.play();
    /// assert!(player.is_playing());
    ///
    /// player.pause();
    /// assert!(!player.is_playing());
    /// ```
    pub fn pause(&mut self) {
        self.is_playing = false;
    }

    /// Check if playback is currently active
    ///
    /// Returns true if play() has been called without a subsequent pause().
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use pmat::services::dap::{Recording, TimelinePlayer};
    /// # let recording = Recording::new("test".to_string(), vec![]);
    /// let mut player = TimelinePlayer::new(recording);
    ///
    /// assert!(!player.is_playing());
    /// player.play();
    /// assert!(player.is_playing());
    /// ```
    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

    /// Set the playback speed multiplier
    ///
    /// The speed affects how fast frames advance during auto-play mode:
    /// - 0.5 = half speed (2x slower)
    /// - 1.0 = normal speed
    /// - 2.0 = double speed (2x faster)
    ///
    /// The UI layer implements the timing logic using this value.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use pmat::services::dap::{Recording, TimelinePlayer};
    /// # let recording = Recording::new("test".to_string(), vec![]);
    /// let mut player = TimelinePlayer::new(recording);
    ///
    /// assert_eq!(player.playback_speed(), 1.0);
    ///
    /// player.set_speed(2.0);
    /// assert_eq!(player.playback_speed(), 2.0);
    ///
    /// player.set_speed(0.5);
    /// assert_eq!(player.playback_speed(), 0.5);
    /// ```
    pub fn set_speed(&mut self, speed: f64) {
        self.playback_speed = speed;
    }

    /// Get the current playback speed
    ///
    /// Returns the playback speed multiplier (default: 1.0).
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use pmat::services::dap::{Recording, TimelinePlayer};
    /// # let recording = Recording::new("test".to_string(), vec![]);
    /// let mut player = TimelinePlayer::new(recording);
    ///
    /// assert_eq!(player.playback_speed(), 1.0);
    /// player.set_speed(2.0);
    /// assert_eq!(player.playback_speed(), 2.0);
    /// ```
    pub fn playback_speed(&self) -> f64 {
        self.playback_speed
    }

    /// Get a reference to the underlying recording
    ///
    /// Provides access to recording metadata and all snapshots.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use pmat::services::dap::{Recording, TimelinePlayer};
    /// # let recording = Recording::new("test_program".to_string(), vec![]);
    /// let player = TimelinePlayer::new(recording);
    ///
    /// let metadata = player.recording().metadata();
    /// println!("Program: {}", metadata.program);
    /// ```
    pub fn recording(&self) -> &Recording {
        &self.recording
    }
}

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
