// Sprint 78: TUI-001 GREEN - Terminal Event Loop Implementation
//
// Minimal implementation to pass TUI-001 RED tests.
// Provides terminal event handling for interactive timeline TUI.
//
// Sprint 79+: Presentar-terminal Brick architecture (ratatui-free)
// Benefits: Jidoka verification gates, zero-allocation rendering, 95% test coverage

#![cfg(feature = "tui")]

use anyhow::Result;
use std::{collections::VecDeque, io::stdout, time::Duration};

// Presentar-terminal provides crossterm access through its API
// Using crossterm directly through presentar-terminal's re-export
use crossterm::{
    event::{self, Event, KeyEvent},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};

// Presentar core types for Brick trait implementation
#[allow(unused_imports)]
use presentar_core::{Brick, BrickAssertion, BrickBudget, Widget};

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
// EventLoop - Terminal event handling
// ============================================================================

/// Event loop for terminal input handling
pub struct EventLoop {
    /// Whether the event loop is running
    running: bool,
    /// Whether raw mode is enabled
    raw_mode_enabled: bool,
    /// Event queue for testing
    event_queue: VecDeque<TerminalEvent>,
}

impl EventLoop {
    /// Create new event loop
    pub fn new() -> Self {
        Self {
            running: false,
            raw_mode_enabled: false,
            event_queue: VecDeque::new(),
        }
    }

    /// Check if event loop is running
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Check if raw mode is enabled
    pub fn is_raw_mode_enabled(&self) -> bool {
        self.raw_mode_enabled
    }

    /// Check if terminal is available (TTY)
    pub fn is_terminal_available(&self) -> bool {
        use std::io::IsTerminal;
        std::io::stdin().is_terminal()
    }

    /// Enable terminal raw mode
    pub fn enable_raw_mode(&mut self) -> Result<()> {
        if !self.is_terminal_available() {
            anyhow::bail!("Not a terminal");
        }

        terminal::enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;

        self.raw_mode_enabled = true;
        Ok(())
    }

    /// Disable terminal raw mode
    pub fn disable_raw_mode(&mut self) -> Result<()> {
        if self.raw_mode_enabled {
            stdout().execute(LeaveAlternateScreen)?;
            terminal::disable_raw_mode()?;
            self.raw_mode_enabled = false;
        }
        Ok(())
    }

    /// Start event loop
    pub fn start(&mut self) {
        self.running = true;
    }

    /// Stop event loop
    pub fn stop(&mut self) {
        self.running = false;
    }

    /// Poll for event with timeout
    pub fn poll_event(&mut self, timeout: Duration) -> Result<Option<TerminalEvent>> {
        // Check queued events first (for testing)
        if let Some(event) = self.event_queue.pop_front() {
            return Ok(Some(event));
        }

        // Poll crossterm event
        if event::poll(timeout)? {
            let event = event::read()?;
            Ok(self.parse_crossterm_event(event))
        } else {
            Ok(None)
        }
    }

    /// Parse crossterm event
    fn parse_crossterm_event(&self, event: Event) -> Option<TerminalEvent> {
        match event {
            Event::Key(KeyEvent { code, .. }) => {
                let key_code = match code {
                    event::KeyCode::Char(c) => KeyCode::Char(c),
                    event::KeyCode::Right => KeyCode::Right,
                    event::KeyCode::Left => KeyCode::Left,
                    event::KeyCode::Home => KeyCode::Home,
                    event::KeyCode::End => KeyCode::End,
                    _ => return None,
                };
                Some(TerminalEvent::Key(key_code))
            }
            Event::Resize(w, h) => Some(TerminalEvent::Resize(w, h)),
            Event::Mouse(_) => None, // Ignore mouse events (not supported in v1)
            _ => None,
        }
    }

    /// Parse event (used by tests)
    pub fn parse_event(&self, event: TerminalEvent) -> Option<TerminalEvent> {
        match event {
            TerminalEvent::Key(_) => Some(event),
            TerminalEvent::Resize(_, _) => Some(event),
            TerminalEvent::Mouse(_, _) => None, // Filter out mouse events
        }
    }

    /// Queue event for testing
    pub fn queue_event(&mut self, event: TerminalEvent) {
        self.event_queue.push_back(event);
    }

    /// Get next queued event (for testing)
    pub fn next_queued_event(&mut self) -> Option<TerminalEvent> {
        self.event_queue.pop_front()
    }
}

impl Drop for EventLoop {
    fn drop(&mut self) {
        // Restore terminal on drop
        let _ = self.disable_raw_mode();
    }
}

impl Default for EventLoop {
    fn default() -> Self {
        Self::new()
    }
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
    pub fn new(total_frames: usize) -> Self {
        Self {
            total_frames,
            current_frame: 0,
            playback_state: PlaybackState::Paused,
        }
    }

    /// Get total frames
    pub fn total_frames(&self) -> usize {
        self.total_frames
    }

