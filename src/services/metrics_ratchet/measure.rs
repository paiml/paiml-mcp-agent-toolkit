//! Running the pinned command of every declared metric.
//!
//! [`super::config`] is pure by design and takes measurements as an argument.
//! This is the module that produces them, and it is where every fail-closed
//! decision about measurement lives: a command that does not run, exits in a
//! way the grep family does not use, prints nothing, or prints something that
//! is not a count becomes [`Measurement::Unavailable`] — never a zero.
//!
//! That distinction is the whole reason `command` is a field of
//! [`super::config::MetricBaseline`] rather than a sentence in a commit
//! message. The scope predicate IS the metric: "the unwrap count of this
//! repository" has been quoted as 570, 11,002, 20,326 and 20,378 within one
//! programme of work by people who each meant a different set of files, two of
//! those differ by 9,324, and one moved by 52 inside a single session. A
//! number nobody can recompute has already rotted.

use super::config::{Measurement, Measurements, MetricBaseline};
use std::collections::BTreeMap;
use std::io::Read;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

/// `grep` and `git grep` exit 1 to mean "no matches", which is a legitimate
/// count of zero, not an error. Anything else — 2 from a bad pathspec, 127
/// from a missing binary, 128 from git — is a broken measurement.
const ACCEPTABLE_EXIT_CODES: [i32; 2] = [0, 1];

/// The environment variable that carries how many measurements deep the
/// current process is nested inside other measurements. Read by
/// [`measure_now`] before it spawns anything, and set on every child it does
/// spawn, so the depth travels with the process tree rather than living in
/// any one process's memory.
///
/// This exists because of GH #1324: `pmat comply ratchet` → `measure_now` runs
/// `bash -c <command>`, and a metric whose `command` itself invokes
/// `pmat comply check` re-enters this same function with no guard, fanning out
/// by the number of ratchet entries at every level. On a 48-core machine that
/// reached a 1-minute load average of 3,026 with 9,740 processes.
const RATCHET_DEPTH_ENV: &str = "PMAT_RATCHET_DEPTH";

/// A measurement started directly by `pmat comply ratchet`/`check` runs at
/// depth 0 and is allowed. A measurement whose OWN command starts another
/// measurement would run at depth 1, and that is refused: a metric may be
/// measured, but a measurement may not measure. One level is enough to
/// describe every legitimate case (there is none) and it is the smallest
/// value that still lets the depth-0 case through.
const RATCHET_MAX_DEPTH: u32 = 1;

/// How long a single metric's command may run before [`measure_now`] kills it
/// and reports `Unavailable`. A ratchet measurement is a `grep`/`git grep`
/// pipeline or, at the expensive end, a single `cargo` invocation over an
/// already-built tree (this repo's own compiler-derived metric is ~40s); 300s
/// is generous enough to absorb a cold cache or a slow CI runner without ever
/// being generous enough to let a runaway process (recursive or otherwise)
/// sit unbounded.
const MEASUREMENT_TIMEOUT: Duration = Duration::from_secs(300);

/// How often [`measure_now`]'s wait loop polls the child for completion.
const POLL_INTERVAL: Duration = Duration::from_millis(20);

/// How long the timeout/error paths wait for the two pipe readers once the
/// tree has been signalled.
///
/// Killing the whole process group (see [`kill_process_tree`]) closes every
/// write end of both pipes, so the readers normally reach EOF at once and
/// this budget is not spent. It exists for the case a descendant dodged the
/// group kill by calling `setsid` for itself and is still holding a pipe
/// open: `Receiver::recv` has no timeout of its own, and waiting on such a
/// reader unconditionally would trade a leaked thread for a caller that
/// itself never returns — the exact bug (GH #1324 PR review) this constant
/// closes. Modelled on `DRAIN_GRACE` in
/// `cli/handlers/work_falsification/deny_refresh.rs`.
const DRAIN_GRACE: Duration = Duration::from_secs(2);

/// Measure every declared metric by running its own `command`.
pub fn measure_all(
    project_path: &Path,
    metrics: &BTreeMap<String, MetricBaseline>,
) -> Measurements {
    metrics
        .iter()
        .map(|(id, m)| (id.clone(), measure_metric(project_path, m)))
        .collect()
}

