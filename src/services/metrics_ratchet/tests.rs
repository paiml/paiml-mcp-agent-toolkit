//! Falsification tests for the recursion guard and timeout added for GH #1324
//! (`pmat comply ratchet` fork-bomb: a metric's `command` that itself invokes
//! `pmat comply check` re-entered [`super::measure::measure_now`] with no
//! depth limit and no bound on how long the child could run).
//!
//! None of these tests spawn a recursive `pmat` — that is the exact bomb this
//! module exists to defuse. Every fixture command is `bash`, `printf` or
//! `sleep`, standing in for "a measurement's command started another
//! measurement" without ever doing so.
//!
//! `recursion_at_max_depth_is_refused_without_spawning` and
//! `a_command_exceeding_the_timeout_is_killed` use the thread-local test
//! seams (`set_test_depth_override`/`set_test_timeout_override_ms`) rather
//! than mutating `PMAT_RATCHET_DEPTH`/a real timeout env var directly:
//! `cargo test` runs many tests concurrently as OS threads in one process,
//! and a process-global env var mutated by one test would have been visible
//! to every other test's measurements running on other threads at the same
//! moment — including `drive_tests`/`coherence_drive_tests` in this same
//! crate, which this was tried against first and which it broke (a spurious
//! 1-second timeout killed their real, minutes-long `cargo clippy`
//! measurements). A thread-local is visible only to the thread that set it.

use super::config::Measurement;
use super::measure::{measure, set_test_depth_override, set_test_timeout_override_ms};
use std::time::{Duration, Instant};

/// A temp directory is never this crate's own repository, so [`measure`]'s
/// `#[cfg(test)]` memoization (keyed to `CARGO_MANIFEST_DIR`) never applies
/// here and every call reaches `measure_now` fresh.
fn scratch_dir() -> tempfile::TempDir {
    tempfile::tempdir().expect("tempdir")
}

/// Absent by default: an operator's shell never sets `PMAT_RATCHET_DEPTH`, so
/// a normal, top-level measurement must behave exactly as it did before this
/// guard existed.
#[test]
fn depth_env_absent_by_default_measures_normally() {
    let dir = scratch_dir();
    assert!(
        std::env::var("PMAT_RATCHET_DEPTH").is_err(),
        "no test in this crate sets the real PMAT_RATCHET_DEPTH env var"
    );
    let m = measure(dir.path(), "printf 42");
    assert_eq!(m, Measurement::Value(42));
}

/// A measurement's command may itself start another process (that is the
/// whole point of `command`), and that child sees the depth incremented by
/// exactly one — the mechanism [`super::measure::measure_now`] uses to refuse
/// the second level rather than the first.
#[test]
fn child_process_sees_depth_incremented() {
    let dir = scratch_dir();
    let m = measure(dir.path(), "printf '%s' \"$PMAT_RATCHET_DEPTH\"");
    assert_eq!(
        m,
        Measurement::Value(1),
        "a command run by a depth-0 measurement must see depth 1"
    );
}

/// The refusal itself: a measurement that is already at the maximum depth
/// (i.e. one that was started BY another measurement, per
/// `child_process_sees_depth_incremented` above) must not spawn anything at
/// all — it must report `Unavailable` and name both the refusal and the
/// command it refused.
///
/// "Must not spawn anything" is asserted directly, not inferred from the
/// return value: the refused command would create a marker file if it ran,
/// and this test asserts the file is absent after the call.
#[test]
fn recursion_at_max_depth_is_refused_without_spawning() {
    let dir = scratch_dir();
    let marker = dir.path().join("would-not-exist-if-refused");
    let command = format!("touch {}", marker.display());

    set_test_depth_override(Some(1));
    let m = measure(dir.path(), &command);
    set_test_depth_override(None);

    assert!(
        matches!(m, Measurement::Unavailable(_)),
        "expected Unavailable at max depth, got {m:?}"
    );
    let Measurement::Unavailable(msg) = m else {
        unreachable!("checked by the assert! immediately above")
    };
    assert!(
        msg.contains("refused") && msg.contains("recursion"),
        "message must name the refusal, got: {msg}"
    );
    assert!(
        msg.contains(&command) || msg.contains("touch"),
        "message must name the offending command, got: {msg}"
    );
    assert!(
        msg.contains("PMAT_RATCHET_DEPTH"),
        "message must name the guard env var, got: {msg}"
    );
    assert!(
        !marker.exists(),
        "the refused command must never have been spawned"
    );
}

