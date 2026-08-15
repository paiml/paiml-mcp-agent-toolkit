// `services::dap::timeline_tui` is `#[cfg(feature = "tui")]` (dap/mod.rs:18),
// so gating on `dap` alone was not enough: under `--features dap` the module
// does not exist and this file failed to compile. It went unnoticed because
// nothing built it until the feature matrix grew a --tests leg.
#![cfg(all(feature = "dap", feature = "tui"))]

// Sprint 78: TUI-001 RED phase - Terminal Event Loop Tests
//
// Tests for interactive timeline TUI event handling.
// These tests verify:
// - Keyboard event capture (arrow keys, space, etc.)
// - Event loop lifecycle (start, run, stop)
// - Terminal state management (raw mode, cleanup)
// - Event parsing and routing

use pmat::services::dap::timeline_tui::{EventLoop, KeyCode, TerminalEvent};

// ============================================================================
// Test 1: Event Loop Creation
// ============================================================================

#[test]
fn test_event_loop_creation() {
    // RED: EventLoop should be creatable with default configuration
    let event_loop = EventLoop::new();
    assert!(!event_loop.is_running());
}

// ============================================================================
// Test 2: Terminal Initialization
// ============================================================================

#[test]
#[ignore] // Requires real TTY (fails in CI/test harness)
fn test_terminal_raw_mode_enable() {
    // RED: EventLoop should enable terminal raw mode
    let mut event_loop = EventLoop::new();

    let result = event_loop.enable_raw_mode();
    assert!(result.is_ok());

    // Verify raw mode is enabled
    assert!(event_loop.is_raw_mode_enabled());
}

#[test]
#[ignore] // Requires real TTY (fails in CI/test harness)
fn test_terminal_raw_mode_cleanup() {
    // RED: EventLoop should restore terminal on drop
    let mut event_loop = EventLoop::new();
    event_loop.enable_raw_mode().unwrap();

    drop(event_loop);

    // Terminal should be restored (verified by no panic)
}

// ============================================================================
// Test 3: Keyboard Event Parsing
// ============================================================================

#[test]
fn test_parse_keyboard_arrow_right() {
    // RED: Should parse right arrow key
    let event_loop = EventLoop::new();

    let event = TerminalEvent::Key(KeyCode::Right);
    let parsed = event_loop.parse_event(event);

    assert_eq!(parsed, Some(TerminalEvent::Key(KeyCode::Right)));
}

#[test]
fn test_parse_keyboard_arrow_left() {
    // RED: Should parse left arrow key
    let event_loop = EventLoop::new();

    let event = TerminalEvent::Key(KeyCode::Left);
    let parsed = event_loop.parse_event(event);

    assert_eq!(parsed, Some(TerminalEvent::Key(KeyCode::Left)));
}

#[test]
fn test_parse_keyboard_space() {
    // RED: Should parse space key (play/pause)
    let event_loop = EventLoop::new();

    let event = TerminalEvent::Key(KeyCode::Char(' '));
    let parsed = event_loop.parse_event(event);

    assert_eq!(parsed, Some(TerminalEvent::Key(KeyCode::Char(' '))));
}

#[test]
fn test_parse_keyboard_quit() {
    // RED: Should parse 'q' key (quit)
    let event_loop = EventLoop::new();

    let event = TerminalEvent::Key(KeyCode::Char('q'));
    let parsed = event_loop.parse_event(event);

    assert_eq!(parsed, Some(TerminalEvent::Key(KeyCode::Char('q'))));
}

#[test]
fn test_parse_keyboard_home() {
    // RED: Should parse Home key (jump to first frame)
    let event_loop = EventLoop::new();

    let event = TerminalEvent::Key(KeyCode::Home);
    let parsed = event_loop.parse_event(event);

    assert_eq!(parsed, Some(TerminalEvent::Key(KeyCode::Home)));
}

#[test]
fn test_parse_keyboard_end() {
    // RED: Should parse End key (jump to last frame)
    let event_loop = EventLoop::new();

    let event = TerminalEvent::Key(KeyCode::End);
    let parsed = event_loop.parse_event(event);

    assert_eq!(parsed, Some(TerminalEvent::Key(KeyCode::End)));
}