/// Run one metric's command and apply the zero guard.
///
/// The guard exists because the exit-code guard below cannot reach the most
/// likely way for a pinned command to rot. `measure` protects the shape it
/// documents — a producer that FAILS inside a pipeline — and a pathspec that
/// has stopped matching any file is a producer that SUCCEEDS over nothing:
///
/// ```text
/// git grep -oF 'TOKEN' -- 'no/such/path/*.rs' | wc -l   ->  0, exit 1, no stderr
/// git grep -oF 'NOT_PRESENT' -- 'src/*.rs'    | wc -l   ->  0, exit 1, no stderr
/// ```
///
/// Byte-identical. (`TOKEN` stands in for a real pattern on purpose: this
/// repository ratchets an `.unwrap` call literal with a `git grep -oF` of exactly
/// this shape, and writing that literal out here would have moved the number this guard
/// protects by three — the metric counts occurrences in prose about itself. The
/// ratchet caught that on the commit that introduced the guard.) So the only place the two can be distinguished is against
/// the baseline, and only by a human: a drop from N to 0 in one run is either
/// the largest improvement in the project's history or a broken predicate. The
/// gate refuses to guess, and says which two things it is choosing between.
///
/// This was measured, not imagined: before the guard, editing one metric's
/// pathspec to `no/such/path/*.rs` made `pmat comply coherence` report
/// `FIRING  measured 0 count against limit 100` and exit 0, while the ratchet
/// read `0 <= 20390` as a Pass. Both gates went green on a metric that had
/// stopped measuring anything at all.
pub fn measure_metric(project_path: &Path, metric: &MetricBaseline) -> Measurement {
    let raw = match metric.analyzer.as_deref() {
        Some(name) => measure_analyzer(project_path, name),
        None => measure(project_path, &metric.command),
    };
    guard_zero(raw, metric)
}

/// Measure by calling an analyzer this crate contains, in-process.
///
/// No `pmat` is spawned: a `command` that ran `pmat` would resolve whichever
/// pmat is first on `$PATH` (CRUX-19's defect, in a gate about honesty), and
/// under `cargo test --lib` in CI there is none. The `command` field still
/// carries the shell reproduction, and `drive_tests` cross-checks the two on a
/// tree where both can run. An unknown analyzer name is `Unavailable` — a
/// metric that names something this build cannot measure has rotted, and must
/// not read as zero.
fn measure_analyzer(project_path: &Path, name: &str) -> Measurement {
    match name {
        "reachability.orphan_count" | "reachability.quarantined_count" => {
            use crate::services::reachability;
            let (roots, tracked) = match reachability::discover(project_path) {
                Ok(v) => v,
                Err(e) => {
                    return Measurement::Unavailable(format!(
                        "reachability discovery failed under {}: {e}",
                        project_path.display()
                    ))
                }
            };
            if roots.is_empty() {
                return Measurement::Unavailable(format!(
                    "no cargo targets found under {} — reachability could not be measured",
                    project_path.display()
                ));
            }
            let report = reachability::analyze(project_path, &roots, &tracked);
            let n = if name == "reachability.orphan_count" {
                report.orphans.len()
            } else {
                report.quarantined.len()
            };
            match i64::try_from(n) {
                Ok(v) => Measurement::Value(v),
                Err(_) => Measurement::Unavailable("count does not fit in i64".into()),
            }
        }
        other => Measurement::Unavailable(format!(
            "unknown analyzer `{other}` — this build has no in-process measurement by that name"
        )),
    }
}

/// Turn an unexplained zero into an `Unavailable`. Pure, so the falsification
/// tests can drive both sides of it without a shell.
pub fn guard_zero(raw: Measurement, metric: &MetricBaseline) -> Measurement {
    match raw {
        Measurement::Value(0) if metric.baseline > 0 && !metric.zero_is_reachable => {
            Measurement::Unavailable(format!(
                "measured 0 against a baseline of {} — either every occurrence was removed in \
                 one change or the command has stopped matching anything, and a count cannot \
                 tell those apart; re-run the command by hand and, if the zero is real, set \
                 `zero_is_reachable = true` on this metric",
                metric.baseline
            ))
        }
        other => other,
    }
}

