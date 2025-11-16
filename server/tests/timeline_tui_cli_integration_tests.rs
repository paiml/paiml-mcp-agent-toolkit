#![cfg(feature = "tui")]

// Sprint 78: TUI-006 RED phase - CLI Integration Tests
//
// Tests for CLI integration with --interactive flag.
// These tests verify:
// - --interactive flag recognition
// - Interactive mode activation
// - Non-interactive mode default behavior
// - Flag parsing and validation

use pmat::cli::handlers::TimelineMode;

// ============================================================================
// Test 1: TimelineMode Enum
// ============================================================================

#[test]
fn test_timeline_mode_enum_variants() {
    // RED: Should have Interactive and NonInteractive modes
    let interactive = TimelineMode::Interactive;
    let non_interactive = TimelineMode::NonInteractive;

    assert_ne!(
        format!("{:?}", interactive),
        format!("{:?}", non_interactive)
    );
}

// ============================================================================
// Test 2: Interactive Flag Parsing
// ============================================================================

#[test]
fn test_parse_interactive_flag_present() {
    // RED: Should detect --interactive flag
    let args = vec!["pmat", "timeline", "test.pmat", "--interactive"];
    let mode = TimelineMode::from_args(&args);

    assert_eq!(mode, TimelineMode::Interactive);
}

#[test]
fn test_parse_interactive_flag_absent() {
    // RED: Should default to non-interactive mode
    let args = vec!["pmat", "timeline", "test.pmat"];
    let mode = TimelineMode::from_args(&args);

    assert_eq!(mode, TimelineMode::NonInteractive);
}

#[test]
fn test_parse_interactive_short_flag() {
    // RED: Should support -i short form
    let args = vec!["pmat", "timeline", "test.pmat", "-i"];
    let mode = TimelineMode::from_args(&args);

    assert_eq!(mode, TimelineMode::Interactive);
}

// ============================================================================
// Test 3: Mode Detection
// ============================================================================

#[test]
fn test_is_interactive_mode() {
    // RED: Should identify interactive mode
    let mode = TimelineMode::Interactive;

    assert!(mode.is_interactive());
    assert!(!mode.is_non_interactive());
}

#[test]
fn test_is_non_interactive_mode() {
    // RED: Should identify non-interactive mode
    let mode = TimelineMode::NonInteractive;

    assert!(mode.is_non_interactive());
    assert!(!mode.is_interactive());
}

// ============================================================================
// Test 4: Terminal Requirement Check
// ============================================================================

#[test]
fn test_interactive_requires_terminal() {
    // RED: Interactive mode should require TTY
    let mode = TimelineMode::Interactive;

    assert!(mode.requires_terminal());
}

#[test]
fn test_non_interactive_no_terminal_requirement() {
    // RED: Non-interactive mode should not require TTY
    let mode = TimelineMode::NonInteractive;

    assert!(!mode.requires_terminal());
}

// ============================================================================
// Test 5: Error Handling
// ============================================================================

#[test]
fn test_interactive_mode_error_when_no_tty() {
    // RED: Should error if interactive mode requested without TTY
    let mode = TimelineMode::Interactive;
    let has_tty = false;

    let result = mode.validate_terminal_availability(has_tty);

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("TTY"));
}

#[test]
fn test_non_interactive_mode_ok_without_tty() {
    // RED: Non-interactive should work without TTY
    let mode = TimelineMode::NonInteractive;
    let has_tty = false;

    let result = mode.validate_terminal_availability(has_tty);

    assert!(result.is_ok());
}

#[test]
fn test_interactive_mode_ok_with_tty() {
    // RED: Interactive should work with TTY
    let mode = TimelineMode::Interactive;
    let has_tty = true;

    let result = mode.validate_terminal_availability(has_tty);

    assert!(result.is_ok());
}

// ============================================================================
// Test 6: Mode Description
// ============================================================================

#[test]
fn test_interactive_mode_description() {
    // RED: Should provide human-readable description
    let mode = TimelineMode::Interactive;

    let desc = mode.description();

    assert!(desc.contains("interactive") || desc.contains("TUI"));
}

#[test]
fn test_non_interactive_mode_description() {
    // RED: Should provide human-readable description
    let mode = TimelineMode::NonInteractive;

    let desc = mode.description();

    assert!(desc.contains("non-interactive") || desc.contains("batch"));
}

// ============================================================================
// Test 7: Default Mode
// ============================================================================

#[test]
fn test_default_mode_is_non_interactive() {
    // RED: Should default to non-interactive mode
    let mode = TimelineMode::default();

    assert_eq!(mode, TimelineMode::NonInteractive);
}

// ============================================================================
// Test 8: CLI Flag Validation
// ============================================================================

#[test]
fn test_conflicting_flags_error() {
    // RED: Should error if both interactive and --json flags present
    let args = vec!["pmat", "timeline", "test.pmat", "--interactive", "--json"];
    let result = TimelineMode::validate_args(&args);

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.to_lowercase().contains("conflicting"));
}

#[test]
fn test_valid_interactive_only() {
    // RED: Interactive flag alone should be valid
    let args = vec!["pmat", "timeline", "test.pmat", "--interactive"];
    let result = TimelineMode::validate_args(&args);

    assert!(result.is_ok());
}

// ============================================================================
// Test 9: Integration with Timeline Command
// ============================================================================

#[cfg(feature = "tui")]
#[test]
fn test_timeline_command_with_interactive_flag() {
    // RED: Timeline command should accept --interactive flag
    let args = vec!["timeline", "test.pmat", "--interactive"];
    // This should not panic, even if it can't run (no TTY in test env)
    // Just verify flag is recognized
    let mode = TimelineMode::from_args(&args);
    assert_eq!(mode, TimelineMode::Interactive);
}

#[test]
fn test_timeline_command_default_non_interactive() {
    // RED: Timeline command should default to non-interactive
    let args = vec!["timeline", "test.pmat"];
    let mode = TimelineMode::from_args(&args);
    assert_eq!(mode, TimelineMode::NonInteractive);
}

// ============================================================================
// Test 10: Feature Gate
// ============================================================================

#[cfg(not(feature = "tui"))]
#[test]
fn test_interactive_mode_disabled_without_tui_feature() {
    // RED: Should error if interactive mode requested without tui feature
    let mode = TimelineMode::Interactive;

    let result = mode.check_feature_availability();

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("tui"));
}

#[cfg(feature = "tui")]
#[test]
fn test_interactive_mode_enabled_with_tui_feature() {
    // RED: Should work if tui feature is enabled
    let mode = TimelineMode::Interactive;

    let result = mode.check_feature_availability();

    assert!(result.is_ok());
}

// ============================================================================
// Test 11: Help Text
// ============================================================================

#[test]
fn test_interactive_flag_in_help_text() {
    // RED: --interactive should appear in help text
    use pmat::cli::get_timeline_help_text;

    let help = get_timeline_help_text();

    assert!(help.contains("--interactive") || help.contains("-i"));
    assert!(help.contains("TUI") || help.contains("interactive"));
}
