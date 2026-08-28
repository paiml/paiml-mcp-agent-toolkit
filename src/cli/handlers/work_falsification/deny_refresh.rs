#![cfg_attr(coverage_nightly, coverage(off))]
//! GH #629: make the supply-chain claim satisfiable by refreshing its own cache.
//!
//! The gate reads `.pmat-metrics/deny-status.json` (falling back to
//! `deny-cache.txt`) and blocks once that file is older than
//! [`CACHE_BLOCK_HOURS`](super::types::CACHE_BLOCK_HOURS). Nothing in pmat, and
//! nothing pmat installs into a consumer repo, ever wrote either file — so the
//! advice it printed ("Run 'cargo deny check' first") could not clear it.
//! `cargo deny` writes to stdout, not to the cache. Every `pmat work complete`
//! therefore needed `--override-claims supply-chain`, which trains operators to
//! wave through a *security* gate.
//!
//! The fix is for pmat to populate the cache itself, from cargo-deny's exit
//! code. The O(1) contract is kept where it matters: a fresh cache is still a
//! single `stat` + parse, and the subprocess runs at most once per block window.

use super::types::CachedMetric;
use std::io::Read;
use std::path::Path;
use std::process::{Child, Command, Output, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

/// How long the timeout path waits for the two pipe readers once the tree is
/// dead.
///
/// Killing the whole process group closes every write end of both pipes, so the
/// readers reach EOF at once and this budget is normally not spent at all. It
/// is a ceiling rather than a delay, and it exists because `JoinHandle::join`
/// has no timeout: a descendant that had deliberately left the group (by
/// calling `setsid` for itself) could still hold a pipe open, and joining such
/// a reader unconditionally would trade a leaked thread for a hung caller.
/// `quality_proxy_analysis` reaches this helper from a tokio worker through
/// `spawn_blocking`, where a hang is the worse of the two.
const DRAIN_GRACE: Duration = Duration::from_secs(2);

/// Drain one of the child's pipes on its own thread.
///
/// The bytes come back over a channel rather than out of `JoinHandle::join`
/// because only a channel can be waited on with a deadline; see [`DRAIN_GRACE`].
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
/// signalled with a single call.
///
/// `process_group(0)` is `setpgid(0, 0)` performed between fork and exec
/// (stable since Rust 1.64; this crate's `rust-version` is 1.91.0), so the
/// child becomes the leader of a fresh group whose id is its own pid, and
/// everything it spawns inherits that group.
///
/// The cost, stated plainly rather than left for a reader to discover: the
/// child is no longer in the terminal's foreground process group, so a Ctrl-C
/// typed at the terminal reaches pmat but not the `cargo` underneath it. The
/// pipes limit the damage — their read ends die with pmat, and a surviving
/// `cargo` takes EPIPE on its next write — but "its next write" is not a bound,
/// so this is a real trade: an interactive interrupt becomes less immediate, in
/// exchange for the timeout, the only case that fires unattended, actually
/// stopping the compiler instead of merely stopping the wait for it.
#[cfg(unix)]
fn lead_new_process_group(cmd: &mut Command) {
    use std::os::unix::process::CommandExt;
    cmd.process_group(0);
}

/// Windows has no POSIX process group to lead, and `CommandExt::process_group`
/// does not exist there. Gating rather than assuming is deliberate: #1081
/// shipped Unix-only code on a path Windows executes, and `windows-check`
/// (`cargo check --bin pmat --locked`, `.github/workflows/ci.yml:145`) is the
/// leg that catches that class.
#[cfg(not(unix))]
fn lead_new_process_group(_cmd: &mut Command) {}

/// Kill the child *and everything it spawned*.
///
/// `Child::kill` signals only the direct child, and every caller of
/// `run_with_timeout` spawns a supervisor: `cargo` schedules the work and the
/// `rustc` / `clippy-driver` grandchildren are the processes that actually burn
/// the CPU and the memory — the epic behind this bound measured a
/// recursive-macro payload at roughly 25 GB RSS. Killing the supervisor and
/// returning therefore left the runaway compiler running and unowned: the
/// deadline bounded how long pmat waited for the process, not the process.
///
/// `lead_new_process_group` has already put the child at the head of its own
/// group, so its pgid equals its pid and one signal reaches the whole subtree.
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
            // first argument is POSIX's spelling of "every process in the group
            // whose id is its absolute value", and that group is the one
            // `lead_new_process_group` created for this child, so the blast
            // radius is exactly this child and its descendants. The result is
            // discarded for the same reason `child.kill()`'s was: by the time
            // the deadline fires the child may already have exited.
            let _ = unsafe { libc::kill(-pid, libc::SIGKILL) };
        }
        // Unreachable on any Unix pmat builds for — pids are bounded far below
        // `pid_t::MAX` — but an `as` cast here would wrap a pid that did not
        // fit into a *different* group id and signal strangers. Narrowing the
        // kill to the direct child is the safe way to be wrong.
        Err(_) => {
            let _ = child.kill();
        }
    }
}

