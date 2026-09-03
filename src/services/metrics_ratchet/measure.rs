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
use std::path::Path;
use std::process::Command;

/// `grep` and `git grep` exit 1 to mean "no matches", which is a legitimate
/// count of zero, not an error. Anything else — 2 from a bad pathspec, 127
/// from a missing binary, 128 from git — is a broken measurement.
const ACCEPTABLE_EXIT_CODES: [i32; 2] = [0, 1];

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

/// Run one command and classify its result. Every fail-closed decision lives
/// here; [`measure`] adds only the test-time memo.
fn measure_now(project_path: &Path, command: &str) -> Measurement {
    if command.trim().is_empty() {
        return Measurement::Unavailable(
            "the metric declares no command, so its baseline cannot be reproduced".into(),
        );
    }
    let output = Command::new("bash")
        .arg("-o")
        .arg("pipefail")
        .arg("-c")
        .arg(command)
        .current_dir(project_path)
        // Deterministic collation and message text: a metric that greps must
        // not depend on the locale of whoever ran it.
        .env("LC_ALL", "C")
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
        .output();

    let output = match output {
        Ok(o) => o,
        Err(e) => return Measurement::Unavailable(format!("could not run bash: {e}")),
    };

    match output.status.code() {
        Some(c) if ACCEPTABLE_EXIT_CODES.contains(&c) => {}
        Some(c) => {
            return Measurement::Unavailable(format!(
                "command exited {c}: {}",
                first_line(&String::from_utf8_lossy(&output.stderr))
            ))
        }
        None => return Measurement::Unavailable("command was killed by a signal".into()),
    }

    parse_count(&String::from_utf8_lossy(&output.stdout))
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
