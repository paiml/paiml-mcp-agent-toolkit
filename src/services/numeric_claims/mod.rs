//! CB-2104 `pmat comply numeric-claims` — numbers a repository writes down
//! about itself, judged against each other.
//!
//! Two rules over one corpus scanner. They cover disjoint surfaces and neither
//! subsumes the other:
//!
//! * **R1 REPLICATED DIVERGENT CLAIM** ([`cohort`]) — N files carry the same
//!   sentence with a number in it and the numbers disagree, so at least N − m
//!   of them are wrong. Reads unnamed prose numerals under an identical
//!   sentence template.
//! * **R2 CONTRADICTION** — two *named* things in the tree disagree about one
//!   quantity: a config key and its own trailing comment, a `const` and the
//!   file it says it mirrors.
//!
//! Severity is WARN. The check never blocks: findings exit 0. Only
//! UNMEASURABLE — an empty or unreadable corpus — exits 2, because "we could
//! not measure it" must never read as "it did not regress".
//!
//! R1's lane is layered pure/impure the way [`super::metrics_ratchet`] is:
//! [`frame`] and [`cohort`] never touch a file, a clock or a subprocess, so
//! every guard can be driven — and ablated — from a literal. [`corpus`] is the
//! impure half: `git ls-files`, the exclusion list, and the generated-file
//! detection that G1 needs.

//! R2's lane is layered the same way: [`extract`] turns bytes into
//! `(quantity, value, annotation)` triples, [`annotate`] classifies the
//! annotation, [`rules`] holds C1..C5. None of the three opens a path.

pub mod annotate;
pub mod census;
pub mod cohort;
pub mod corpus;
pub mod extract;
pub mod frame;
pub mod render;
pub mod rules;

#[cfg(test)]
mod annotate_tests;
#[cfg(test)]
mod census_tests;
#[cfg(test)]
mod cohort_tests;
#[cfg(test)]
mod corpus_tests;
#[cfg(test)]
mod extract_tests;
#[cfg(test)]
mod frame_tests;
#[cfg(test)]
mod render_tests;
#[cfg(test)]
mod rules_tests;

use serde::Serialize;

/// One in-scope file, already read into memory.
///
/// R2 never opens a path itself. The corpus lane decides *what* is in scope
/// (`git ls-files`, the exclusion list, G1 generated-file detection); R2
/// decides what the bytes *say*. That split is what lets every rule below be
/// driven — and ablated — from a string literal in a test.
#[derive(Debug, Clone)]
pub struct CorpusFile {
    /// Repo-relative path, as `git ls-files` prints it.
    pub path: String,
    /// Full file contents.
    pub text: String,
}

impl CorpusFile {
    /// Build a corpus entry from anything string-shaped.
    pub fn new(path: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            text: text.into(),
        }
    }
}

/// Which rule produced a [`Finding`].
///
/// `R1` belongs to the cohort lane; `C1`..`C5` are R2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum RuleId {
    /// R1 replicated divergent claim.
    R1,
    /// C1 self-breach: a limit whose own annotation reports a violating observation.
    C1,
    /// C2 restatement mismatch: the annotation restates the value and disagrees.
    C2,
    /// C3 arithmetic: a stated headroom percentage the numbers do not support.
    C3,
    /// C4 unjustified divergence across sibling files of one schema.
    C4,
    /// C5 named cross-reference: "aligned with X" where X holds a different value.
    C5,
}

impl RuleId {
    /// Stable identifier used in text and JSON output.
    pub fn as_str(self) -> &'static str {
        match self {
            RuleId::R1 => "R1",
            RuleId::C1 => "C1",
            RuleId::C2 => "C2",
            RuleId::C3 => "C3",
            RuleId::C4 => "C4",
            RuleId::C5 => "C5",
        }
    }

    /// Human-readable rule name.
    pub fn title(self) -> &'static str {
        match self {
            RuleId::R1 => "REPLICATED DIVERGENT CLAIM",
            RuleId::C1 => "SELF-BREACH",
            RuleId::C2 => "RESTATEMENT MISMATCH",
            RuleId::C3 => "ARITHMETIC",
            RuleId::C4 => "UNJUSTIFIED DIVERGENCE",
            RuleId::C5 => "NAMED CROSS-REFERENCE",
        }
    }
}

/// One place in the tree that states a value.
#[derive(Debug, Clone, Serialize)]
pub struct Site {
    /// Repo-relative path.
    pub file: String,
    /// 1-indexed line.
    pub line: usize,
    /// The evaluated value, in the units the site wrote it in.
    pub value: f64,
    /// The source line, trimmed and truncated.
    pub text: String,
}

/// One reported contradiction or divergence.
///
/// `wrong_floor` is a *floor* on how many sites must be wrong, never a count of
/// wrong sites: when two sites disagree at most one can be right, but the rule
/// cannot say which. `anchored` records whether any site carries a declared
/// reproducing predicate — R2 never resolves one, so it is `false` today.
#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    /// Which rule fired.
    pub rule: RuleId,
    /// The quantity in dispute — a section-qualified key, or a template.
    pub quantity: String,
    /// Every site that participates in the disagreement.
    pub sites: Vec<Site>,
    /// Lower bound on the number of wrong sites.
    pub wrong_floor: usize,
    /// Whether the quantity has a declared reproducing predicate.
    pub anchored: bool,
    /// One-line statement of the contradiction.
    pub detail: String,
    /// The annotation text that fired the rule, verbatim.
    pub evidence: String,
    /// What to do about it.
    pub fix: String,
}