/// Run `command` from `project_path` and read a count from its output.
///
/// The shell is `bash -o pipefail`. Both halves matter. `sh` is `dash` on
/// Debian and does not reliably support `pipefail`; and without `pipefail` the
/// canonical `<producer> | wc -l` shape reports `wc`'s status, so a producer
/// that failed outright reads as a clean `0` — which a ratchet, looking only
/// upward, greets as the largest improvement in the project's history and the
/// lowering job then makes permanent.
pub fn measure(project_path: &Path, command: &str) -> Measurement {
    #[cfg(test)]
    if is_own_repo(project_path) {
        // Take the map lock only long enough to hand out this command's cell,
        // then release it and initialise the cell outside. A lock held across
        // the subprocess would serialise unrelated commands; a lock NOT held
        // across it lets every thread miss simultaneously, which is what the
        // first version of this memo did — six callers raced, all missed, and
        // the suite got 11% faster instead of 6x.
        let cell = {
            let mut map = REPO_MEASUREMENTS
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            std::sync::Arc::clone(map.entry(command.to_owned()).or_default())
        };
        return cell
            .get_or_init(|| measure_now(project_path, command))
            .clone();
    }
    measure_now(project_path, command)
}

/// Test-only memo for measurements taken against THIS repository.
///
/// `cargo test --lib` reaches the committed ratchet from six independent call
/// sites — three `run_coherence`, one `run`, one per-metric `measure` loop and
/// one `measure_all` — and each used to re-run every command from scratch. With
/// a compiler-derived metric in the set that is ~40s a pass, so the suite spent
/// roughly four minutes measuring the same unchanged tree six times, on each of
/// the eight feature legs CI runs.
///
/// Keyed on the command text, which IS the metric's identity here: two metrics
/// sharing a command must measure the same thing, or one of them is misdeclared.
///
/// Scoped to this repository on purpose. Fixture-driven tests build a temp dir,
/// measure it, mutate it and measure again — caching those would make a test
/// observe a tree that no longer exists. The repository's own working tree is
/// not mutated by the suite, so within one process its measurements are stable.
///
/// Each command gets its own `OnceLock` so a second caller BLOCKS on the first
/// rather than duplicating a 40-second compile: `get_or_init` is what turns six
/// concurrent identical measurements into one.
#[cfg(test)]
type MeasurementCell = std::sync::Arc<std::sync::OnceLock<Measurement>>;

#[cfg(test)]
static REPO_MEASUREMENTS: std::sync::LazyLock<
    std::sync::Mutex<std::collections::HashMap<String, MeasurementCell>>,
> = std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));

/// True only for the repository this crate is compiled from, resolved through
/// symlinks so a caller passing a different spelling of the same path is not
/// treated as a separate tree.
#[cfg(test)]
fn is_own_repo(project_path: &Path) -> bool {
    let here = std::fs::canonicalize(env!("CARGO_MANIFEST_DIR")).ok();
    let there = std::fs::canonicalize(project_path).ok();
    here.is_some() && here == there
}

// Test-only, thread-local overrides for `current_depth` and
// `measurement_timeout`.
//
// `cargo test` runs many tests concurrently as OS threads inside one
// process, and `std::env::set_var` is process-global — a test that wants to
// simulate "already at max recursion depth" or "a 1-second timeout" by
// mutating the real environment variable would leak that state into every
// other test's measurements running on other threads at the same moment
// (this was tried first, and it killed unrelated `drive_tests`/
// `coherence_drive_tests` measurements with a spurious 1s timeout). A
// thread-local is visible only to the thread that set it, so one test
// exercising the guard cannot affect another test's measurement.
//
// Production code never sets this; only `#[cfg(test)]` code (this module's
// own tests) may.
#[cfg(test)]
thread_local! {
    static TEST_DEPTH_OVERRIDE: std::cell::Cell<Option<u32>> = const { std::cell::Cell::new(None) };
    static TEST_TIMEOUT_OVERRIDE_MS: std::cell::Cell<Option<u64>> = const { std::cell::Cell::new(None) };
}