/// A command that outlives the (test-only, short) measurement timeout is
/// killed and reported `Unavailable` rather than hanging the caller forever.
/// The override only has an effect on the thread that sets it, and only
/// under `#[cfg(test)]`, so this cannot shorten a real `pmat comply ratchet`
/// run.
#[test]
fn a_command_exceeding_the_timeout_is_killed() {
    let dir = scratch_dir();

    set_test_timeout_override_ms(Some(1000));
    let m = measure(dir.path(), "sleep 5 && printf 1");
    set_test_timeout_override_ms(None);

    assert!(
        matches!(m, Measurement::Unavailable(_)),
        "expected Unavailable on timeout, got {m:?}"
    );
    let Measurement::Unavailable(msg) = m else {
        unreachable!("checked by the assert! immediately above")
    };
    assert!(
        msg.contains("timeout") && msg.contains("killed"),
        "message must name the timeout and the kill, got: {msg}"
    );
}

/// PR-review finding on GH #1324 (round 1): `child.kill()` only signals the
/// immediate `bash`. A command that backgrounds work with `&` leaves that
/// work as a SEPARATE process holding the inherited stdout/stderr pipe open,
/// so killing only `bash` does not close the pipe — the read side blocks in
/// `read_to_end` for as long as that descendant runs, which for a real fork
/// bomb is unbounded. This is the "the timeout meant to stop a runaway hangs
/// on exactly the runaway it exists for" bug.
///
/// This variant keeps a FOREGROUND `sleep 30` after the background job, so
/// `bash` itself is still running when the deadline fires — `try_wait`
/// returns `Ok(None)` until then, and the fix under test is specifically the
/// TIMEOUT arm's group-kill. See
/// `a_grandchild_holding_the_pipe_is_killed_when_the_shell_exits_first` for
/// the companion case where `bash` exits immediately instead (round 2's
/// finding: that case never reaches this arm at all, because nothing times
/// out — `try_wait` reports success right away).
///
/// Neither side of this fixture would ever complete inside the test's
/// timeout budget on its own — only killing the whole process group can make
/// `measure` return here, and only killing it PROMPTLY (not merely
/// "eventually, when the 30s elapse on their own") passes the wall-clock
/// assertion below.
///
/// Before the process-group fix this test hung indefinitely (verified with
/// an external `timeout(1)` wrapper against the pre-fix `measure.rs`, which
/// reported exit 124 — SIGTERM from `timeout(1)`, not a completed test run).
/// After the fix it returns in well under a second.
#[test]
fn a_grandchild_holding_the_pipe_is_killed_when_the_shell_stays_alive() {
    let dir = scratch_dir();
    let marker = dir.path().join("grandchild-should-never-write-this");
    let command = format!("(sleep 30; touch {}) & sleep 30", marker.display());

    set_test_timeout_override_ms(Some(300));
    let start = Instant::now();
    let m = measure(dir.path(), &command);
    let elapsed = start.elapsed();
    set_test_timeout_override_ms(None);

    assert!(
        elapsed < Duration::from_secs(5),
        "measure() must return promptly even when a backgrounded grandchild \
         inherited the pipe and outlives the timeout kill of the immediate \
         child; took {elapsed:?}"
    );
    assert!(
        matches!(m, Measurement::Unavailable(_)),
        "expected Unavailable on timeout, got {m:?}"
    );
    assert!(
        !marker.exists(),
        "the backgrounded grandchild must have been killed by the process-group \
         kill, not left to run to completion and write its marker"
    );
}