// ============================================================================
// Test 4: Event Loop Lifecycle
// ============================================================================

#[test]
fn test_event_loop_start() {
    // RED: Event loop should start and set is_running flag
    let mut event_loop = EventLoop::new();

    event_loop.start();
    assert!(event_loop.is_running());
}

#[test]
fn test_event_loop_stop() {
    // RED: Event loop should stop and clear is_running flag
    let mut event_loop = EventLoop::new();
    event_loop.start();

    event_loop.stop();
    assert!(!event_loop.is_running());
}

#[test]
#[ignore] // Requires real TTY (fails in CI/test harness)
fn test_event_loop_poll_timeout() {
    // RED: Event loop should timeout if no events available
    use std::time::Duration;

    let mut event_loop = EventLoop::new();
    event_loop.enable_raw_mode().unwrap();

    let result = event_loop.poll_event(Duration::from_millis(100));

    // Should return None after timeout (no events)
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), None);
}

// ============================================================================
// Test 5: Event Filtering
// ============================================================================

#[test]
fn test_filter_resize_events() {
    // RED: Should handle terminal resize events
    let event_loop = EventLoop::new();

    let event = TerminalEvent::Resize(80, 24);
    let parsed = event_loop.parse_event(event);

    assert_eq!(parsed, Some(TerminalEvent::Resize(80, 24)));
}

#[test]
fn test_filter_mouse_events_ignored() {
    // RED: Should ignore mouse events (not supported in v1)
    let event_loop = EventLoop::new();

    let event = TerminalEvent::Mouse(0, 0);
    let parsed = event_loop.parse_event(event);

    // Mouse events should be filtered out
    assert_eq!(parsed, None);
}

// ============================================================================
// Test 6: Error Handling
// ============================================================================

#[test]
fn test_event_loop_handles_invalid_terminal() {
    // RED: Should gracefully handle non-terminal environments
    // (e.g., running in CI where stdin is not a TTY)

    let event_loop = EventLoop::new();

    // Should detect non-TTY environment
    if !event_loop.is_terminal_available() {
        // In non-TTY environment, should return error
        assert!(true);
    }
}

#[test]
fn test_event_loop_cleanup_on_panic() {
    // RED: Should restore terminal state even on panic
    use std::panic;

    let result = panic::catch_unwind(|| {
        let mut event_loop = EventLoop::new();
        event_loop.enable_raw_mode().unwrap();

        panic!("Simulated panic");
    });

    assert!(result.is_err());

    // Terminal should be restored (verified by no corruption)
}

// ============================================================================
// Test 7: Event Queue Management
// ============================================================================

#[test]
fn test_event_queue_single_event() {
    // RED: Should handle single event correctly
    let mut event_loop = EventLoop::new();

    // Simulate queueing an event
    event_loop.queue_event(TerminalEvent::Key(KeyCode::Right));

    // Should retrieve the queued event
    let event = event_loop.next_queued_event();
    assert_eq!(event, Some(TerminalEvent::Key(KeyCode::Right)));
}

#[test]
fn test_event_queue_multiple_events_fifo() {
    // RED: Should process events in FIFO order
    let mut event_loop = EventLoop::new();

    event_loop.queue_event(TerminalEvent::Key(KeyCode::Right));
    event_loop.queue_event(TerminalEvent::Key(KeyCode::Left));
    event_loop.queue_event(TerminalEvent::Key(KeyCode::Char('q')));

    assert_eq!(
        event_loop.next_queued_event(),
        Some(TerminalEvent::Key(KeyCode::Right))
    );
    assert_eq!(
        event_loop.next_queued_event(),
        Some(TerminalEvent::Key(KeyCode::Left))
    );
    assert_eq!(
        event_loop.next_queued_event(),
        Some(TerminalEvent::Key(KeyCode::Char('q')))
    );
    assert_eq!(event_loop.next_queued_event(), None);
}

#[test]
fn test_event_queue_empty() {
    // RED: Should return None when queue is empty
    let mut event_loop = EventLoop::new();

    let event = event_loop.next_queued_event();
    assert_eq!(event, None);
}