/// Make every measurement on the CALLING THREAD behave as though
/// [`RATCHET_DEPTH_ENV`] were set to `depth`, without touching the real
/// process environment. `None` clears the override. `#[cfg(test)]` only.
#[cfg(test)]
pub(crate) fn set_test_depth_override(depth: Option<u32>) {
    TEST_DEPTH_OVERRIDE.with(|cell| cell.set(depth));
}

/// Make every measurement on the CALLING THREAD use `ms` milliseconds instead
/// of [`MEASUREMENT_TIMEOUT`], without touching the real process environment.
/// `None` clears the override. `#[cfg(test)]` only.
#[cfg(test)]
pub(crate) fn set_test_timeout_override_ms(ms: Option<u64>) {
    TEST_TIMEOUT_OVERRIDE_MS.with(|cell| cell.set(ms));
}

/// How many measurements deep the current process is nested, read from
/// [`RATCHET_DEPTH_ENV`]. Absent or non-numeric reads as 0 — the depth-0,
/// top-level case — rather than refusing to measure at all, because an
/// operator's shell will never have this variable set.
fn current_depth() -> u32 {
    #[cfg(test)]
    {
        if let Some(d) = TEST_DEPTH_OVERRIDE.with(std::cell::Cell::get) {
            return d;
        }
    }
    std::env::var(RATCHET_DEPTH_ENV)
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0)
}

/// A short, single-line reproduction of `command` for an error message: long
/// enough to identify the offending metric, short enough that a fork-bomb's
/// own repeated command text does not itself flood the log.
fn truncate_command(command: &str) -> String {
    const MAX: usize = 160;
    let flat: String = command.split_whitespace().collect::<Vec<_>>().join(" ");
    if flat.chars().count() <= MAX {
        flat
    } else {
        let head: String = flat.chars().take(MAX).collect();
        format!("{head}…")
    }
}

/// The wall-clock budget for one measurement. Overridable only under
/// `#[cfg(test)]`, and only via the thread-local set by
/// [`set_test_timeout_override_ms`], so a test can assert the kill path
/// without waiting out [`MEASUREMENT_TIMEOUT`] and without affecting any
/// other test's measurements.
fn measurement_timeout() -> Duration {
    #[cfg(test)]
    {
        if let Some(ms) = TEST_TIMEOUT_OVERRIDE_MS.with(std::cell::Cell::get) {
            return Duration::from_millis(ms);
        }
    }
    MEASUREMENT_TIMEOUT
}

/// Drain one of the child's pipes on its own thread, delivering the bytes
/// back over a channel rather than a `JoinHandle`, because only a channel can
/// be waited on with a deadline (`Receiver::recv_timeout`); see
/// [`DRAIN_GRACE`]. Modelled on `drain_on_thread` in
/// `cli/handlers/work_falsification/deny_refresh.rs`.
fn drain_on_thread<R: Read + Send + 'static>(pipe: Option<R>) -> Receiver<Vec<u8>> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut buf = Vec::new();
        if let Some(mut p) = pipe {
            let _ = p.read_to_end(&mut buf);
        }
        let _ = tx.send(buf);
    });
    rx
}

/// Give the child a process group of its own, so its whole subtree can be
/// signalled with a single call instead of only its immediate pid.
///
/// `process_group(0)` is `setpgid(0, 0)` performed between fork and exec, so
/// the child becomes the leader of a fresh group whose id is its own pid, and
/// everything it spawns (including anything it merely backgrounds with `&`)
/// inherits that group. See `lead_new_process_group` in
/// `cli/handlers/work_falsification/deny_refresh.rs` for the fuller
/// discussion, including the interactive-Ctrl-C trade this makes.
#[cfg(unix)]
fn lead_new_process_group(cmd: &mut Command) {
    use std::os::unix::process::CommandExt;
    cmd.process_group(0);
}

/// Windows has no POSIX process group to lead.
#[cfg(not(unix))]
fn lead_new_process_group(_cmd: &mut Command) {}