    /// Get current frame position
    pub fn current_frame(&self) -> usize {
        self.current_frame
    }

    /// Set current frame (with bounds checking)
    pub fn set_current_frame(&mut self, frame: usize) {
        self.current_frame = frame.min(self.total_frames.saturating_sub(1));
    }

    /// Advance frame by offset (can be negative)
    pub fn advance_frame(&mut self, offset: i32) {
        let new_frame = self.current_frame as i32 + offset;
        if new_frame < 0 {
            self.current_frame = 0;
        } else {
            self.set_current_frame(new_frame as usize);
        }
    }

    /// Get playback state
    pub fn playback_state(&self) -> PlaybackState {
        self.playback_state
    }

    /// Toggle playback state
    pub fn toggle_playback(&mut self) {
        self.playback_state = match self.playback_state {
            PlaybackState::Paused => PlaybackState::Playing,
            PlaybackState::Playing => PlaybackState::Paused,
        };
    }

    /// Jump to first frame
    pub fn jump_to_start(&mut self) {
        self.current_frame = 0;
    }

    /// Jump to last frame
    pub fn jump_to_end(&mut self) {
        self.current_frame = self.total_frames.saturating_sub(1);
    }

    /// Calculate progress as percentage (0.0 to 100.0)
    pub fn progress_percentage(&self) -> f64 {
        if self.total_frames == 0 {
            return 0.0;
        }
        (self.current_frame as f64 / self.total_frames as f64) * 100.0
    }

    /// Get frame info string (e.g., "50/100")
    pub fn frame_info(&self) -> String {
        format!("{}/{}", self.current_frame, self.total_frames)
    }

    /// Get playback controls text
    pub fn playback_controls_text(&self) -> String {
        match self.playback_state {
            PlaybackState::Paused => "▶ Play".to_string(),
            PlaybackState::Playing => "⏸ Pause".to_string(),
        }
    }

    /// Get keyboard shortcuts hint
    pub fn keyboard_shortcuts(&self) -> String {
        "← Prev | → Next | Space Play/Pause | Home Start | End End | q Quit".to_string()
    }
}

// ============================================================================
// VariableInspectorView - Scrollable variable list view
// ============================================================================

/// Variable inspector view with scrolling support
pub struct VariableInspectorView {
    /// Variables as (name, value) pairs
    variables: Vec<(String, String)>,
    /// Current scroll offset (top visible line)
    scroll_offset: usize,
    /// Viewport height (visible lines)
    viewport_height: usize,
}

impl VariableInspectorView {
    /// Create new empty variable inspector
    pub fn new() -> Self {
        Self {
            variables: Vec::new(),
            scroll_offset: 0,
            viewport_height: 10, // Default viewport height
        }
    }

    /// Create inspector from variable list
    pub fn from_variables(variables: Vec<(String, String)>) -> Self {
        Self {
            variables,
            scroll_offset: 0,
            viewport_height: 10,
        }
    }

    /// Add a variable to the inspector
    pub fn add_variable(&mut self, name: String, value: String) {
        self.variables.push((name, value));
    }

    /// Get total variable count
    pub fn variable_count(&self) -> usize {
        self.variables.len()
    }

    /// Get current scroll offset
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Set scroll offset (with bounds checking)
    pub fn set_scroll_offset(&mut self, offset: usize) {
        if self.variables.is_empty() {
            self.scroll_offset = 0;
            return;
        }
        self.scroll_offset = offset.min(self.variables.len().saturating_sub(1));
    }

    /// Scroll down one line
    pub fn scroll_down(&mut self) {
        if self.variables.is_empty() {
            return;
        }
        self.set_scroll_offset(self.scroll_offset.saturating_add(1));
    }

    /// Scroll up one line
    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    /// Get viewport height
    pub fn viewport_height(&self) -> usize {
        self.viewport_height
    }

    /// Set viewport height
    pub fn set_viewport_height(&mut self, height: usize) {
        self.viewport_height = height;
    }

    /// Scroll down by viewport height
    pub fn page_down(&mut self) {
        let new_offset = self.scroll_offset.saturating_add(self.viewport_height);
        self.set_scroll_offset(new_offset);
    }

    /// Scroll up by viewport height
    pub fn page_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(self.viewport_height);
    }

    /// Get visible range (start_index, end_index)
    pub fn visible_range(&self) -> (usize, usize) {
        let start = self.scroll_offset;
        let end = (start + self.viewport_height).min(self.variables.len());
        (start, end)
    }

    /// Get variable at index
    pub fn get_variable(&self, index: usize) -> Option<(&String, &String)> {
        self.variables.get(index).map(|(name, value)| (name, value))
    }

    /// Format variable line at index
    pub fn format_line(&self, index: usize) -> Option<String> {
        self.get_variable(index)
            .map(|(name, value)| format!("{}: {}", name, value))
    }

    /// Get all visible lines
    pub fn visible_lines(&self) -> Vec<String> {
        let (start, end) = self.visible_range();
        (start..end).filter_map(|i| self.format_line(i)).collect()
    }
}

