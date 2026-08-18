//! Mutation-adequacy gate (EV-4, #1034): turn a `cargo mutants` run into a verdict.
//!
//! Mutation score is the only test-adequacy metric in the evidence table with
//! controlled experimental support that it tracks real-fault detection
//! *independently of coverage* (Just et al., FSE 2014); Inozemtseva & Holmes
//! (ICSE 2014) show line coverage does not. So a coverage gate cannot stand in
//! for this one, and the falsifier in `mutation_gate_tests` demonstrates why:
//! weakening an assertion while adding a compensating test leaves line coverage
//! at exactly 100% and drops the mutation score from 88.9% to 0%.
//!
//! ## Why the gate is not just "cargo mutants' exit code"
//!
//! Measured against cargo-mutants 27.0.0, not assumed:
//!
//! * `cargo mutants --in-diff D` where `D` changes no Rust source **exits 0 and
//!   writes no `mutants.out/` at all**. A CI step that runs the tool and then
//!   trusts its exit code passes without having tested one mutant.
//! * Worse, that run leaves any **pre-existing `mutants.out/outcomes.json` in
//!   place, untouched**. A gate that reads the file after such a run reads a
//!   stale, unrelated, possibly all-caught result and reports success.
//!
//! Hence [`evaluate_mutation_gate`] is fail-closed and cross-checks the
//! artifact against the diff it is supposed to describe.
//!
//! ## Invariants (issue #1034, EV-4)
//!
//! * `INV-MUT-1` — a surviving (missed or timed-out) mutant inside the diff fails.
//! * `INV-MUT-2` — an empty mutant set is **not** a pass: `|mutants(diff)| = 0`
//!   while the diff touches mutable Rust source is a failure, and so is a
//!   missing/unreadable artifact.
//! * `INV-MUT-3` — a degraded or stubbed backend cannot report all-mutants-caught:
//!   the unmutated baseline, the per-mutant build/test phase evidence, the
//!   backend version and the headline counts must all corroborate each other.
//!
//! Order matters. `INV-MUT-3` is evaluated first, so a backend that simply
//! prints "everything was caught" is rejected before its numbers are believed.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// Default `exclude_globs` assumed when a project has no `mutants.toml`.
///
/// Mirrors this repository's committed `mutants.toml`; `load_exclude_globs`
/// prefers the file whenever it exists so the gate's notion of "mutable Rust
/// source" cannot drift from the tool's.
pub const DEFAULT_EXCLUDE_GLOBS: &[&str] = &[
    "**/tests/**",
    "**/benches/**",
    "**/examples/**",
    "**/build.rs",
    "**/*_tests.rs",
    "**/*_test.rs",
];

/// Which cargo-mutants scenario an outcome entry describes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScenarioKind {
    /// The unmutated baseline run.
    Baseline,
    /// One mutated build.
    Mutant,
}

/// One phase (`Build` / `Test`) of one scenario, with whether it succeeded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhaseResult {
    /// Phase name as cargo-mutants reports it: `Build`, `Test`, ...
    pub phase: String,
    /// `process_status` was the string `"Success"` (as opposed to `{"Failure": n}`).
    pub succeeded: bool,
}

/// One entry of `outcomes.json`'s `outcomes` array.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutcomeEntry {
    /// Baseline or mutant.
    pub kind: ScenarioKind,
    /// `summary`: `Success`, `CaughtMutant`, `MissedMutant`, `Timeout`, `Unviable`.
    pub summary: String,
    /// Source file the mutant was applied to (`None` for the baseline).
    pub file: Option<String>,
    /// Per-phase evidence that the scenario was actually built and run.
    pub phases: Vec<PhaseResult>,
}

/// A parsed `mutants.out/outcomes.json`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MutationOutcomes {
    /// Headline `total_mutants`.
    pub total_mutants: u64,
    /// Headline `caught`.
    pub caught: u64,
    /// Headline `missed`.
    pub missed: u64,
    /// Headline `timeout`.
    pub timeout: u64,
    /// Headline `unviable`.
    pub unviable: u64,
    /// `cargo_mutants_version`, absent in a hand-rolled/stubbed artifact.
    pub version: Option<String>,
    /// The per-scenario entries.
    pub entries: Vec<OutcomeEntry>,
}

