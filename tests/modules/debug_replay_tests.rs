// DEBUG-003: Replay CLI Handler Tests
// Sprint 74 - RED Phase
//
// Tests for `pmat debug replay` command implementation
// These tests drive the creation of replay handler and Timeline UI integration
//
// `handle_debug_replay` now exits the process with
// DEBUG_UNIMPLEMENTED_EXIT_CODE (2) instead of returning `Err`, matching
// `pmat serve`'s honest-failure policy. Calling it from a test would take the
// whole test binary down with it, and the assertions here were
// `result.is_ok() || result.is_err()` — true for every possible Result, so they
// never checked anything. They now pin the signature and the diagnostic, which
// is what "the handler exists and is callable with these parameters" was
// trying to say.

use std::path::PathBuf;
use tempfile::NamedTempFile;

/// The handler exists with the documented signature.
#[test]
fn test_replay_handler_exists() {
    // Building the future type-checks the signature without running it.
    let recording = PathBuf::from("test_recording.pmat");
    let _future = pmat::cli::handlers::debug_handlers::handle_debug_replay(recording, None, false);
}

/// The handler accepts a recording path, a `--position` and `--interactive`.
#[test]
fn test_replay_accepts_position_and_interactive() {
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let recording = temp_file.path().to_path_buf();
    std::fs::write(&recording, b"mock_recording_data").expect("Failed to write mock data");

    // Building the future type-checks every parameter without running it — the
    // handler exits the process, so it must not be awaited here.
    let _future =
        pmat::cli::handlers::debug_handlers::handle_debug_replay(recording.clone(), Some(5), true);
    let _future = pmat::cli::handlers::debug_handlers::handle_debug_replay(recording, None, false);
}

/// The diagnostic names DEBUG-003 and says plainly that nothing replays yet.
///
/// Replaces "Handler should attempt to display timeline", which asserted
/// nothing: no timeline is displayed and none was checked for.
#[test]
fn test_replay_reports_it_is_not_implemented() {
    let mut buf = Vec::new();
    pmat::cli::handlers::debug_handlers::write_debug_unimplemented_message(
        &mut buf,
        "replay",
        "DEBUG-003",
        "recording=\"/tmp/recording.pmat\" position=Some(5) interactive=true",
    )
    .expect("write");
    let s = String::from_utf8(buf).expect("utf8");
    assert!(s.contains("DEBUG-003"), "got: {s}");
    assert!(s.contains("not implemented"), "got: {s}");
    assert_eq!(
        pmat::cli::handlers::debug_handlers::DEBUG_UNIMPLEMENTED_EXIT_CODE,
        2,
        "unimplemented must exit 2 (misuse), not 1"
    );
}