/// Kill the child *and everything it spawned*.
///
/// `Child::kill` signals only the direct `bash` process. A measurement's
/// command can background work with `&`, and a fork bomb is nothing but such
/// descendants — killing only `bash` and returning left the runaway subtree
/// running and unowned, and left this function's own readers blocked in
/// `read_to_end` on a pipe a grandchild still held open, i.e. hung forever on
/// exactly the case this guard exists for. `lead_new_process_group` has
/// already put the child at the head of its own group, so its pgid equals its
/// pid and one signal reaches the whole subtree — except a descendant that
/// deliberately left the group itself (e.g. by calling `setsid`), which
/// [`DRAIN_GRACE`] bounds rather than promises to reach.
#[cfg(unix)]
fn kill_process_tree(child: &mut Child) {
    // Annotated `try_into` rather than `libc::pid_t::try_from`: `pid_t` is a
    // type alias, and an annotated conversion resolves the same way on every
    // target whatever the alias points at.
    let pid: Result<libc::pid_t, _> = child.id().try_into();
    match pid {
        Ok(pid) => {
            // SAFETY: `kill` takes two integers by value and returns one; it
            // dereferences no pointer, so there is no allocation, aliasing or
            // initialisation obligation for a caller to uphold. A negative
            // first argument is POSIX's spelling of "every process in the
            // group whose id is its absolute value", and that group is the
            // one `lead_new_process_group` created for this child, so the
            // blast radius is exactly this child and its descendants. The
            // result is discarded for the same reason `child.kill()`'s was:
            // by the time the deadline fires the child may already have
            // exited.
            let _ = unsafe { libc::kill(-pid, libc::SIGKILL) };
        }
        // Unreachable in practice — pids are bounded far below `pid_t::MAX`
        // — but an `as` cast here would wrap a pid that did not fit into a
        // *different* group id and signal strangers. Narrowing the kill to
        // the direct child is the safe way to be wrong.
        Err(_) => {
            let _ = child.kill();
        }
    }
}

/// Windows keeps the direct-child-only behaviour: bounding a whole tree there
/// needs a Job Object, which needs a crate this build does not carry (`libc`
/// is declared for `cfg(unix)` only).
#[cfg(not(unix))]
fn kill_process_tree(child: &mut Child) {
    let _ = child.kill();
}

