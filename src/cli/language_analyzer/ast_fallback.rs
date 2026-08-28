//! Which analyzer actually produced a file's complexity numbers.
//!
//! Issue #1068 (#1050 P6). `analyze_file_complexity` prefers `syn` for Rust and
//! drops to the regex counter when the parse fails. The only disclosure was
//! `Warning: AST analysis failed for …/src/lib.rs, using heuristic fallback`
//! on STDERR — so `--format json`, `--output FILE` and every machine consumer
//! saw a document shaped exactly like a clean parse. On a full bashrs run ten
//! files were counted by regex and none of them were distinguishable; on one of
//! five real files the fallback was over by 2.4x.
//!
//! The provenance is recorded HERE, at the branch that decides it, rather than
//! re-derived by the caller: a second derivation is a second chance to
//! disagree, which is the defect the complexity census comments already record.
//!
//! ## Why a process-global ledger, and why it is normally inert
//!
//! `analyze_file_complexity` returns `FileComplexityMetrics`, a type shared by
//! the MCP tools, the TDG analyzer, `refactor auto` and the deep-context
//! walker. Threading a provenance field through it would touch all of them for
//! one command's disclosure. Instead the ledger is armed for the duration of
//! one complexity run and disarmed when the run collects it, so every other
//! caller in the process pays one relaxed atomic load and stores nothing.
//!
//! ## Why one run at a time
//!
//! A process-global ledger with `arm`/`take` is only correct for ONE run at a
//! time, and that was not enforced. Two concurrent analyses interleaved as:
//! A arms, B arms and CLEARS A's entries, A's files record, B takes and steals
//! them, A takes and gets nothing — reporting `unrecorded: N` for a walk whose
//! every file WAS recorded. Reproduced by the test suite the moment two
//! complexity tests ran in parallel:
//!
//! ```text
//! "analysis_provenance":{"ast":0,...,"unrecorded":2,"files_analyzed":2}
//! ```
//!
//! That is not a test artifact. `mcp-http` is in the default feature set as of
//! 3.32.0, so the MCP and HTTP servers both serve concurrent `analyze_complexity`
//! calls in one process, where the same interleaving loses or cross-attributes
//! provenance between unrelated requests.
//!
//! `arm` therefore takes an async lock held until `take` returns it, so runs
//! queue instead of corrupting each other. The cost is small and bounded: the
//! guarded region is one CPU-bound walk that was never going to parallelise
//! usefully against another copy of itself, and every non-complexity caller
//! still pays only the relaxed atomic load.
//!
//! The alternative — a task-local run id — was rejected because it would go
//! SILENT rather than blocking if any future caller moved the walk onto
//! `spawn_blocking` or rayon: records made off-task would land nowhere and the
//! run would report `unrecorded`, which is the exact failure being fixed.

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock, PoisonError};

/// How a file's numbers were obtained.
///
/// Four states, not two. Collapsing them would reintroduce the defect in a
/// different place:
///
/// * a Python file was ALWAYS going to be counted by the heuristic analyzer,
///   because this build has no Python AST complexity analyzer — reporting that
///   normal path as a degradation is the same defect pointing the other way;
/// * an `include!()` fragment is Rust that pmat has an AST analyzer for and
///   deliberately does not apply, because the file is not standalone-parseable.
///   That is a choice, not a failure, and not the same as (2). It is also not
///   rare: 470 of the 830 `.rs` files under this repo's own
///   `src/cli/handlers` are fragments, so folding them into "heuristic" would
///   have a pure-Rust tree report a majority counted by a language analyzer
///   that supposedly does not exist for it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provenance {
    /// Parsed by the Rust AST analyzer.
    Ast,
    /// Counted by the language's heuristic analyzer, which is the only
    /// analyzer this build has for that language. Not a degradation.
    Heuristic,
    /// Rust, but an `include!()` fragment — not standalone-parseable, so the
    /// AST analyzer is deliberately skipped. A choice, not a failure.
    HeuristicIncludeFragment,
    /// Rust, and `syn` refused the file: the numbers are regex-derived and may
    /// be wrong.
    HeuristicFallback,
}

