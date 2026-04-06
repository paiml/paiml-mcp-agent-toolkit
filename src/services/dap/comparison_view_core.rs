// ComparisonView - Constructor, frame accessors, navigation methods

impl ComparisonView {
    /// Create a new comparison view from two recordings
    ///
    /// # Example
    /// ```ignore
    /// use pmat::services::dap::{Recording, ComparisonView};
    ///
    /// let recording_a = Recording::new("program_a".to_string(), vec![]);
    /// let recording_b = Recording::new("program_b".to_string(), vec![]);
    ///
    /// let comparison = ComparisonView::new(recording_a, recording_b);
    /// ```
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(recording_a: Recording, recording_b: Recording) -> Self {
        let name_a = recording_a.metadata().program.clone();
        let name_b = recording_b.metadata().program.clone();

        Self {
            player_a: TimelinePlayer::new(recording_a),
            player_b: TimelinePlayer::new(recording_b),
            sync_mode: SyncMode::ByFrame,
            name_a,
            name_b,
        }
    }

    /// Get current frame number for recording A
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn current_frame_a(&self) -> usize {
        self.player_a.current_frame()
    }

    /// Get current frame number for recording B
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn current_frame_b(&self) -> usize {
        self.player_b.current_frame()
    }

    /// Get total frames for recording A
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn total_frames_a(&self) -> usize {
        self.player_a.total_frames()
    }

    /// Get total frames for recording B
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn total_frames_b(&self) -> usize {
        self.player_b.total_frames()
    }

    /// Get minimum frame count of both recordings
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn total_frames_min(&self) -> usize {
        self.total_frames_a().min(self.total_frames_b())
    }

    /// Get maximum frame count of both recordings
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn total_frames_max(&self) -> usize {
        self.total_frames_a().max(self.total_frames_b())
    }

    /// Check if recording A is exhausted (at or past its last frame)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn recording_a_exhausted(&self) -> bool {
        let total = self.total_frames_a();
        if total == 0 {
            return true;
        }
        self.current_frame_a() >= total - 1
    }

    /// Check if recording B is exhausted (at or past its last frame)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn recording_b_exhausted(&self) -> bool {
        let total = self.total_frames_b();
        if total == 0 {
            return true;
        }
        self.current_frame_b() >= total - 1
    }

    /// Get current synchronization mode
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn sync_mode(&self) -> SyncMode {
        self.sync_mode
    }

    /// Set synchronization mode
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn set_sync_mode(&mut self, mode: SyncMode) {
        self.sync_mode = mode;
    }

    /// Advance both recordings to next frame
    ///
    /// Returns Ok(()) if successful, Err if either recording is exhausted
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn next_frame(&mut self) -> Result<()> {
        // Attempt to advance both
        let a_result = self.player_a.next_frame();
        let b_result = self.player_b.next_frame();

        // If either failed, return error
        if a_result.is_none() && b_result.is_none() {
            anyhow::bail!("Both recordings exhausted");
        }

        Ok(())
    }

    /// Move both recordings to previous frame
    ///
    /// Returns Ok(()) if successful, Err if either recording is at frame 0
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn prev_frame(&mut self) -> Result<()> {
        // Attempt to move both back
        let a_result = self.player_a.prev_frame();
        let b_result = self.player_b.prev_frame();

        // If both failed (at start), return error
        if a_result.is_none() && b_result.is_none() {
            anyhow::bail!("Both recordings at start");
        }

        Ok(())
    }

    /// Jump both recordings to specific frame
    ///
    /// Returns Ok(()) if successful, Err if frame is out of bounds for both
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn jump_to(&mut self, frame: usize) -> Result<()> {
        // Jump both players
        let a_result = self.player_a.jump_to(frame);
        let b_result = self.player_b.jump_to(frame);

        // If both failed, return error
        if a_result.is_err() && b_result.is_err() {
            anyhow::bail!("Frame {} out of bounds for both recordings", frame);
        }

        Ok(())
    }
}
