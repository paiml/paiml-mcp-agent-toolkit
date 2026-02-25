// Navigation and constructor methods for TimelinePlayer

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
}