impl Default for VariableInspectorView {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// StackFrameNavigator - Interactive stack frame selection
// ============================================================================

/// Stack frame navigator for interactive debugging
pub struct StackFrameNavigator {
    /// Stack frames as (function_name, file, line) tuples
    frames: Vec<(String, String, usize)>,
    /// Currently selected frame index
    selected_index: usize,
}

impl StackFrameNavigator {
    /// Create new empty stack frame navigator
    pub fn new() -> Self {
        Self {
            frames: Vec::new(),
            selected_index: 0,
        }
    }

    /// Create navigator from frame list
    pub fn from_frames(frames: Vec<(String, String, usize)>) -> Self {
        Self {
            frames,
            selected_index: 0,
        }
    }

    /// Get total frame count
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    /// Get selected frame index
    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    /// Set selected frame index (with bounds checking)
    pub fn set_selected_index(&mut self, index: usize) {
        if self.frames.is_empty() {
            self.selected_index = 0;
            return;
        }
        self.selected_index = index.min(self.frames.len().saturating_sub(1));
    }

    /// Move selection to next frame (down)
    pub fn select_next(&mut self) {
        if self.frames.is_empty() {
            return;
        }
        self.set_selected_index(self.selected_index.saturating_add(1));
    }

    /// Move selection to previous frame (up)
    pub fn select_previous(&mut self) {
        self.selected_index = self.selected_index.saturating_sub(1);
    }

    /// Get frame at index
    pub fn get_frame(&self, index: usize) -> Option<(&String, &String, usize)> {
        self.frames
            .get(index)
            .map(|(func, file, line)| (func, file, *line))
    }

    /// Get currently selected frame
    pub fn get_selected_frame(&self) -> Option<(&String, &String, usize)> {
        self.get_frame(self.selected_index)
    }

    /// Format frame as "function @ file:line"
    pub fn format_frame_line(&self, index: usize) -> Option<String> {
        self.get_frame(index)
            .map(|(func, file, line)| format!("{} @ {}:{}", func, file, line))
    }

    /// Format frame with selection marker
    pub fn format_frame_with_marker(&self, index: usize) -> Option<String> {
        self.get_frame(index).map(|(func, file, line)| {
            let marker = if self.is_frame_selected(index) {
                "▶"
            } else {
                " "
            };
            format!("{} {} @ {}:{}", marker, func, file, line)
        })
    }

    /// Render all frames with selection markers
    pub fn render_all_frames(&self) -> Vec<String> {
        (0..self.frames.len())
            .filter_map(|i| self.format_frame_with_marker(i))
            .collect()
    }

    /// Add a frame to the navigator
    pub fn add_frame(&mut self, function: String, file: String, line: usize) {
        self.frames.push((function, file, line));
    }

    /// Check if frame is selected
    pub fn is_frame_selected(&self, index: usize) -> bool {
        index == self.selected_index
    }
}

impl Default for StackFrameNavigator {
    fn default() -> Self {
        Self::new()
    }
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

// ============================================================================
// KeyboardHandler - Keyboard shortcut management
// ============================================================================

use std::collections::HashMap;

/// Keyboard shortcut handler
pub struct KeyboardHandler {
    /// Key bindings map
    bindings: HashMap<KeyCode, TuiAction>,
}

impl KeyboardHandler {
    /// Create new keyboard handler with default bindings
    pub fn new() -> Self {
        let mut bindings = HashMap::new();

        // Default key bindings
        bindings.insert(KeyCode::Right, TuiAction::NextFrame);
        bindings.insert(KeyCode::Left, TuiAction::PreviousFrame);
        bindings.insert(KeyCode::Char(' '), TuiAction::TogglePlayback);
        bindings.insert(KeyCode::Home, TuiAction::JumpToStart);
        bindings.insert(KeyCode::End, TuiAction::JumpToEnd);
        bindings.insert(KeyCode::Char('q'), TuiAction::Quit);

        Self { bindings }
    }

    /// Check if handler has default bindings
    pub fn has_default_bindings(&self) -> bool {
        self.bindings.contains_key(&KeyCode::Right)
            && self.bindings.contains_key(&KeyCode::Left)
            && self.bindings.contains_key(&KeyCode::Char(' '))
    }

    /// Bind a key to an action
    pub fn bind_key(&mut self, key: KeyCode, action: TuiAction) {
        self.bindings.insert(key, action);
    }

    /// Unbind a key
    pub fn unbind_key(&mut self, key: KeyCode) {
        self.bindings.remove(&key);
    }

    /// Get action for key
    pub fn get_action(&self, key: KeyCode) -> Option<TuiAction> {
        self.bindings.get(&key).copied()
    }

    /// Check if key is bound
    pub fn is_key_bound(&self, key: KeyCode) -> bool {
        self.bindings.contains_key(&key)
    }