/// What the diff under test changes, as far as mutation is concerned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffScope {
    /// No diff was supplied — the gate cannot tell whether an empty mutant set
    /// is legitimate, and therefore treats it as a failure.
    Unknown,
    /// The diff changes no file cargo-mutants would mutate.
    NoMutableRustSource,
    /// The diff changes these mutable Rust source paths.
    MutableRustSource(BTreeSet<String>),
}

/// One reason the gate reached its verdict, tagged with the invariant at stake.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateFinding {
    /// `INV-MUT-1` / `INV-MUT-2` / `INV-MUT-3`.
    pub invariant: &'static str,
    /// Human-readable explanation.
    pub message: String,
}

/// The gate's verdict.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MutationGateVerdict {
    /// True only when no finding fired.
    pub passed: bool,
    /// Every reason the gate failed, most fundamental first.
    pub findings: Vec<GateFinding>,
    /// One-line summary suitable for a violation message.
    pub summary: String,
}

impl MutationGateVerdict {
    /// The invariant ids that fired, in order.
    #[must_use]
    pub fn fired(&self) -> Vec<&'static str> {
        self.findings.iter().map(|f| f.invariant).collect()
    }
}

// ── diff classification ────────────────────────────────────────────────────

/// Does `path` match one of cargo-mutants' `exclude_globs`?
///
/// Supports the two shapes those globs actually take: `**/<dir>/**` (any path
/// component equal to `<dir>`) and `**/<name>` where `<name>` may start with
/// `*` (suffix match on the file name).
#[must_use]
pub fn matches_exclude_glob(path: &str, glob: &str) -> bool {
    let rest = glob.strip_prefix("**/").unwrap_or(glob);
    if let Some(dir) = rest.strip_suffix("/**") {
        return path.split('/').any(|c| c == dir);
    }
    let file_name = path.rsplit('/').next().unwrap_or(path);
    match rest.strip_prefix('*') {
        Some(suffix) => file_name.ends_with(suffix),
        None => file_name == rest,
    }
}

/// Would cargo-mutants mutate `path`, given `excludes`?
#[must_use]
pub fn is_mutable_rust_source(path: &str, excludes: &[String]) -> bool {
    if !path.ends_with(".rs") {
        return false;
    }
    !excludes.iter().any(|g| matches_exclude_glob(path, g))
}

/// Read `exclude_globs` out of a project's `mutants.toml`, falling back to
/// [`DEFAULT_EXCLUDE_GLOBS`] when the file is absent or has no such key.
///
/// Deliberately a line scanner rather than a TOML parse: cargo-mutants itself
/// silently ignores unknown keys, so the value that matters is the literal text
/// under `exclude_globs`, and a partial/odd file must not make the gate skip.
#[must_use]
pub fn load_exclude_globs(project_path: &Path) -> Vec<String> {
    let default = || {
        DEFAULT_EXCLUDE_GLOBS
            .iter()
            .map(|s| (*s).to_string())
            .collect::<Vec<_>>()
    };
    let Ok(text) = std::fs::read_to_string(project_path.join("mutants.toml")) else {
        return default();
    };
    let mut globs = Vec::new();
    let mut in_block = false;
    for line in text.lines() {
        let t = line.trim();
        if t.starts_with('#') {
            continue;
        }
        if !in_block {
            if t.starts_with("exclude_globs") && t.contains('[') {
                in_block = true;
                // `exclude_globs = ["a", "b"]` on one line
                if let Some(inner) = t.split_once('[').map(|(_, r)| r) {
                    push_quoted(inner, &mut globs);
                    if inner.contains(']') {
                        in_block = false;
                    }
                }
            }
            continue;
        }
        push_quoted(t, &mut globs);
        if t.contains(']') {
            in_block = false;
        }
    }
    if globs.is_empty() {
        default()
    } else {
        globs
    }
}

fn push_quoted(s: &str, out: &mut Vec<String>) {
    let mut rest = s;
    while let Some(open) = rest.find('"') {
        let after = &rest[open + 1..];
        let Some(close) = after.find('"') else { break };
        out.push(after[..close].to_string());
        rest = &after[close + 1..];
    }
}

/// Classify a unified diff into a [`DiffScope`].
///
/// Reads the `+++ b/<path>` headers, which is what `git diff` emits and what
/// `cargo mutants --in-diff` itself consumes. `/dev/null` targets (deletions)
/// are skipped: a deleted file has nothing left to mutate.
#[must_use]
pub fn classify_diff(diff_text: &str, excludes: &[String]) -> DiffScope {
    let mut files = BTreeSet::new();
    for line in diff_text.lines() {
        let Some(rest) = line.strip_prefix("+++ ") else {
            continue;
        };
        let path = rest.split('\t').next().unwrap_or(rest).trim();
        if path == "/dev/null" {
            continue;
        }
        let path = path.strip_prefix("b/").unwrap_or(path);
        if is_mutable_rust_source(path, excludes) {
            files.insert(path.to_string());
        }
    }
    if files.is_empty() {
        DiffScope::NoMutableRustSource
    } else {
        DiffScope::MutableRustSource(files)
    }
}

