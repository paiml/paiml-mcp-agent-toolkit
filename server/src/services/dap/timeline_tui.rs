// Sprint 78: TUI-001 GREEN - Terminal Event Loop Implementation
//
// Minimal implementation to pass TUI-001 RED tests.
// Provides terminal event handling for interactive timeline TUI.

#![cfg(feature = "tui")]

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyEvent},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use std::{
    collections::VecDeque,
    io::stdout,
    time::Duration,
};

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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