    /// Handle terminal event and return action
    pub fn handle_event(&self, event: &TerminalEvent) -> Option<TuiAction> {
        match event {
            TerminalEvent::Key(key_code) => self.get_action(*key_code),
            _ => None,
        }
    }

    /// List all current bindings
    pub fn list_bindings(&self) -> Vec<(KeyCode, TuiAction)> {
        self.bindings.iter().map(|(k, a)| (*k, *a)).collect()
    }

    /// Generate help text for all bindings
    pub fn generate_help_text(&self) -> String {
        let mut help = String::new();

        // Sort bindings for consistent output
        let mut bindings: Vec<_> = self.bindings.iter().collect();
        bindings.sort_by_key(|(k, _)| format!("{:?}", k));

        for (key, action) in bindings {
            let key_str = match key {
                KeyCode::Char(' ') => "Space".to_string(),
                KeyCode::Char(c) => c.to_string(),
                KeyCode::Right => "→".to_string(),
                KeyCode::Left => "←".to_string(),
                KeyCode::Home => "Home".to_string(),
                KeyCode::End => "End".to_string(),
            };
            help.push_str(&format!("{}: {} | ", key_str, action.description()));
        }

        // Remove trailing " | "
        if help.len() >= 3 {
            help.truncate(help.len() - 3);
        }

        help
    }
}

impl Default for KeyboardHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // TerminalEvent and KeyCode tests
    // ============================================================================

    #[test]
    fn test_terminal_event_key() {
        let event = TerminalEvent::Key(KeyCode::Char('a'));
        assert_eq!(event, TerminalEvent::Key(KeyCode::Char('a')));
    }

    #[test]
    fn test_terminal_event_resize() {
        let event = TerminalEvent::Resize(80, 24);
        assert_eq!(event, TerminalEvent::Resize(80, 24));
    }

    #[test]
    fn test_terminal_event_mouse() {
        let event = TerminalEvent::Mouse(10, 20);
        assert_eq!(event, TerminalEvent::Mouse(10, 20));
    }

    #[test]
    fn test_key_code_char() {
        let key = KeyCode::Char('x');
        assert_eq!(key, KeyCode::Char('x'));
    }

    #[test]
    fn test_key_code_arrows() {
        assert_ne!(KeyCode::Left, KeyCode::Right);
        assert_eq!(KeyCode::Left, KeyCode::Left);
    }

    #[test]
    fn test_key_code_home_end() {
        assert_ne!(KeyCode::Home, KeyCode::End);
    }

    // ============================================================================
    // EventLoop tests
    // ============================================================================

    #[test]
    fn test_event_loop_new() {
        let event_loop = EventLoop::new();
        assert!(!event_loop.is_running());
        assert!(!event_loop.is_raw_mode_enabled());
    }

    #[test]
    fn test_event_loop_default() {
        let event_loop = EventLoop::default();
        assert!(!event_loop.is_running());
    }

    #[test]
    fn test_event_loop_start_stop() {
        let mut event_loop = EventLoop::new();
        assert!(!event_loop.is_running());

        event_loop.start();
        assert!(event_loop.is_running());

        event_loop.stop();
        assert!(!event_loop.is_running());
    }

    #[test]
    fn test_event_loop_queue_event() {
        let mut event_loop = EventLoop::new();
        let event = TerminalEvent::Key(KeyCode::Char('a'));

        event_loop.queue_event(event.clone());

        let next = event_loop.next_queued_event();
        assert_eq!(next, Some(event));
    }

    #[test]
    fn test_event_loop_queue_multiple_events() {
        let mut event_loop = EventLoop::new();

        event_loop.queue_event(TerminalEvent::Key(KeyCode::Char('a')));
        event_loop.queue_event(TerminalEvent::Key(KeyCode::Char('b')));
        event_loop.queue_event(TerminalEvent::Resize(100, 50));

        assert_eq!(event_loop.next_queued_event(), Some(TerminalEvent::Key(KeyCode::Char('a'))));
        assert_eq!(event_loop.next_queued_event(), Some(TerminalEvent::Key(KeyCode::Char('b'))));
        assert_eq!(event_loop.next_queued_event(), Some(TerminalEvent::Resize(100, 50)));
        assert_eq!(event_loop.next_queued_event(), None);
    }

    #[test]
    fn test_event_loop_parse_event_key() {
        let event_loop = EventLoop::new();
        let event = TerminalEvent::Key(KeyCode::Right);

        let parsed = event_loop.parse_event(event.clone());
        assert_eq!(parsed, Some(event));
    }

    #[test]
    fn test_event_loop_parse_event_resize() {
        let event_loop = EventLoop::new();
        let event = TerminalEvent::Resize(120, 40);

        let parsed = event_loop.parse_event(event.clone());
        assert_eq!(parsed, Some(event));
    }

