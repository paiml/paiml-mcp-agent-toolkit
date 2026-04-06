#![cfg_attr(coverage_nightly, coverage(off))]
// TimelineRenderer - Timeline visualization state

use super::types::PlaybackState;

// ============================================================================
// TimelineRenderer - Timeline visualization state
// ============================================================================

/// Timeline renderer for frame-based debugging visualization
pub struct TimelineRenderer {
    /// Total number of frames
    total_frames: usize,
    /// Current frame position
    current_frame: usize,
    /// Playback state
    playback_state: PlaybackState,
}

impl TimelineRenderer {
    /// Create new timeline renderer with frame count
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(total_frames: usize) -> Self {
        Self {
            total_frames,
            current_frame: 0,
            playback_state: PlaybackState::Paused,
        }
    }

    /// Get total frames
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn total_frames(&self) -> usize {
        self.total_frames
    }

    /// Get current frame position
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn current_frame(&self) -> usize {
        self.current_frame
    }

    /// Set current frame (with bounds checking)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn set_current_frame(&mut self, frame: usize) {
        self.current_frame = frame.min(self.total_frames.saturating_sub(1));
    }

    /// Advance frame by offset (can be negative)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn advance_frame(&mut self, offset: i32) {
        let new_frame = self.current_frame as i32 + offset;
        if new_frame < 0 {
            self.current_frame = 0;
        } else {
            self.set_current_frame(new_frame as usize);
        }
    }

    /// Get playback state
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn playback_state(&self) -> PlaybackState {
        self.playback_state
    }

    /// Toggle playback state
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn toggle_playback(&mut self) {
        self.playback_state = match self.playback_state {
            PlaybackState::Paused => PlaybackState::Playing,
            PlaybackState::Playing => PlaybackState::Paused,
        };
    }

    /// Jump to first frame
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn jump_to_start(&mut self) {
        self.current_frame = 0;
    }

    /// Jump to last frame
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn jump_to_end(&mut self) {
        self.current_frame = self.total_frames.saturating_sub(1);
    }

    /// Calculate progress as percentage (0.0 to 100.0)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn progress_percentage(&self) -> f64 {
        if self.total_frames == 0 {
            return 0.0;
        }
        (self.current_frame as f64 / self.total_frames as f64) * 100.0
    }

    /// Get frame info string (e.g., "50/100")
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn frame_info(&self) -> String {
        format!("{}/{}", self.current_frame, self.total_frames)
    }

    /// Get playback controls text
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn playback_controls_text(&self) -> String {
        match self.playback_state {
            PlaybackState::Paused => "▶ Play".to_string(),
            PlaybackState::Playing => "⏸ Pause".to_string(),
        }
    }

    /// Get keyboard shortcuts hint
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn keyboard_shortcuts(&self) -> String {
        "← Prev | → Next | Space Play/Pause | Home Start | End End | q Quit".to_string()
    }
}