// ── outcomes.json parsing ──────────────────────────────────────────────────

/// Parse the text of a cargo-mutants `outcomes.json`.
///
/// # Errors
/// Returns a message when the document is not an object, or its `outcomes` key
/// is missing or not an array. Everything else is recorded rather than rejected
/// so that [`evaluate_mutation_gate`] — not the parser — decides the verdict.
pub fn parse_outcomes(json: &str) -> Result<MutationOutcomes, String> {
    let val: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("outcomes.json is not valid JSON: {e}"))?;
    let obj = val
        .as_object()
        .ok_or_else(|| "outcomes.json is not a JSON object".to_string())?;
    let arr = obj
        .get("outcomes")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "outcomes.json has no `outcomes` array".to_string())?;

    let num = |k: &str| obj.get(k).and_then(serde_json::Value::as_u64).unwrap_or(0);

    let entries = arr.iter().map(parse_entry).collect::<Vec<_>>();

    Ok(MutationOutcomes {
        total_mutants: num("total_mutants"),
        caught: num("caught"),
        missed: num("missed"),
        timeout: num("timeout"),
        unviable: num("unviable"),
        version: obj
            .get("cargo_mutants_version")
            .and_then(|v| v.as_str())
            .map(str::to_string),
        entries,
    })
}

fn parse_entry(v: &serde_json::Value) -> OutcomeEntry {
    let scenario = v.get("scenario");
    let (kind, file) = match scenario {
        Some(serde_json::Value::String(_)) => (ScenarioKind::Baseline, None),
        Some(serde_json::Value::Object(o)) => {
            let file = o
                .get("Mutant")
                .and_then(|m| m.get("file"))
                .and_then(|f| f.as_str())
                .map(str::to_string);
            (ScenarioKind::Mutant, file)
        }
        _ => (ScenarioKind::Mutant, None),
    };
    let summary = v
        .get("summary")
        .and_then(|s| s.as_str())
        .unwrap_or("")
        .to_string();
    let phases = v
        .get("phase_results")
        .and_then(|p| p.as_array())
        .map(|a| a.iter().map(parse_phase).collect())
        .unwrap_or_default();
    OutcomeEntry {
        kind,
        summary,
        file,
        phases,
    }
}

fn parse_phase(v: &serde_json::Value) -> PhaseResult {
    PhaseResult {
        phase: v
            .get("phase")
            .and_then(|p| p.as_str())
            .unwrap_or("")
            .to_string(),
        succeeded: v.get("process_status").and_then(|s| s.as_str()) == Some("Success"),
    }
}

/// Locate and parse `<dir>/outcomes.json`.
///
/// # Errors
/// Returns a message when the file cannot be read or parsed.
pub fn read_outcomes_dir(dir: &Path) -> Result<MutationOutcomes, String> {
    let path = dir.join("outcomes.json");
    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    parse_outcomes(&text)
}

/// Where the gate looks for the cargo-mutants artifact: `$PMAT_MUTANTS_OUT`, or
/// `<project>/mutants.out`.
#[must_use]
pub fn outcomes_dir_for(project_path: &Path) -> PathBuf {
    match std::env::var_os("PMAT_MUTANTS_OUT") {
        Some(v) if !v.is_empty() => PathBuf::from(v),
        _ => project_path.join("mutants.out"),
    }
}

// ── the gate ───────────────────────────────────────────────────────────────

fn finding(invariant: &'static str, message: String) -> GateFinding {
    GateFinding { invariant, message }
}