    #[test]
    fn test_event_loop_parse_event_mouse_filtered() {
        let event_loop = EventLoop::new();
        let event = TerminalEvent::Mouse(5, 5);

        // Mouse events should be filtered out
        let parsed = event_loop.parse_event(event);
        assert_eq!(parsed, None);
    }

    // ============================================================================
    // PlaybackState tests
    // ============================================================================

    #[test]
    fn test_playback_state_paused() {
        let state = PlaybackState::Paused;
        assert_eq!(state, PlaybackState::Paused);
    }

    #[test]
    fn test_playback_state_playing() {
        let state = PlaybackState::Playing;
        assert_eq!(state, PlaybackState::Playing);
    }

    #[test]
    fn test_playback_state_inequality() {
        assert_ne!(PlaybackState::Paused, PlaybackState::Playing);
    }

    // ============================================================================
    // TimelineRenderer tests
    // ============================================================================

    #[test]
    fn test_timeline_renderer_new() {
        let renderer = TimelineRenderer::new(100);
        assert_eq!(renderer.total_frames(), 100);
        assert_eq!(renderer.current_frame(), 0);
        assert_eq!(renderer.playback_state(), PlaybackState::Paused);
    }

    #[test]
    fn test_timeline_renderer_set_current_frame() {
        let mut renderer = TimelineRenderer::new(100);
        renderer.set_current_frame(50);
        assert_eq!(renderer.current_frame(), 50);
    }

    #[test]
    fn test_timeline_renderer_set_current_frame_bounds() {
        let mut renderer = TimelineRenderer::new(10);
        renderer.set_current_frame(100); // Exceeds total
        assert_eq!(renderer.current_frame(), 9); // Should be clamped to last frame
    }

    #[test]
    fn test_timeline_renderer_advance_frame_forward() {
        let mut renderer = TimelineRenderer::new(100);
        renderer.set_current_frame(50);
        renderer.advance_frame(5);
        assert_eq!(renderer.current_frame(), 55);
    }

    #[test]
    fn test_timeline_renderer_advance_frame_backward() {
        let mut renderer = TimelineRenderer::new(100);
        renderer.set_current_frame(50);
        renderer.advance_frame(-10);
        assert_eq!(renderer.current_frame(), 40);
    }

    #[test]
    fn test_timeline_renderer_advance_frame_clamp_start() {
        let mut renderer = TimelineRenderer::new(100);
        renderer.set_current_frame(5);
        renderer.advance_frame(-20);
        assert_eq!(renderer.current_frame(), 0);
    }

    #[test]
    fn test_timeline_renderer_toggle_playback() {
        let mut renderer = TimelineRenderer::new(100);
        assert_eq!(renderer.playback_state(), PlaybackState::Paused);

        renderer.toggle_playback();
        assert_eq!(renderer.playback_state(), PlaybackState::Playing);

        renderer.toggle_playback();
        assert_eq!(renderer.playback_state(), PlaybackState::Paused);
    }

    #[test]
    fn test_timeline_renderer_jump_to_start() {
        let mut renderer = TimelineRenderer::new(100);
        renderer.set_current_frame(75);
        renderer.jump_to_start();
        assert_eq!(renderer.current_frame(), 0);
    }

    #[test]
    fn test_timeline_renderer_jump_to_end() {
        let mut renderer = TimelineRenderer::new(100);
        renderer.jump_to_end();
        assert_eq!(renderer.current_frame(), 99);
    }

    #[test]
    fn test_timeline_renderer_progress_percentage() {
        let mut renderer = TimelineRenderer::new(100);
        assert_eq!(renderer.progress_percentage(), 0.0);

        renderer.set_current_frame(50);
        assert!((renderer.progress_percentage() - 50.0).abs() < 0.01);

        renderer.set_current_frame(99);
        assert!((renderer.progress_percentage() - 99.0).abs() < 0.01);
    }

    #[test]
    fn test_timeline_renderer_progress_percentage_empty() {
        let renderer = TimelineRenderer::new(0);
        assert_eq!(renderer.progress_percentage(), 0.0);
    }

    #[test]
    fn test_timeline_renderer_frame_info() {
        let mut renderer = TimelineRenderer::new(100);
        renderer.set_current_frame(42);
        assert_eq!(renderer.frame_info(), "42/100");
    }

    #[test]
    fn test_timeline_renderer_playback_controls_text() {
        let mut renderer = TimelineRenderer::new(100);
        assert!(renderer.playback_controls_text().contains("Play"));

        renderer.toggle_playback();
        assert!(renderer.playback_controls_text().contains("Pause"));
    }

    #[test]
    fn test_timeline_renderer_keyboard_shortcuts() {
        let renderer = TimelineRenderer::new(100);
        let shortcuts = renderer.keyboard_shortcuts();
        assert!(shortcuts.contains("Prev"));
        assert!(shortcuts.contains("Next"));
        assert!(shortcuts.contains("Quit"));
    }

    // ============================================================================
    // VariableInspectorView tests
    // ============================================================================