/// PR-review finding on GH #1324 (round 2): the round-1 fix above only
/// bounded the TIMEOUT and error arms. If the command backgrounds work and
/// `bash` then exits on its own — `(sleep 30; touch marker) &` with no
/// trailing foreground command — `try_wait` returns `Ok(Some(status))`
/// immediately, the SUCCESS arm is taken, and nothing ever times out. `bash`
/// exiting closes only ITS end of the pipe; the backgrounded grandchild
/// inherited the write end and keeps it open for the length of its own
/// sleep, so a plain blocking read to EOF on that arm hung just as
/// unboundedly as `child.kill()` did on the timeout arm — with no timeout
/// anywhere in the call stack to blame.
///
/// Reproduced independent of this module's code, per the PR review:
/// `bash -c 'bash -c "(sleep 8) &" | cat'` takes 8 seconds — the outer
/// `bash` exits instantly, but `cat` blocks until the grandchild's `sleep`
/// closes the pipe on its own.
///
/// This is deliberately the SAME fixture as
/// `..._when_the_shell_stays_alive` minus the trailing foreground `sleep
/// 30`, so that dropping one thing (not adding one) is what moves the run
/// from the timeout arm to the success arm — the round-1 test's own shape
/// could not have caught this, precisely because it always exercised the
/// timeout arm.
///
/// PR-review finding, round 3 (this is now the CURRENT behaviour, not the
/// fix history above): the success arm must NOT signal the process group on
/// drain expiry, unlike the timeout/error arms. By the time `try_wait`
/// returns `Ok(Some(status))`, the leader has already been reaped — its pid
/// is free, and the OS may have already handed it to an unrelated process by
/// the time the drain times out. Killing `-pid` there would not be "kill our
/// stray descendant", it would be "kill whatever process group now holds
/// this recycled id" — a gate that might kill a stranger's work is worse
/// than the hang it replaced. So this arm reports `Unavailable` on drain
/// expiry but leaves the descendant running; the marker file below is
/// asserted to EVENTUALLY appear, proving it, where round 2's version of
/// this test asserted the opposite (that it never would).
///
/// The 3-second delay is deliberately just past `DRAIN_GRACE` (2s): long
/// enough that the drain reliably expires (exercising the fixed code path)
/// and `measure()` returns before the grandchild finishes, short enough that
/// polling for the marker afterward does not make this test slow.
///
/// Both versions of this fixture were run RED before their respective fixes
/// and GREEN after; see the commit message for round 3's timings.
#[test]
fn a_grandchild_holding_the_pipe_is_left_alone_when_the_shell_exits_first() {
    let dir = scratch_dir();
    let marker = dir.path().join("grandchild-writes-this-once-left-alone");
    let command = format!("(sleep 3; touch {}) &", marker.display());

    let start = Instant::now();
    let m = measure(dir.path(), &command);
    let elapsed = start.elapsed();

    assert!(
        elapsed < Duration::from_secs(5),
        "measure() must return promptly (bounded by DRAIN_GRACE), even \
         though `bash` itself exits immediately and a backgrounded \
         grandchild is left holding the pipe; took {elapsed:?}"
    );
    assert!(
        matches!(m, Measurement::Unavailable(_)),
        "expected Unavailable when the drain does not complete within \
         DRAIN_GRACE, got {m:?}"
    );
    assert!(
        !marker.exists(),
        "measure() only just returned (~DRAIN_GRACE) and the grandchild's \
         own 3s sleep has not elapsed yet — if this fires, the timing \
         assumption behind this test (delay > DRAIN_GRACE) has rotted"
    );

    // Not signalled, so left to run to completion on its own: poll (rather
    // than a fixed sleep) so this assertion is not itself racing the
    // grandchild's clock.
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline && !marker.exists() {
        std::thread::sleep(Duration::from_millis(50));
    }
    assert!(
        marker.exists(),
        "the backgrounded grandchild must have been left alone to finish, \
         not signalled — by the time the drain expired its pid had already \
         been reaped and was free to be reused, so killing it would risk \
         killing an unrelated process rather than our own descendant"
    );
}