/// INV-MUT-3: does the artifact corroborate that a real backend really ran?
fn backend_integrity(o: &MutationOutcomes) -> Vec<GateFinding> {
    let mut out = Vec::new();

    if o.version.is_none() {
        out.push(finding(
            "INV-MUT-3",
            "outcomes.json carries no `cargo_mutants_version`, so the result cannot be \
             attributed to a real mutation backend"
                .into(),
        ));
    }

    let baseline = o.entries.iter().find(|e| e.kind == ScenarioKind::Baseline);
    match baseline {
        None => out.push(finding(
            "INV-MUT-3",
            "no unmutated baseline scenario: nothing establishes that the test suite passes \
             before mutation, so `caught` counts are meaningless"
                .into(),
        )),
        Some(b) if b.summary != "Success" => out.push(finding(
            "INV-MUT-3",
            format!(
                "unmutated baseline summary is `{}`, not `Success`",
                b.summary
            ),
        )),
        Some(b) if !b.phases.iter().any(|p| p.phase == "Test" && p.succeeded) => {
            out.push(finding(
                "INV-MUT-3",
                "the baseline reports Success without a successful Test phase".into(),
            ));
        }
        Some(_) => {}
    }

    let mutant_entries: Vec<_> = o
        .entries
        .iter()
        .filter(|e| e.kind == ScenarioKind::Mutant)
        .collect();

    if mutant_entries.len() as u64 != o.total_mutants {
        out.push(finding(
            "INV-MUT-3",
            format!(
                "headline total_mutants={} contradicts {} enumerated mutant scenarios",
                o.total_mutants,
                mutant_entries.len()
            ),
        ));
    }

    let mut tally = (0u64, 0u64, 0u64, 0u64);
    for e in &mutant_entries {
        match e.summary.as_str() {
            "CaughtMutant" => tally.0 += 1,
            "MissedMutant" => tally.1 += 1,
            "Timeout" => tally.2 += 1,
            "Unviable" => tally.3 += 1,
            other => out.push(finding(
                "INV-MUT-3",
                format!("unrecognised mutant summary `{other}`"),
            )),
        }
    }
    if (tally.0, tally.1, tally.2, tally.3) != (o.caught, o.missed, o.timeout, o.unviable) {
        out.push(finding(
            "INV-MUT-3",
            format!(
                "headline counts (caught={}, missed={}, timeout={}, unviable={}) contradict the \
                 enumerated outcomes (caught={}, missed={}, timeout={}, unviable={})",
                o.caught, o.missed, o.timeout, o.unviable, tally.0, tally.1, tally.2, tally.3
            ),
        ));
    }

    // Per-mutant evidence. A mutant that was judged by running tests must carry
    // a Test phase; an Unviable one must carry a failed Build phase. A stub that
    // emits summaries with no phase_results fails here.
    let mut no_evidence = 0usize;
    for e in &mutant_entries {
        let ok = match e.summary.as_str() {
            "CaughtMutant" | "MissedMutant" | "Timeout" => {
                e.phases.iter().any(|p| p.phase == "Test")
            }
            "Unviable" => e.phases.iter().any(|p| p.phase == "Build" && !p.succeeded),
            _ => false,
        };
        if !ok {
            no_evidence += 1;
        }
    }
    if no_evidence > 0 {
        out.push(finding(
            "INV-MUT-3",
            format!(
                "{no_evidence} mutant outcome(s) carry no build/test phase evidence — the verdict \
                 was asserted, not executed"
            ),
        ));
    }

    out
}