    #[test]
    fn test_variable_inspector_new() {
        let view = VariableInspectorView::new();
        assert_eq!(view.variable_count(), 0);
        assert_eq!(view.scroll_offset(), 0);
    }

    #[test]
    fn test_variable_inspector_default() {
        let view = VariableInspectorView::default();
        assert_eq!(view.variable_count(), 0);
    }

    #[test]
    fn test_variable_inspector_from_variables() {
        let vars = vec![
            ("x".to_string(), "10".to_string()),
            ("y".to_string(), "20".to_string()),
        ];
        let view = VariableInspectorView::from_variables(vars);
        assert_eq!(view.variable_count(), 2);
    }

    #[test]
    fn test_variable_inspector_add_variable() {
        let mut view = VariableInspectorView::new();
        view.add_variable("counter".to_string(), "42".to_string());
        assert_eq!(view.variable_count(), 1);
    }

    #[test]
    fn test_variable_inspector_scroll_down() {
        let vars = vec![
            ("a".to_string(), "1".to_string()),
            ("b".to_string(), "2".to_string()),
            ("c".to_string(), "3".to_string()),
        ];
        let mut view = VariableInspectorView::from_variables(vars);

        assert_eq!(view.scroll_offset(), 0);
        view.scroll_down();
        assert_eq!(view.scroll_offset(), 1);
    }

    #[test]
    fn test_variable_inspector_scroll_up() {
        let vars = vec![
            ("a".to_string(), "1".to_string()),
            ("b".to_string(), "2".to_string()),
        ];
        let mut view = VariableInspectorView::from_variables(vars);
        view.set_scroll_offset(1);

        view.scroll_up();
        assert_eq!(view.scroll_offset(), 0);
    }

    #[test]
    fn test_variable_inspector_scroll_up_at_top() {
        let mut view = VariableInspectorView::new();
        view.scroll_up();
        assert_eq!(view.scroll_offset(), 0);
    }

    #[test]
    fn test_variable_inspector_viewport_height() {
        let mut view = VariableInspectorView::new();
        assert_eq!(view.viewport_height(), 10);

        view.set_viewport_height(20);
        assert_eq!(view.viewport_height(), 20);
    }

    #[test]
    fn test_variable_inspector_page_down() {
        let vars: Vec<_> = (0..30).map(|i| (format!("var{}", i), format!("{}", i))).collect();
        let mut view = VariableInspectorView::from_variables(vars);
        view.set_viewport_height(10);

        view.page_down();
        assert_eq!(view.scroll_offset(), 10);
    }

    #[test]
    fn test_variable_inspector_page_up() {
        let vars: Vec<_> = (0..30).map(|i| (format!("var{}", i), format!("{}", i))).collect();
        let mut view = VariableInspectorView::from_variables(vars);
        view.set_viewport_height(10);
        view.set_scroll_offset(20);

        view.page_up();
        assert_eq!(view.scroll_offset(), 10);
    }

    #[test]
    fn test_variable_inspector_visible_range() {
        let vars: Vec<_> = (0..30).map(|i| (format!("var{}", i), format!("{}", i))).collect();
        let mut view = VariableInspectorView::from_variables(vars);
        view.set_viewport_height(10);

        let (start, end) = view.visible_range();
        assert_eq!(start, 0);
        assert_eq!(end, 10);
    }

    #[test]
    fn test_variable_inspector_get_variable() {
        let vars = vec![
            ("name".to_string(), "value".to_string()),
        ];
        let view = VariableInspectorView::from_variables(vars);

        let var = view.get_variable(0);
        assert!(var.is_some());
        let (name, value) = var.unwrap();
        assert_eq!(name, "name");
        assert_eq!(value, "value");
    }

    #[test]
    fn test_variable_inspector_get_variable_out_of_bounds() {
        let view = VariableInspectorView::new();
        assert!(view.get_variable(0).is_none());
    }

    #[test]
    fn test_variable_inspector_format_line() {
        let vars = vec![
            ("x".to_string(), "42".to_string()),
        ];
        let view = VariableInspectorView::from_variables(vars);

        let line = view.format_line(0);
        assert_eq!(line, Some("x: 42".to_string()));
    }

    #[test]
    fn test_variable_inspector_visible_lines() {
        let vars = vec![
            ("a".to_string(), "1".to_string()),
            ("b".to_string(), "2".to_string()),
        ];
        let mut view = VariableInspectorView::from_variables(vars);
        view.set_viewport_height(5);

        let lines = view.visible_lines();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "a: 1");
        assert_eq!(lines[1], "b: 2");
    }

    // ============================================================================
    // StackFrameNavigator tests
    // ============================================================================

    #[test]
    fn test_stack_frame_navigator_new() {
        let nav = StackFrameNavigator::new();
        assert_eq!(nav.frame_count(), 0);
        assert_eq!(nav.selected_index(), 0);
    }

    #[test]
    fn test_stack_frame_navigator_default() {
        let nav = StackFrameNavigator::default();
        assert_eq!(nav.frame_count(), 0);
    }