/// Windows keeps exactly today's behaviour: terminate the direct child.
///
/// Bounding a whole tree there needs a Job Object, which needs a crate this
/// build does not carry (`libc` is declared for `cfg(unix)` only, and is a
/// POSIX binding regardless). Windows is no worse off than before this change;
/// it is simply not better off, and saying so here beats leaving a reader to
/// infer it from a `#[cfg]`.
#[cfg(not(unix))]
fn kill_process_tree(child: &mut Child) {
    let _ = child.kill();
}

/// Run a command, killing it and everything it spawned if it outruns `limit`.
///
/// Returns `Ok(None)` on timeout. stdout and stderr are drained on their own
/// threads: polling `try_wait` while the child writes into a full pipe buffer
/// would deadlock, which is the classic way a hand-rolled timeout makes hangs
/// *more* likely rather than less. Those readers are waited for on the timeout
/// path too, not only on the success path — before, each timeout left one
/// thread per pipe blocked in `read_to_end` for as long as the orphaned
/// grandchild held the write end open, which was forever.
///
/// stdin is `/dev/null`. That is not decoration: a child in a background
/// process group that reads from the terminal takes SIGTTIN and *stops*, and a
/// stopped child is invisible to `try_wait` — it would sit there until the
/// deadline. No caller feeds this helper input; all four spawn `cargo` or
/// `rustfmt` with arguments.
pub(crate) fn run_with_timeout(
    cmd: &mut Command,
    limit: Duration,
) -> std::io::Result<Option<Output>> {
    lead_new_process_group(cmd);
    let mut child = cmd
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let out_rx = drain_on_thread(child.stdout.take());
    let err_rx = drain_on_thread(child.stderr.take());

    let deadline = Instant::now() + limit;
    let status = loop {
        match child.try_wait()? {
            Some(status) => break status,
            None if Instant::now() >= deadline => {
                // Signal before reaping. A reaped pid can be reused, and while
                // the killed leader is still an unreaped zombie its group id
                // stays valid, so this order is also what lets one signal reach
                // descendants that outlived their parent.
                kill_process_tree(&mut child);
                let _ = child.wait();
                let _ = out_rx.recv_timeout(DRAIN_GRACE);
                let _ = err_rx.recv_timeout(DRAIN_GRACE);
                return Ok(None);
            }
            None => std::thread::sleep(Duration::from_millis(50)),
        }
    };

    Ok(Some(Output {
        status,
        stdout: out_rx.recv().unwrap_or_default(),
        stderr: err_rx.recv().unwrap_or_default(),
    }))
}

/// Result of trying to bring the deny cache up to date.
pub(crate) enum DenyRefresh {
    /// cargo-deny ran to completion; the cache now reflects its verdict.
    Recorded(Box<CachedMetric>),
    /// cargo-deny is not available, so the claim cannot be evaluated here.
    ToolMissing,
    /// cargo-deny could not be run, or its verdict could not be persisted.
    Failed(String),
}

/// Where the refreshed verdict is written, relative to the project root.
pub(crate) const DENY_STATUS_PATH: &str = ".pmat-metrics/deny-status.json";

/// Install hint shown when cargo-deny is absent.
pub(crate) const CARGO_DENY_INSTALL_HINT: &str = "cargo install cargo-deny --locked";

/// True if cargo reported that the `deny` subcommand does not exist.
///
/// `Command::new("cargo")` succeeds whenever cargo itself is installed, so a
/// missing cargo-deny surfaces as a normal non-zero exit with this message
/// rather than as an `io::ErrorKind::NotFound`.
fn is_missing_subcommand(stderr: &str) -> bool {
    stderr.contains("no such command") || stderr.contains("no such subcommand")
}