/// Run one command and classify its result. Every fail-closed decision lives
/// here; [`measure`] adds only the test-time memo.
///
/// Two guards sit in front of the actual spawn (GH #1324): a recursion depth
/// check, because a metric's command that itself invokes a measurement would
/// otherwise fan out without bound, and a wall-clock timeout on the child
/// once spawned, because `Command::output()` alone blocks forever on a
/// command that never exits (recursive or not). The timeout leads the child
/// into its own process group ([`lead_new_process_group`]) and, on firing,
/// kills the whole group ([`kill_process_tree`]) rather than only the direct
/// `bash` pid: a fork bomb IS its descendants, and killing only the
/// immediate child left a grandchild holding the stdout/stderr pipe open,
/// which hung the drain forever — the exact case this function exists to
/// bound. The wait loop's `Err` arm does the same cleanup: dropping a
/// `Child` on Unix does not kill it, so returning without signalling the
/// group there would have leaked the same way.
///
/// The SUCCESS path gets the identical bounded drain (PR #1327 round 2): a
/// command that backgrounds work and then exits — `bash -c '(sleep 300) &'`
/// — makes `try_wait` return `Ok(Some(status))` immediately, long before the
/// backgrounded grandchild does. `bash` exiting closes only ITS end of the
/// pipe; the grandchild inherited the write end and keeps it open, so a plain
/// `read_to_end` to EOF never returns. The timeout guard above never even
/// fires here, because nothing timed out — `bash` exited cleanly. Bounding
/// the drain on this arm too is what lets this function always return.
///
/// That arm does NOT kill the group, unlike the two above (PR #1327 round
/// 3): `try_wait` returning `Ok(Some(status))` means the leader has already
/// been reaped, so its pid is free and the OS may have already handed it to
/// an unrelated process by the time a drain expires. Signalling `-pid` there
/// would not be "kill our stray descendant", it would be "kill whatever
/// process group now holds this recycled id" — worse than the hang it
/// replaces. The bound (not a signal) is what protects this arm; see the
/// comment at the success-arm drain below for the full argument.
fn measure_now(project_path: &Path, command: &str) -> Measurement {
    if command.trim().is_empty() {
        return Measurement::Unavailable(
            "the metric declares no command, so its baseline cannot be reproduced".into(),
        );
    }

    let depth = current_depth();
    if depth >= RATCHET_MAX_DEPTH {
        return Measurement::Unavailable(format!(
            "refused to run a ratchet measurement at recursion depth {depth} (max {RATCHET_MAX_DEPTH}); \
             this command was itself started by another measurement, which is the fork-bomb GH #1324 \
             guards against — set {RATCHET_DEPTH_ENV} only if you understand that risk. Refused command: {}",
            truncate_command(command)
        ));
    }

    let mut cmd = Command::new("bash");
    cmd.arg("-o")
        .arg("pipefail")
        .arg("-c")
        .arg(command)
        .current_dir(project_path)
        // Deterministic collation and message text: a metric that greps must
        // not depend on the locale of whoever ran it.
        .env("LC_ALL", "C")
        // Carries the recursion guard to any measurement this command starts.
        .env(RATCHET_DEPTH_ENV, (depth + 1).to_string())
        // A metric must measure the tree, not the shell that happened to invoke
        // it. These five can silently change what a nested cargo compiles — and
        // therefore what a compiler-derived metric counts — when the gate runs
        // under another cargo. CARGO_TARGET_DIR is deliberately NOT removed:
        // the commands that need isolation set it inline, which is what keeps a
        // nested cargo from blocking on the outer invocation's target-dir lock.
        .env_remove("RUSTFLAGS")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env_remove("CARGO_BUILD_RUSTFLAGS")
        .env_remove("CARGO_BUILD_TARGET")
        .env_remove("CARGO_BUILD_JOBS")
        // A command in a background process group that reads the terminal
        // takes SIGTTIN and stops, and a stopped child is invisible to
        // `try_wait` — it would sit there until the deadline. No metric
        // command needs input.
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    lead_new_process_group(&mut cmd);
    let child = cmd.spawn();

    let mut child = match child {
        Ok(c) => c,
        Err(e) => return Measurement::Unavailable(format!("could not run bash: {e}")),
    };

    // Drain stdout/stderr on background threads concurrently with the wait
    // loop below, the same way `Command::output()` does internally — a child
    // that writes more than one pipe buffer's worth would otherwise block on
    // write() while we are only polling `try_wait`, deadlocking the wait.
    // The bytes come back over a channel (not a plain `JoinHandle`) so the
    // timeout/error paths below can bound how long they wait for them; see
    // `DRAIN_GRACE`.
    let out_rx = drain_on_thread(child.stdout.take());
    let err_rx = drain_on_thread(child.stderr.take());

    let timeout = measurement_timeout();
    let deadline = Instant::now() + timeout;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) => {
                if Instant::now() >= deadline {
                    break None;
                }
                std::thread::sleep(POLL_INTERVAL);
            }
            Err(e) => {
                // Same cleanup as the timeout arm below: signal the group,
                // reap, bound the drain, then report. Dropping `child` here
                // without doing that would leave it (and anything it
                // spawned) running, unowned, forever.
                kill_process_tree(&mut child);
                let _ = child.wait();
                let _ = out_rx.recv_timeout(DRAIN_GRACE);
                let _ = err_rx.recv_timeout(DRAIN_GRACE);
                return Measurement::Unavailable(format!("could not wait on bash: {e}"));
            }
        }
    };

    let Some(status) = status else {
        // Timed out: signal the whole process group, not just `bash`, so a
        // backgrounded descendant that inherited the pipe is killed too.
        // Reap before draining — a killed-but-unreaped leader is still a
        // valid group for the signal to have reached, and reaping first
        // could let its pid be reused. Bound the drain with `DRAIN_GRACE`
        // rather than joining unconditionally: a descendant that left the
        // group itself (`setsid`) may still hold a pipe open, and this
        // function must always return rather than hang on that one
        // survivor — the reader thread is left detached in that case, not
        // joined, which trades a leaked thread for a caller that returns.
        kill_process_tree(&mut child);
        let _ = child.wait();
        let _ = out_rx.recv_timeout(DRAIN_GRACE);
        let _ = err_rx.recv_timeout(DRAIN_GRACE);
        return Measurement::Unavailable(format!(
            "command exceeded the {}s measurement timeout and was killed: {}",
            timeout.as_secs(),
            truncate_command(command)
        ));
    };

    // Normal exit: `bash` itself is done, but anything it backgrounded with
    // `&` does not have to be — `bash` exiting closes only ITS end of the
    // pipe, and a grandchild that inherited the write end keeps it open
    // until IT exits. Blocking here unconditionally (as `Command::output()`
    // does) hangs on exactly that shape, with no timeout ever in play,
    // because nothing timed out: `bash` exited cleanly. So the drain is
    // bounded on this arm too, identically to the timeout/error arms above.
    //
    // Unlike those arms, THIS one must never call `kill_process_tree`.
    // `try_wait` returning `Ok(Some(status))` above means the leader has
    // already been reaped — its pid is free, and the OS is free to hand it
    // to an unrelated process the moment that happens. Signalling `-pid`
    // after that point is not "kill our stray descendant", it is "kill
    // whatever process group now holds this recycled id", which on a busy
    // machine can be someone else's work entirely. The timeout and `Err`
    // arms above are safe to signal because they break out of the loop
    // (`None`) or hit `try_wait`'s `Err` arm BEFORE any reap has happened —
    // the pid is still provably ours there. Here it is not, so the bound
    // alone (not a signal) is what protects this function: a drain that
    // does not complete within `DRAIN_GRACE` is reported `Unavailable` and
    // the descendant is simply left running, unsignalled. The depth guard
    // above is what keeps a recursive measurement from spawning in the
    // first place; this bound is what keeps a non-recursive but
    // long-lived descendant from hanging the caller. Neither is "kill the
    // leftover" — there is no pid left that this function may safely
    // touch.
    //
    // A drain that does not complete within `DRAIN_GRACE` is reported
    // `Unavailable` even though `bash`'s own exit was clean: the reader
    // thread sends its buffer exactly once, at EOF, so an expired
    // `recv_timeout` means zero bytes are available to this function — there
    // is no partial buffer to fall back to — and inventing a count from an
    // incomplete read would be exactly the silent-zero failure mode
    // `guard_zero` elsewhere in this module exists to refuse. Fail closed
    // instead, the same way every other unreadable-output case here does.
    let stdout = match out_rx.recv_timeout(DRAIN_GRACE) {
        Ok(buf) => buf,
        Err(_) => {
            return Measurement::Unavailable(format!(
                "bash exited but a descendant it backgrounded kept stdout open past the {}s drain grace period; its pid is no longer ours to signal, so it was left running: {}",
                DRAIN_GRACE.as_secs(),
                truncate_command(command)
            ));
        }
    };
    let stderr = match err_rx.recv_timeout(DRAIN_GRACE) {
        Ok(buf) => buf,
        Err(_) => {
            return Measurement::Unavailable(format!(
                "bash exited but a descendant it backgrounded kept stderr open past the {}s drain grace period; its pid is no longer ours to signal, so it was left running: {}",
                DRAIN_GRACE.as_secs(),
                truncate_command(command)
            ));
        }
    };

    match status.code() {
        Some(c) if ACCEPTABLE_EXIT_CODES.contains(&c) => {}
        Some(c) => {
            return Measurement::Unavailable(format!(
                "command exited {c}: {}",
                first_line(&String::from_utf8_lossy(&stderr))
            ))
        }
        None => return Measurement::Unavailable("command was killed by a signal".into()),
    }

    parse_count(&String::from_utf8_lossy(&stdout))
}

/// The last non-empty line of `stdout`, parsed as a count.
///
/// Last rather than first: the `| wc -l` idiom puts the answer at the end, and
/// a command that also prints progress must not be silently misread.
pub fn parse_count(stdout: &str) -> Measurement {
    let Some(line) = stdout.lines().rev().find(|l| !l.trim().is_empty()) else {
        return Measurement::Unavailable("command printed nothing".into());
    };
    match line.trim().parse::<i64>() {
        Ok(v) => Measurement::Value(v),
        Err(_) => Measurement::Unavailable(format!(
            "command printed `{}`, which is not a count",
            line.trim()
        )),
    }
}

fn first_line(text: &str) -> String {
    text.lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("(no stderr)")
        .trim()
        .to_string()
}