    #[test]
    fn test_stack_frame_navigator_from_frames() {
        let frames = vec![
            ("main".to_string(), "main.rs".to_string(), 10),
            ("helper".to_string(), "lib.rs".to_string(), 20),
        ];
        let nav = StackFrameNavigator::from_frames(frames);
        assert_eq!(nav.frame_count(), 2);
    }

    #[test]
    fn test_stack_frame_navigator_add_frame() {
        let mut nav = StackFrameNavigator::new();
        nav.add_frame("test".to_string(), "test.rs".to_string(), 42);
        assert_eq!(nav.frame_count(), 1);
    }

    #[test]
    fn test_stack_frame_navigator_select_next() {
        let frames = vec![
            ("a".to_string(), "a.rs".to_string(), 1),
            ("b".to_string(), "b.rs".to_string(), 2),
            ("c".to_string(), "c.rs".to_string(), 3),
        ];
        let mut nav = StackFrameNavigator::from_frames(frames);

        assert_eq!(nav.selected_index(), 0);
        nav.select_next();
        assert_eq!(nav.selected_index(), 1);
        nav.select_next();
        assert_eq!(nav.selected_index(), 2);
    }

    #[test]
    fn test_stack_frame_navigator_select_next_at_end() {
        let frames = vec![
            ("a".to_string(), "a.rs".to_string(), 1),
        ];
        let mut nav = StackFrameNavigator::from_frames(frames);
        nav.select_next();
        nav.select_next(); // Should stay at last
        assert_eq!(nav.selected_index(), 0);
    }

    #[test]
    fn test_stack_frame_navigator_select_previous() {
        let frames = vec![
            ("a".to_string(), "a.rs".to_string(), 1),
            ("b".to_string(), "b.rs".to_string(), 2),
        ];
        let mut nav = StackFrameNavigator::from_frames(frames);
        nav.set_selected_index(1);

        nav.select_previous();
        assert_eq!(nav.selected_index(), 0);
    }

    #[test]
    fn test_stack_frame_navigator_select_previous_at_start() {
        let mut nav = StackFrameNavigator::new();
        nav.select_previous();
        assert_eq!(nav.selected_index(), 0);
    }

    #[test]
    fn test_stack_frame_navigator_get_frame() {
        let frames = vec![
            ("main".to_string(), "main.rs".to_string(), 42),
        ];
        let nav = StackFrameNavigator::from_frames(frames);

        let frame = nav.get_frame(0);
        assert!(frame.is_some());
        let (func, file, line) = frame.unwrap();
        assert_eq!(func, "main");
        assert_eq!(file, "main.rs");
        assert_eq!(line, 42);
    }

    #[test]
    fn test_stack_frame_navigator_get_selected_frame() {
        let frames = vec![
            ("first".to_string(), "f.rs".to_string(), 1),
            ("second".to_string(), "s.rs".to_string(), 2),
        ];
        let mut nav = StackFrameNavigator::from_frames(frames);
        nav.set_selected_index(1);

        let selected = nav.get_selected_frame();
        assert!(selected.is_some());
        let (func, _, _) = selected.unwrap();
        assert_eq!(func, "second");
    }

    #[test]
    fn test_stack_frame_navigator_format_frame_line() {
        let frames = vec![
            ("process".to_string(), "mod.rs".to_string(), 100),
        ];
        let nav = StackFrameNavigator::from_frames(frames);

        let line = nav.format_frame_line(0);
        assert_eq!(line, Some("process @ mod.rs:100".to_string()));
    }

    #[test]
    fn test_stack_frame_navigator_format_frame_with_marker() {
        let frames = vec![
            ("a".to_string(), "a.rs".to_string(), 1),
            ("b".to_string(), "b.rs".to_string(), 2),
        ];
        let nav = StackFrameNavigator::from_frames(frames);

        let selected_line = nav.format_frame_with_marker(0);
        assert!(selected_line.unwrap().contains("▶"));

        let unselected_line = nav.format_frame_with_marker(1);
        assert!(!unselected_line.unwrap().starts_with("▶"));
    }

    #[test]
    fn test_stack_frame_navigator_is_frame_selected() {
        let frames = vec![
            ("a".to_string(), "a.rs".to_string(), 1),
            ("b".to_string(), "b.rs".to_string(), 2),
        ];
        let nav = StackFrameNavigator::from_frames(frames);

        assert!(nav.is_frame_selected(0));
        assert!(!nav.is_frame_selected(1));
    }

    #[test]
    fn test_stack_frame_navigator_render_all_frames() {
        let frames = vec![
            ("a".to_string(), "a.rs".to_string(), 1),
            ("b".to_string(), "b.rs".to_string(), 2),
        ];
        let nav = StackFrameNavigator::from_frames(frames);

        let rendered = nav.render_all_frames();
        assert_eq!(rendered.len(), 2);
    }

    // ============================================================================
    // TuiAction tests
    // ============================================================================