impl Provenance {
    /// The wire spelling. Stable: it is a JSON field value.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ast => "ast",
            Self::Heuristic => "heuristic",
            Self::HeuristicIncludeFragment => "heuristic_include_fragment",
            Self::HeuristicFallback => "heuristic_fallback",
        }
    }
}

/// Off unless a run has armed it. Checked before the lock so the ledger costs
/// nothing on the paths that will never read it.
static ARMED: AtomicBool = AtomicBool::new(false);

fn ledger() -> &'static Mutex<BTreeMap<String, Provenance>> {
    static LEDGER: OnceLock<Mutex<BTreeMap<String, Provenance>>> = OnceLock::new();
    LEDGER.get_or_init(|| Mutex::new(BTreeMap::new()))
}

/// Key by the string the metrics themselves carry.
///
/// `FileComplexityMetrics::path` is `path.to_string_lossy()` on both the AST
/// and the heuristic branch, so keying on the same expression makes the join
/// in the serializer exact — no canonicalization, and no way for a
/// relative-vs-absolute mismatch to silently drop a file's provenance.
fn key(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

/// Record how one file was measured. A no-op unless a run armed the ledger.
pub(crate) fn record(path: &Path, provenance: Provenance) {
    if !ARMED.load(Ordering::Relaxed) {
        return;
    }
    // A poisoned ledger still holds every entry written before the panic;
    // skipping the write instead would UNDER-report fallbacks, which is the
    // exact failure this module exists to stop.
    ledger()
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .insert(key(path), provenance);
}

fn run_lock() -> Arc<tokio::sync::Mutex<()>> {
    static RUN_LOCK: OnceLock<Arc<tokio::sync::Mutex<()>>> = OnceLock::new();
    Arc::clone(RUN_LOCK.get_or_init(|| Arc::new(tokio::sync::Mutex::new(()))))
}

/// Exclusive right to record provenance. Held from `arm` until `take`.
///
/// Not `Clone`, and `take` consumes it, so the ledger cannot be collected twice
/// or read by a run that never armed it.
pub struct RunGuard {
    /// Underscore-named rather than carrying an allow attribute: the permit is
    /// held for its Drop, never read, and this repo ratchets allow-attribute
    /// counts by literal string, so a suppression here would raise the baseline.
    _permit: tokio::sync::OwnedMutexGuard<()>,
}

/// Start recording for one run, discarding anything a previous run left.
///
/// Waits for any run already in flight rather than clearing its ledger out from
/// under it — see the module note on why one run at a time.
pub async fn arm() -> RunGuard {
    let guard = run_lock().lock_owned().await;
    ledger()
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clear();
    ARMED.store(true, Ordering::Relaxed);
    RunGuard { _permit: guard }
}

/// Stop recording and take what this run observed, releasing the run lock.
#[must_use]
pub fn take(guard: RunGuard) -> BTreeMap<String, Provenance> {
    ARMED.store(false, Ordering::Relaxed);
    let observed = std::mem::take(&mut *ledger().lock().unwrap_or_else(PoisonError::into_inner));
    drop(guard);
    observed
}

/// How many files fell into each bucket, and the population they partition.
///
/// `analyzed` is passed in rather than derived from the map: the map covers
/// what the ledger SAW, and a breakdown that quietly used its own length as
/// the denominator would be unfalsifiable — the very shape
/// (`files_discovered == files_analyzed`) that issue #1065 was about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProvenanceTally {
    pub ast: usize,
    pub heuristic: usize,
    pub heuristic_include_fragment: usize,
    pub heuristic_fallback: usize,
    /// Files the run reported. The four buckets plus `unrecorded()` equal this.
    pub analyzed: usize,
}

impl ProvenanceTally {
    /// Files the run reported that the ledger has no entry for. Should be 0;
    /// it is published so a join that silently stopped matching shows up as a
    /// number instead of as four plausible buckets that do not add up.
    #[must_use]
    pub fn unrecorded(self) -> usize {
        self.analyzed.saturating_sub(
            self.ast + self.heuristic + self.heuristic_include_fragment + self.heuristic_fallback,
        )
    }
}