/// Decide the mutation gate.
///
/// `outcomes` is `None` when no artifact was produced. `scope` describes the
/// diff the run was supposed to cover.
#[must_use]
pub fn evaluate_mutation_gate(
    outcomes: Option<&MutationOutcomes>,
    scope: &DiffScope,
) -> MutationGateVerdict {
    let mut findings = Vec::new();

    let Some(o) = outcomes else {
        // No artifact at all. Legitimate only when the diff provably changes
        // nothing cargo-mutants would mutate.
        if *scope == DiffScope::NoMutableRustSource {
            return MutationGateVerdict {
                passed: true,
                findings,
                summary: "diff changes no mutable Rust source; no mutants were required".into(),
            };
        }
        findings.push(finding(
            "INV-MUT-2",
            "no cargo-mutants artifact (outcomes.json) was produced or it is unreadable, so \
             mutation adequacy is UNMEASURED — an unmeasurable metric is a failure, not a pass"
                .into(),
        ));
        return verdict(findings, 0, 0);
    };

    // INV-MUT-3 first: a degraded backend's numbers must not be believed.
    findings.extend(backend_integrity(o));

    // Scope integrity. Every mutant must live in a file the diff changed;
    // otherwise the artifact is stale or from a different run. This is the
    // check that catches cargo-mutants leaving an old mutants.out in place when
    // `--in-diff` selects nothing.
    if let DiffScope::MutableRustSource(files) = scope {
        let stray: Vec<&str> = o
            .entries
            .iter()
            .filter(|e| e.kind == ScenarioKind::Mutant)
            .filter_map(|e| e.file.as_deref())
            .filter(|f| !files.contains(*f))
            .collect();
        if !stray.is_empty() {
            let mut uniq: Vec<&str> = stray.clone();
            uniq.sort_unstable();
            uniq.dedup();
            findings.push(finding(
                "INV-MUT-3",
                format!(
                    "{} mutant(s) are in file(s) the diff does not touch (e.g. {}) — the artifact \
                     does not describe this diff and may be stale",
                    stray.len(),
                    uniq.iter().take(3).copied().collect::<Vec<_>>().join(", ")
                ),
            ));
        }
    }

    // INV-MUT-2: an empty (or wholly unexecuted) mutant set is not a pass.
    if o.total_mutants == 0 {
        match scope {
            DiffScope::NoMutableRustSource => {}
            DiffScope::Unknown => findings.push(finding(
                "INV-MUT-2",
                "0 mutants were generated and no diff was supplied to prove that is legitimate — \
                 an empty mutant set is not a pass"
                    .into(),
            )),
            DiffScope::MutableRustSource(files) => findings.push(finding(
                "INV-MUT-2",
                format!(
                    "0 mutants were generated although the diff changes {} mutable Rust source \
                     file(s)",
                    files.len()
                ),
            )),
        }
    } else if o.caught + o.missed + o.timeout == 0 {
        findings.push(finding(
            "INV-MUT-2",
            format!(
                "{} mutants were generated but none was executed ({} unviable) — nothing was \
                 tested",
                o.total_mutants, o.unviable
            ),
        ));
    }

    // INV-MUT-1: survivors in the diff fail the build.
    if o.missed > 0 || o.timeout > 0 {
        let survivors: Vec<String> = o
            .entries
            .iter()
            .filter(|e| e.summary == "MissedMutant" || e.summary == "Timeout")
            .filter_map(|e| e.file.clone())
            .take(3)
            .collect();
        findings.push(finding(
            "INV-MUT-1",
            format!(
                "{} mutant(s) survived and {} timed out in the changed code{}",
                o.missed,
                o.timeout,
                if survivors.is_empty() {
                    String::new()
                } else {
                    format!(" (e.g. {})", survivors.join(", "))
                }
            ),
        ));
    }

    verdict(findings, o.caught, o.total_mutants)
}

fn verdict(findings: Vec<GateFinding>, caught: u64, total: u64) -> MutationGateVerdict {
    let passed = findings.is_empty();
    let summary = if passed {
        format!("{caught}/{total} mutants caught in the changed code")
    } else {
        findings
            .iter()
            .map(|f| format!("{}: {}", f.invariant, f.message))
            .collect::<Vec<_>>()
            .join("; ")
    };
    MutationGateVerdict {
        passed,
        findings,
        summary,
    }
}

/// Run the whole gate for a project: locate the artifact, classify the diff,
/// evaluate. `diff_path` is the unified diff the run covered (typically the one
/// passed to `cargo mutants --in-diff`); `None` means the scope is unknown, and
/// the gate fails closed on an empty mutant set.
#[must_use]
pub fn run_mutation_gate(project_path: &Path, diff_path: Option<&Path>) -> MutationGateVerdict {
    let excludes = load_exclude_globs(project_path);
    let scope = match diff_path {
        None => DiffScope::Unknown,
        Some(p) => match std::fs::read_to_string(p) {
            Ok(text) => classify_diff(&text, &excludes),
            // An unreadable diff is an unmeasurable scope: fail closed by
            // treating it as unknown rather than as "nothing changed".
            Err(_) => DiffScope::Unknown,
        },
    };
    let dir = outcomes_dir_for(project_path);
    match read_outcomes_dir(&dir) {
        Ok(o) => evaluate_mutation_gate(Some(&o), &scope),
        Err(_) => evaluate_mutation_gate(None, &scope),
    }
}

/// Where the gate expects the diff, when the caller does not pass one:
/// `$PMAT_MUTATION_DIFF`.
#[must_use]
pub fn diff_path_from_env() -> Option<PathBuf> {
    match std::env::var_os("PMAT_MUTATION_DIFF") {
        Some(v) if !v.is_empty() => Some(PathBuf::from(v)),
        _ => None,
    }
}

#[cfg(test)]
#[path = "mutation_gate_tests.rs"]
mod mutation_gate_tests;