    #[test]
    fn test_tui_action_description() {
        assert_eq!(TuiAction::NextFrame.description(), "Next frame");
        assert_eq!(TuiAction::PreviousFrame.description(), "Previous frame");
        assert_eq!(TuiAction::TogglePlayback.description(), "Play/Pause");
        assert_eq!(TuiAction::JumpToStart.description(), "Jump to start");
        assert_eq!(TuiAction::JumpToEnd.description(), "Jump to end");
        assert_eq!(TuiAction::Quit.description(), "Quit");
        assert_eq!(TuiAction::ScrollDown.description(), "Scroll down");
        assert_eq!(TuiAction::ScrollUp.description(), "Scroll up");
        assert_eq!(TuiAction::SelectNextFrame.description(), "Next stack frame");
        assert_eq!(TuiAction::SelectPreviousFrame.description(), "Previous stack frame");
    }

    #[test]
    fn test_tui_action_equality() {
        assert_eq!(TuiAction::Quit, TuiAction::Quit);
        assert_ne!(TuiAction::Quit, TuiAction::NextFrame);
    }

    // ============================================================================
    // KeyboardHandler tests
    // ============================================================================

    #[test]
    fn test_keyboard_handler_new() {
        let handler = KeyboardHandler::new();
        assert!(handler.has_default_bindings());
    }

    #[test]
    fn test_keyboard_handler_default() {
        let handler = KeyboardHandler::default();
        assert!(handler.has_default_bindings());
    }

    #[test]
    fn test_keyboard_handler_default_bindings() {
        let handler = KeyboardHandler::new();

        assert_eq!(handler.get_action(KeyCode::Right), Some(TuiAction::NextFrame));
        assert_eq!(handler.get_action(KeyCode::Left), Some(TuiAction::PreviousFrame));
        assert_eq!(handler.get_action(KeyCode::Char(' ')), Some(TuiAction::TogglePlayback));
        assert_eq!(handler.get_action(KeyCode::Home), Some(TuiAction::JumpToStart));
        assert_eq!(handler.get_action(KeyCode::End), Some(TuiAction::JumpToEnd));
        assert_eq!(handler.get_action(KeyCode::Char('q')), Some(TuiAction::Quit));
    }

    #[test]
    fn test_keyboard_handler_bind_key() {
        let mut handler = KeyboardHandler::new();
        handler.bind_key(KeyCode::Char('n'), TuiAction::NextFrame);

        assert_eq!(handler.get_action(KeyCode::Char('n')), Some(TuiAction::NextFrame));
    }

    #[test]
    fn test_keyboard_handler_unbind_key() {
        let mut handler = KeyboardHandler::new();
        handler.unbind_key(KeyCode::Right);

        assert_eq!(handler.get_action(KeyCode::Right), None);
    }

    #[test]
    fn test_keyboard_handler_is_key_bound() {
        let handler = KeyboardHandler::new();

        assert!(handler.is_key_bound(KeyCode::Right));
        assert!(!handler.is_key_bound(KeyCode::Char('z')));
    }

    #[test]
    fn test_keyboard_handler_handle_event_key() {
        let handler = KeyboardHandler::new();
        let event = TerminalEvent::Key(KeyCode::Right);

        let action = handler.handle_event(&event);
        assert_eq!(action, Some(TuiAction::NextFrame));
    }

    #[test]
    fn test_keyboard_handler_handle_event_resize() {
        let handler = KeyboardHandler::new();
        let event = TerminalEvent::Resize(80, 24);

        let action = handler.handle_event(&event);
        assert_eq!(action, None);
    }

    #[test]
    fn test_keyboard_handler_handle_event_unbound_key() {
        let handler = KeyboardHandler::new();
        let event = TerminalEvent::Key(KeyCode::Char('z'));

        let action = handler.handle_event(&event);
        assert_eq!(action, None);
    }

    #[test]
    fn test_keyboard_handler_list_bindings() {
        let handler = KeyboardHandler::new();
        let bindings = handler.list_bindings();

        assert!(!bindings.is_empty());
        assert!(bindings.len() >= 6); // At least 6 default bindings
    }

    #[test]
    fn test_keyboard_handler_generate_help_text() {
        let handler = KeyboardHandler::new();
        let help = handler.generate_help_text();

        assert!(!help.is_empty());
        assert!(help.contains("Next frame") || help.contains("Previous frame"));
    }

    #[test]
    fn test_keyboard_handler_custom_binding() {
        let mut handler = KeyboardHandler::new();
        handler.bind_key(KeyCode::Char('j'), TuiAction::ScrollDown);
        handler.bind_key(KeyCode::Char('k'), TuiAction::ScrollUp);

        assert_eq!(handler.get_action(KeyCode::Char('j')), Some(TuiAction::ScrollDown));
        assert_eq!(handler.get_action(KeyCode::Char('k')), Some(TuiAction::ScrollUp));
    }
}
