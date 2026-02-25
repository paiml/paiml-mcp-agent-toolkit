// Playback control methods for TimelinePlayer

impl TimelinePlayer {
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
