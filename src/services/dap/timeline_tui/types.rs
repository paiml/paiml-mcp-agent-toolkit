#![cfg_attr(coverage_nightly, coverage(off))]
// Terminal event types, key codes, playback state, and TUI actions

// ============================================================================
// Public Types
// ============================================================================

/// Terminal event types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerminalEvent {
    /// Keyboard key press
    Key(KeyCode),
    /// Terminal resize (width, height)
    Resize(u16, u16),
    /// Mouse event (x, y) - not supported in v1
    Mouse(u16, u16),
}

/// Key codes for keyboard events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    /// Character key
    Char(char),
    /// Right arrow
    Right,
    /// Left arrow
    Left,
    /// Home key
    Home,
    /// End key
    End,
}

// ============================================================================
// PlaybackState - Playback control state
// ============================================================================

/// Playback state for timeline
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    /// Playback is paused
    Paused,
    /// Playback is active
    Playing,
}

// ============================================================================
// TuiAction - Actions that can be performed in the TUI
// ============================================================================

/// Actions available in the timeline TUI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TuiAction {
    /// Move to next frame
    NextFrame,
    /// Move to previous frame
    PreviousFrame,
    /// Toggle playback (play/pause)
    TogglePlayback,
    /// Jump to first frame
    JumpToStart,
    /// Jump to last frame
    JumpToEnd,
    /// Quit the TUI
    Quit,
    /// Scroll down in variable inspector
    ScrollDown,
    /// Scroll up in variable inspector
    ScrollUp,
    /// Select next stack frame
    SelectNextFrame,
    /// Select previous stack frame
    SelectPreviousFrame,
}

impl TuiAction {
    /// Get human-readable description of action
    pub fn description(&self) -> &str {
        match self {
            TuiAction::NextFrame => "Next frame",
            TuiAction::PreviousFrame => "Previous frame",
            TuiAction::TogglePlayback => "Play/Pause",
            TuiAction::JumpToStart => "Jump to start",
            TuiAction::JumpToEnd => "Jump to end",
            TuiAction::Quit => "Quit",
            TuiAction::ScrollDown => "Scroll down",
            TuiAction::ScrollUp => "Scroll up",
            TuiAction::SelectNextFrame => "Next stack frame",
            TuiAction::SelectPreviousFrame => "Previous stack frame",
        }
    }
}