/// Count vulnerability diagnostics in cargo-deny output.
///
/// cargo-deny emits one `error[vulnerability]` block per advisory. Bans,
/// licence and source violations use their own codes and are deliberately not
/// counted here — they still fail the gate via the exit code, but the numeric
/// evidence attached to the claim is specifically a vulnerability count.
fn count_vulnerabilities(output: &str) -> u64 {
    output.matches("error[vulnerability]").count() as u64
}

/// A one-line reason a cargo-deny run failed.
///
/// cargo-deny fails for bans, licences and sources as well as advisories, and
/// those carry no `error[vulnerability]`. Without this, such a run was reported
/// as "0 vulnerabilities" — a failure verdict phrased as a success, which is
/// worse than no message. Prefers cargo-deny's own per-check summary
/// (`advisories ok, bans ok, licenses FAILED, sources ok`), then the first
/// error diagnostic.
fn failure_summary(stdout: &str, stderr: &str) -> String {
    let lines = stdout.lines().chain(stderr.lines()).map(str::trim);
    let mut first_error = None;

    for line in lines {
        if line.contains("FAILED") && line.contains(" ok") {
            return line.to_string();
        }
        if first_error.is_none() && line.starts_with("error") {
            first_error = Some(line.to_string());
        }
    }
    first_error.unwrap_or_else(|| "cargo deny check failed".to_string())
}

/// How long cargo-deny may run before the gate gives up on it.
///
/// `cargo deny check` git-fetches the RustSec advisory database, so a stalled
/// connection would otherwise block `pmat work complete` forever behind a
/// half-printed "Reading deny cache... " line, with no output and no way to
/// tell a hang from slow work.
const DENY_TIMEOUT: Duration = Duration::from_secs(120);