/// Proof the check ran, emitted unconditionally.
///
/// A "clean" result that does not carry these counters is a bug, not a pass:
/// the researched design's report for *"I analysed 12,693 numbers and found
/// nothing"* was byte-identical to *"`git ls-files` returned nothing"*.
/// Suppression counters are here for the same reason — a guard that can hide a
/// finding must say how often it did.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Census {
    /// Files handed to R2 — every scanned type, JSON included.
    pub files_scanned: usize,
    /// Files handed to R1, which does not read JSON.
    ///
    /// A separate counter rather than a footnote: the two rules run over
    /// different corpora, and one `files_scanned` covering both would make the
    /// coverage percentages beneath it unreadable.
    pub r1_files_scanned: usize,
    /// Files tracked by git in the scanned types, before exclusions.
    pub files_tracked: usize,
    /// R1: numerals surviving measurement framing. Owned by the cohort lane.
    pub r1_framed_numerals: usize,
    /// R1: cohorts with at least two sites. Owned by the cohort lane.
    pub r1_cohorts_min2: usize,
    /// R1: cohorts at or above `--min-sites`. Owned by the cohort lane.
    pub r1_cohorts_at_min_sites: usize,
    /// R2: `(quantity, value, annotation)` triples extracted.
    pub r2_mentions: usize,
    /// R2: annotations that survived the assertiveness gate.
    pub r2_assertive_annotations: usize,
    /// R1 G1: sites dropped as machine-generated. Owned by the cohort lane.
    pub suppressed_generated: usize,
    /// R1 G2: templates dropped for varying in more than one slot.
    pub suppressed_multi_slot: usize,
    /// R2 C2: restatements suppressed because the annotation derives the value.
    pub suppressed_derivation: usize,
    /// R2 C1/C2: rules that did not fire because some unit reading agreed.
    pub suppressed_unit_ambiguity: usize,
    /// R2 C5: cross-references whose name did not resolve to exactly one site.
    pub suppressed_unresolved_xref: usize,
    /// Numeric literals present in the scanned files, whatever their role.
    pub raw_numeric_literals: usize,
    /// Tracked files dropped as machine-managed (lockfiles, vendor, build output).
    pub excluded_machine_managed: usize,
    /// Tracked files dropped as a fixture or generated-state tree.
    pub excluded_fixture_tree: usize,
    /// Tracked files dropped as a record of PAST state (`CHANGELOG*`).
    ///
    /// Every exclusion is a place where a wrong number survives, so the census
    /// has to be able to say how many. [`super::corpus`]'s module note promised
    /// exactly this and nothing printed it until CB-2104 had a renderer.
    pub excluded_changelog: usize,
    /// Tracked, matched, and unreadable. Counted so a vanished corpus cannot
    /// read as an empty one.
    pub unreadable: usize,
    /// Wall time, milliseconds.
    pub elapsed_ms: u128,
}

/// Whether the run produced a usable measurement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Status {
    /// The corpus was measured. Findings, if any, are advisory.
    Ok,
    /// The corpus could not be measured. Never report this as clean.
    Unmeasurable,
}

/// The whole result of one `pmat comply numeric-claims` run.
#[derive(Debug, Clone, Serialize)]
pub struct NumericClaimsReport {
    /// Check identifier.
    pub check: &'static str,
    /// Always `"warn"`. This check never blocks.
    pub severity: &'static str,
    /// Measured or not.
    pub status: Status,
    /// Everything the rules found.
    pub findings: Vec<Finding>,
    /// Proof the check ran.
    pub census: Census,
    /// Precision warnings (lowered `--min-sites`, guards disabled, …).
    pub warnings: Vec<String>,
    /// What the check found when it ran itself against the committed fixture,
    /// before it looked at the real corpus. A green run always carries this
    /// proof that the rules can still fire.
    pub self_test: census::SelfTest,
}

impl NumericClaimsReport {
    /// Process exit code.
    ///
    /// Findings exit 0 — the check is advisory and never blocks. Only
    /// [`Status::Unmeasurable`] exits 2.
    pub fn exit_code(&self) -> i32 {
        match self.status {
            Status::Ok => 0,
            Status::Unmeasurable => 2,
        }
    }
}

/// Everything R2 produces over one corpus.
#[derive(Debug, Clone, Default)]
pub struct R2Outcome {
    /// C1..C5 findings, in rule order.
    pub findings: Vec<Finding>,
    /// R2's half of the census. Fields owned by R1 are left at zero.
    pub census: Census,
}

/// Run R2 — the contradiction rules — over an in-memory corpus.
///
/// This is the whole R2 entry point: the caller owns file discovery, R2 owns
/// what the bytes say.
pub fn run_r2(files: &[CorpusFile]) -> R2Outcome {
    rules::run(files)
}