/// Tally the ledger over the files a run actually reported.
///
/// `analyzed_paths` is the metrics vector's paths, so a file the ledger
/// recorded but the run then filtered out is not counted here.
#[must_use]
pub fn tally<'a>(
    recorded: &BTreeMap<String, Provenance>,
    analyzed_paths: impl Iterator<Item = &'a str>,
) -> ProvenanceTally {
    let mut tally = ProvenanceTally {
        ast: 0,
        heuristic: 0,
        heuristic_include_fragment: 0,
        heuristic_fallback: 0,
        analyzed: 0,
    };
    for path in analyzed_paths {
        tally.analyzed += 1;
        match recorded.get(path) {
            Some(Provenance::Ast) => tally.ast += 1,
            Some(Provenance::Heuristic) => tally.heuristic += 1,
            Some(Provenance::HeuristicIncludeFragment) => tally.heuristic_include_fragment += 1,
            Some(Provenance::HeuristicFallback) => tally.heuristic_fallback += 1,
            None => {}
        }
    }
    tally
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buckets_partition_the_analyzed_population() {
        let mut recorded = BTreeMap::new();
        recorded.insert("a.rs".to_string(), Provenance::Ast);
        recorded.insert("b.rs".to_string(), Provenance::HeuristicFallback);
        recorded.insert("c.py".to_string(), Provenance::Heuristic);
        recorded.insert(
            "d_tests.rs".to_string(),
            Provenance::HeuristicIncludeFragment,
        );

        let t = tally(
            &recorded,
            ["a.rs", "b.rs", "c.py", "d_tests.rs"].into_iter(),
        );
        assert_eq!(t.ast, 1);
        assert_eq!(t.heuristic, 1);
        assert_eq!(t.heuristic_include_fragment, 1);
        assert_eq!(t.heuristic_fallback, 1);
        assert_eq!(t.analyzed, 4);
        assert_eq!(t.unrecorded(), 0);
    }

    /// A join that stopped matching must show up as a number, not as three
    /// buckets that quietly sum to less than the population.
    #[test]
    fn a_file_the_ledger_missed_is_counted_as_unrecorded() {
        let mut recorded = BTreeMap::new();
        recorded.insert("a.rs".to_string(), Provenance::Ast);

        let t = tally(&recorded, ["a.rs", "ghost.rs"].into_iter());
        assert_eq!(t.analyzed, 2);
        assert_eq!(t.unrecorded(), 1);
    }

    /// A file the ledger recorded but the run filtered out is not in the
    /// tally: the tally describes what was REPORTED.
    #[test]
    fn a_filtered_out_file_is_not_tallied() {
        let mut recorded = BTreeMap::new();
        recorded.insert("kept.rs".to_string(), Provenance::Ast);
        recorded.insert("dropped.rs".to_string(), Provenance::HeuristicFallback);

        let t = tally(&recorded, ["kept.rs"].into_iter());
        assert_eq!(t.analyzed, 1);
        assert_eq!(t.heuristic_fallback, 0);
    }

    #[test]
    fn wire_spellings_are_the_four_documented_states() {
        assert_eq!(Provenance::Ast.as_str(), "ast");
        assert_eq!(Provenance::Heuristic.as_str(), "heuristic");
        assert_eq!(
            Provenance::HeuristicIncludeFragment.as_str(),
            "heuristic_include_fragment"
        );
        assert_eq!(Provenance::HeuristicFallback.as_str(), "heuristic_fallback");
    }

    /// The two deliberate heuristic states must stay distinguishable from the
    /// degraded one: `include!()` fragments are the MAJORITY of `.rs` files in
    /// parts of this repo, and folding them into either neighbour would either
    /// invent 470 fallbacks or claim Rust has no AST analyzer.
    #[test]
    fn a_deliberate_skip_is_not_spelled_like_a_failed_parse() {
        assert_ne!(
            Provenance::HeuristicIncludeFragment.as_str(),
            Provenance::HeuristicFallback.as_str()
        );
        assert_ne!(
            Provenance::HeuristicIncludeFragment.as_str(),
            Provenance::Heuristic.as_str()
        );
    }
}