/// Run `cargo deny check` in `project_path` and persist its verdict.
pub(crate) fn refresh_deny_cache(project_path: &Path) -> DenyRefresh {
    let output = match run_with_timeout(
        Command::new("cargo")
            .args(["deny", "check"])
            .current_dir(project_path),
        DENY_TIMEOUT,
    ) {
        Ok(Some(output)) => output,
        Ok(None) => {
            return DenyRefresh::Failed(format!(
                "'cargo deny check' did not finish within {}s (advisory-DB fetch may be stalled; \
                 check connectivity or run it manually)",
                DENY_TIMEOUT.as_secs()
            ))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return DenyRefresh::ToolMissing,
        Err(e) => return DenyRefresh::Failed(format!("could not run 'cargo deny check': {e}")),
    };

    // cargo-deny writes its diagnostics to stderr and its per-check summary to
    // stdout; scan both so the count is independent of that split.
    let stderr = String::from_utf8_lossy(&output.stderr);
    if is_missing_subcommand(&stderr) {
        return DenyRefresh::ToolMissing;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);

    let passed = output.status.success();
    let count = count_vulnerabilities(&stderr) + count_vulnerabilities(&stdout);
    let value = serde_json::json!({
        "passed": passed,
        "vulnerability_count": count,
        "summary": if passed { String::new() } else { failure_summary(&stdout, &stderr) },
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "source": "pmat work complete (cargo deny check)",
    });

    if let Err(e) = write_status(project_path, &value) {
        return DenyRefresh::Failed(format!("could not write {DENY_STATUS_PATH}: {e}"));
    }

    DenyRefresh::Recorded(Box::new(CachedMetric {
        value,
        age_minutes: 0,
        is_stale_warn: false,
        is_stale_block: false,
    }))
}

/// Persist the verdict to `.pmat-metrics/deny-status.json`, creating the
/// directory if the consumer repo has never recorded a metric before.
fn write_status(project_path: &Path, value: &serde_json::Value) -> std::io::Result<()> {
    write_json_status(project_path, DENY_STATUS_PATH, value)
}

/// Write a metric verdict under the project, creating `.pmat-metrics/` if the
/// consumer repo has never recorded one before.
pub(crate) fn write_json_status(
    project_path: &Path,
    relative: &str,
    value: &serde_json::Value,
) -> std::io::Result<()> {
    let path = project_path.join(relative);
    // #1070: `.pmat-metrics/` is created with its own ignore rule, so a verdict
    // written into a consumer repo does not dirty that repo's git status.
    crate::utils::pmat_cache_dir::ensure_parent_dir(&path)?;
    std::fs::write(path, serde_json::to_string_pretty(value)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_subcommand_detected_from_cargo_message() {
        // The exact wording cargo uses when cargo-deny is not installed.
        assert!(is_missing_subcommand("error: no such command: `deny`"));
        assert!(is_missing_subcommand("error: no such subcommand: `deny`"));
    }

    #[test]
    fn missing_subcommand_not_confused_with_a_real_failure() {
        assert!(!is_missing_subcommand(
            "error[vulnerability]: RUSTSEC-2024-0001"
        ));
        assert!(!is_missing_subcommand("advisories FAILED"));
        assert!(!is_missing_subcommand(""));
    }

    #[test]
    fn vulnerabilities_counted_per_diagnostic() {
        let out = "error[vulnerability]: RUSTSEC-2024-0001\n\
                   error[vulnerability]: RUSTSEC-2024-0002\n\
                   warning[unmaintained]: RUSTSEC-2024-0003\n";
        assert_eq!(count_vulnerabilities(out), 2);
    }

    #[test]
    fn failure_summary_prefers_the_per_check_line() {
        // The real shape of a licence failure, which carries no
        // error[vulnerability] and so would otherwise report "0 vulnerabilities".
        let stdout = "advisories ok, bans ok, licenses FAILED, sources ok\n";
        let stderr = "error[unlicensed]: dogfood = 0.1.0 is unlicensed\n";
        assert_eq!(
            failure_summary(stdout, stderr),
            "advisories ok, bans ok, licenses FAILED, sources ok"
        );
    }

    #[test]
    fn failure_summary_falls_back_to_the_first_error() {
        let stderr =
            "error[vulnerability]: RUSTSEC-2024-0001\nerror[vulnerability]: RUSTSEC-2024-0002\n";
        assert_eq!(
            failure_summary("", stderr),
            "error[vulnerability]: RUSTSEC-2024-0001"
        );
    }

    #[test]
    fn failure_summary_always_says_something() {
        assert_eq!(failure_summary("", ""), "cargo deny check failed");
    }

    #[test]
    fn clean_output_counts_zero() {
        assert_eq!(count_vulnerabilities("advisories ok\nbans ok\n"), 0);
        assert_eq!(count_vulnerabilities(""), 0);
    }

    #[test]
    fn write_status_creates_missing_metrics_dir() {
        // A consumer repo that has never recorded a metric has no
        // .pmat-metrics/ at all; the refresh must not fail on that.
        let dir = tempfile::tempdir().unwrap();
        let value = serde_json::json!({ "passed": true, "vulnerability_count": 0 });

        write_status(dir.path(), &value).unwrap();

        let written = std::fs::read_to_string(dir.path().join(DENY_STATUS_PATH)).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&written).unwrap();
        assert_eq!(parsed["passed"], serde_json::json!(true));
        assert_eq!(parsed["vulnerability_count"], serde_json::json!(0));
    }

    #[test]
    fn write_status_overwrites_a_stale_verdict() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".pmat-metrics")).unwrap();
        std::fs::write(
            dir.path().join(DENY_STATUS_PATH),
            r#"{"passed": true, "timestamp": "2026-07-01T21:02:57Z"}"#,
        )
        .unwrap();

        write_status(
            dir.path(),
            &serde_json::json!({ "passed": false, "vulnerability_count": 3 }),
        )
        .unwrap();

        let written = std::fs::read_to_string(dir.path().join(DENY_STATUS_PATH)).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&written).unwrap();
        assert_eq!(parsed["passed"], serde_json::json!(false));
        assert_eq!(parsed["vulnerability_count"], serde_json::json!(3));
    }

    /// True while `pid` still names a *running* process.
    ///
    /// `kill(pid, 0)` is the POSIX liveness probe: it runs the existence and
    /// permission checks and delivers nothing. It answers "yes" for a zombie
    /// though, and a killed grandchild whose parent died with it stays a zombie
    /// until whatever inherits it reaps — prompt under a normal init, never
    /// under a PID 1 that does not reap, which is the shape of a bare
    /// container. On Linux the state field of `/proc/<pid>/stat` separates the
    /// two, so the assertion below does not silently depend on the reaper.
    #[cfg(unix)]
    fn process_is_running(pid: libc::pid_t) -> bool {
        // SAFETY: two integers in, one out, no pointer dereferenced — the same
        // argument as `kill_process_tree`, and signal 0 delivers nothing.
        if unsafe { libc::kill(pid, 0) } != 0 {
            return false;
        }
        #[cfg(target_os = "linux")]
        {
            if let Ok(stat) = std::fs::read_to_string(format!("/proc/{pid}/stat")) {
                // Field 2 is the executable name in parentheses and may itself
                // contain spaces and parentheses, so the state character is the
                // first token after the LAST ')'.
                if let Some((_, after_comm)) = stat.rsplit_once(')') {
                    return after_comm.split_whitespace().next() != Some("Z");
                }
            }
        }
        true
    }

    /// The shape the deadline has to survive: a supervisor whose work outlives
    /// its own death through a process it spawned. `sh` stands in for `cargo`
    /// and the backgrounded `sleep` for the `rustc` / `clippy-driver` that
    /// actually holds the memory.
    ///
    /// This fails on the pre-fix code, and for the reason the fix names:
    /// `child.kill()` signals `sh` alone, so the `sleep` is reparented and runs
    /// its full 300 seconds, and the probe below still finds it alive.
    #[cfg(unix)]
    #[test]
    fn timeout_kills_the_grandchild_not_only_the_child() {
        let dir = tempfile::tempdir().expect("tempdir");
        let pid_file = dir.path().join("grandchild.pid");
        let script = format!(
            "sleep 300 & printf '%s' \"$!\" > '{}'; wait",
            pid_file.display()
        );

        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg(script);

        let started = Instant::now();
        let outcome = run_with_timeout(&mut cmd, Duration::from_secs(1)).expect("sh must spawn");
        let elapsed = started.elapsed();

        assert!(outcome.is_none(), "a 300s sleep must outrun a 1s deadline");

        let pid: libc::pid_t = std::fs::read_to_string(&pid_file)
            .expect("the shell must have recorded the grandchild pid")
            .trim()
            .parse()
            .expect("the recorded pid must parse");

        // Poll rather than probe once: SIGKILL is delivered asynchronously.
        let mut alive = true;
        let until = Instant::now() + Duration::from_secs(5);
        while Instant::now() < until {
            if !process_is_running(pid) {
                alive = false;
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
        }

        // Leave nothing behind whichever way the assertion goes: on the unfixed
        // code this `sleep` would otherwise sit in the test run for five more
        // minutes, which is exactly the defect and a poor way to observe it.
        if alive {
            // SAFETY: as above — integers only, and `pid` names a process this
            // test's own subtree created.
            let _ = unsafe { libc::kill(pid, libc::SIGKILL) };
        }

        assert!(
            !alive,
            "the grandchild ({pid}) outlived the deadline: the timeout killed the \
             supervisor and left the process that burns the memory running"
        );
        assert!(
            elapsed < Duration::from_secs(3),
            "the timeout path took {elapsed:?}; the pipe readers cannot have reached \
             EOF, so it spent the {DRAIN_GRACE:?} grace on each instead of returning"
        );
    }

    /// The process group is created for every spawn, not only the ones that
    /// time out, so pin that an ordinary run still reports what it always did.
    #[cfg(unix)]
    #[test]
    fn a_command_that_finishes_still_returns_its_output_and_status() {
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg("printf out; printf err >&2; exit 3");

        let output = run_with_timeout(&mut cmd, Duration::from_secs(30))
            .expect("sh must spawn")
            .expect("30s is not a bound two printfs can outrun");

        assert_eq!(String::from_utf8_lossy(&output.stdout), "out");
        assert_eq!(String::from_utf8_lossy(&output.stderr), "err");
        assert_eq!(output.status.code(), Some(3));
    }

    /// A child in a background process group that reads the terminal takes
    /// SIGTTIN and stops, and a stopped child looks exactly like a slow one to
    /// `try_wait` — it would burn the whole deadline. Null stdin means `cat`
    /// reads EOF and exits at once instead.
    #[cfg(unix)]
    #[test]
    fn the_child_reads_eof_rather_than_the_terminal() {
        let mut cmd = Command::new("cat");

        let output = run_with_timeout(&mut cmd, Duration::from_secs(10))
            .expect("cat must spawn")
            .expect("cat must see EOF on a null stdin rather than block on a terminal");

        assert!(output.status.success());
        assert!(output.stdout.is_empty());
    }
}
