//! The proof CB-2104 ran, and its refusal to report a result it did not earn.
//!
//! The researched design failed one test outright: its report for *"I analysed
//! 12,693 numbers and found nothing"* was byte-identical to *"`git ls-files`
//! returned nothing and I analysed nothing."* A check that cannot tell those
//! apart is worse than no check, because the second reads as the first.
//!
//! Three layers, all of them mandatory and all of them here:
//!
//! 1. **The census is unconditional and machine-readable.** A "clean" result
//!    that does not carry `files_scanned`, `r1_framed_numerals`, `r2_mentions`
//!    and the suppression counters is a bug, not a pass.
//! 2. **Plausibility, in `.pmat-ratchet.toml`'s idiom** — a metric that
//!    measures 0 against a baseline above 0 is UNMEASURABLE, not passed. A
//!    genuinely empty repository is UNMEASURABLE too: there is nothing to say,
//!    and "no contradictions" would be a claim the run did not earn.
//! 3. **The self-test fixture, on every invocation.** Before it looks at the
//!    real corpus the check runs itself against
//!    [`FIXTURE_ROOT`], which plants one defect per rule family among 36
//!    numbers that must stay silent. Anything short of 4/4 recovered with 0
//!    false positives is UNMEASURABLE, exit 2, and the real result is not
//!    printed at all.
//!
//! The third layer is the control the researched designs lacked. A rule that
//! has silently stopped firing and a repository that is genuinely clean produce
//! the same empty output; only a corpus that MUST fire can separate them. It is
//! also the answer to the harness-bug class: 11 of this project's first
//! flag-efficacy findings were bugs in the harness rather than in the tree, and
//! a fixture that must round-trip catches those before they become findings.
//!
//! The fixture is `include_str!`-ed into the binary rather than read from disk,
//! so it travels with the executable and the self-test works when scanning any
//! repository. It always runs at [`CohortConfig::default`], never at the user's
//! settings: it is a control on the rules, not on the configuration, and a
//! `--min-sites 20` that made the self-test fail would be reporting the wrong
//! thing.

use std::path::Path;
use std::time::Instant;

use serde::Serialize;

use super::cohort::{self, CohortConfig};
use super::corpus::{self, R2_PATHSPECS};
use super::rules;
use super::{Census, CorpusFile, Finding, NumericClaimsReport, RuleId, Status};

/// Where the committed fixture lives, relative to the repository root.
pub const FIXTURE_ROOT: &str = "tests/fixtures/numeric_claims";

/// Innocent numbers plus correct derivations in the fixture's clean half.
///
/// 26 hand-planted innocent numbers — HTTP statuses, ports, a seed, a year, an
/// edition, an arXiv id, a section heading, a table row, a past-state record, a
/// policy target — and the 10 hand-audited correct derivations.
pub const INNOCENT_ITEMS: usize = 36;

/// Below this many files a quiet corpus is small, not broken.
///
/// The plausibility rules only accuse the extractor of having rotted once there
/// is enough corpus for the accusation to mean something.
pub const PLAUSIBILITY_FLOOR: usize = 20;

/// One planted defect, and how a finding is recognised as having recovered it.
#[derive(Debug, Clone, Copy)]
pub struct Planted {
    /// The rule family this defect exercises.
    pub rule: RuleId,
    /// Every fixture path belonging to this defect starts with this.
    pub file_prefix: &'static str,
    /// Substring the finding's `quantity` must contain.
    pub quantity: &'static str,
    /// What is planted, for the failure message.
    pub what: &'static str,
}

/// The four planted defects — one per rule family.
///
/// One per family, because a fixture that exercises three rules proves nothing
/// about the fourth: its silence on a real corpus would be indistinguishable
/// from a rule that no longer runs.
pub const PLANTED: &[Planted] = &[
    Planted {
        rule: RuleId::C1,
        file_prefix: "planted/.pmat-metrics.toml",
        quantity: "quality_gates.max_unwrap_calls",
        what: "a ceiling of 100 whose own trailing comment reports 570",
    },
    Planted {
        rule: RuleId::C4,
        file_prefix: "planted/codecov.yml",
        quantity: "coverage.status.project.default.threshold",
        what: "95% where both sibling codecov.yml files say 2%",
    },
    Planted {
        rule: RuleId::C5,
        file_prefix: "planted/binary_size.rs",
        quantity: "binary_max_bytes",
        what: "52,428,800 declared aligned with a key holding 50,000,000",
    },
    Planted {
        rule: RuleId::R1,
        file_prefix: "planted/crate-",
        quantity: "workspace crates",
        what: "one sentence in seven files: 70 in five of them, 75 in two",
    },
];

/// The result of running the check against its own fixture.
#[derive(Debug, Clone, Default, Serialize)]
pub struct SelfTest {
    /// 4/4 recovered, and nothing else reported.
    pub passed: bool,
    /// How many defects the fixture plants.
    pub planted: usize,
    /// How many of them came back.
    pub recovered: usize,
    /// Innocent numbers and derivations in the fixture's clean half.
    pub innocent_items: usize,
    /// Planted defects that did not come back.
    pub missed: Vec<String>,
    /// Findings that named a file in the innocent half.
    pub false_positives: Vec<String>,
    /// Findings matching no planted defect and naming no innocent file.
    pub unexpected: Vec<String>,
    /// Everything the fixture run produced.
    pub findings: Vec<Finding>,
}

impl SelfTest {
    /// One line for the census and for the UNMEASURABLE reason.
    pub fn summary(&self) -> String {
        let mut s = format!(
            "{}/{} planted defects recovered, {}/{} innocent numbers flagged",
            self.recovered,
            self.planted,
            self.false_positives.len(),
            self.innocent_items
        );
        if !self.missed.is_empty() {
            s.push_str(&format!("; missed {:?}", self.missed));
        }
        if !self.unexpected.is_empty() {
            s.push_str(&format!("; unexpected {:?}", self.unexpected));
        }
        s
    }
}

/// Why a run could not be measured.
///
/// Every variant here exits 2. None of them is a clean tree, and the renderer
/// must never present one as such.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Vacuity {
    /// No file in the scanned types. Nothing to say is not the same as nothing
    /// wrong.
    EmptyCorpus,
    /// A real corpus produced no framed numeral: R1's extractor has rotted.
    NoFramedNumerals,
    /// A real corpus produced no named quantity: R2's extractor has rotted.
    NoMentions,
    /// The committed fixture did not round-trip, so no result can be trusted.
    SelfTestFailed(String),
    /// `git ls-files` refused, or the tree could not be read.
    CorpusUnreadable(String),
}

impl Vacuity {
    /// Why this run is UNMEASURABLE, in one sentence a human can act on.
    pub fn reason(&self) -> String {
        match self {
            Self::EmptyCorpus => "no tracked file in the scanned types: there is nothing to \
                 say, which is not the same as there being nothing wrong"
                .to_string(),
            Self::NoFramedNumerals => format!(
                "more than {PLAUSIBILITY_FLOOR} files scanned and not one numeral survived \
                 measurement framing — R1's extractor has rotted"
            ),
            Self::NoMentions => format!(
                "more than {PLAUSIBILITY_FLOOR} files scanned and not one named quantity was \
                 extracted — R2's extractor has rotted"
            ),
            Self::SelfTestFailed(detail) => format!(
                "the committed self-test fixture did not round-trip ({detail}), so no result \
                 from the real corpus can be trusted"
            ),
            Self::CorpusUnreadable(detail) => format!("the corpus could not be read: {detail}"),
        }
    }
}

macro_rules! fixture {
    ($($p:literal),* $(,)?) => {
        vec![$(CorpusFile::new(
            $p,
            include_str!(concat!("../../../tests/fixtures/numeric_claims/", $p)),
        ),)*]
    };
}

/// The half of the fixture that must fire.
pub fn planted_corpus() -> Vec<CorpusFile> {
    fixture![
        "planted/.pmat-metrics.toml",
        "planted/binary_size.rs",
        "planted/codecov.yml",
        "planted/a/codecov.yml",
        "planted/b/codecov.yml",
        "planted/crate-a/README.md",
        "planted/crate-b/README.md",
        "planted/crate-c/README.md",
        "planted/crate-d/README.md",
        "planted/crate-e/README.md",
        "planted/crate-f/README.md",
        "planted/crate-g/README.md",
    ]
}

/// The half of the fixture that must stay silent.
///
/// Each of the 26 innocent classes is replicated across eight files with a 6/2
/// value split — the exact shape R1 hunts — so silence here is a property of
/// the framing rules and not of a small corpus. An innocent control that could
/// not fire even if the rules rotted would measure nothing.
pub fn innocent_corpus() -> Vec<CorpusFile> {
    fixture![
        "innocent/site0.md",
        "innocent/site1.md",
        "innocent/site2.md",
        "innocent/site3.md",
        "innocent/site4.md",
        "innocent/site5.md",
        "innocent/site6.md",
        "innocent/site7.md",
        "innocent/site0.toml",
        "innocent/site1.toml",
        "innocent/site2.toml",
        "innocent/site3.toml",
        "innocent/site4.toml",
        "innocent/site5.toml",
        "innocent/site6.toml",
        "innocent/site7.toml",
        "innocent/site0.yaml",
        "innocent/site1.yaml",
        "innocent/site2.yaml",
        "innocent/site3.yaml",
        "innocent/site4.yaml",
        "innocent/site5.yaml",
        "innocent/site6.yaml",
        "innocent/site7.yaml",
        "innocent/site0.rs",
        "innocent/site1.rs",
        "innocent/site2.rs",
        "innocent/site3.rs",
        "innocent/site4.rs",
        "innocent/site5.rs",
        "innocent/site6.rs",
        "innocent/site7.rs",
        "innocent/derivations.rs",
    ]
}

/// Both halves, as one corpus.
pub fn fixture_corpus() -> Vec<CorpusFile> {
    let mut files = planted_corpus();
    files.extend(innocent_corpus());
    files
}

/// Does this finding recover that planted defect?
fn recovers(f: &Finding, p: &Planted) -> bool {
    f.rule == p.rule
        && f.quantity.contains(p.quantity)
        && f.sites.iter().any(|s| s.file.starts_with(p.file_prefix))
}

/// One line naming a finding, for the self-test's failure lists.
fn describe(f: &Finding) -> String {
    let where_ = f
        .sites
        .first()
        .map(|s| format!("{}:{}", s.file, s.line))
        .unwrap_or_else(|| "<no site>".to_string());
    format!("[{}] {} at {where_}", f.rule.as_str(), f.quantity)
}

/// Run the rules over a fixture corpus and judge the result.
///
/// Exposed so the control on the control can be written: ablate one planted
/// defect and this must go red. A self-test that cannot fail is decoration.
pub fn self_test_over(files: &[CorpusFile]) -> SelfTest {
    let (findings, _, _) = run_corpus(files.to_vec(), &CohortConfig::default());
    let mut st = SelfTest {
        planted: PLANTED.len(),
        innocent_items: INNOCENT_ITEMS,
        ..SelfTest::default()
    };
    for p in PLANTED {
        if findings.iter().any(|f| recovers(f, p)) {
            st.recovered += 1;
        } else {
            st.missed
                .push(format!("{} {} — {}", p.rule.as_str(), p.quantity, p.what));
        }
    }
    for f in &findings {
        if f.sites.iter().any(|s| s.file.starts_with("innocent/")) {
            st.false_positives.push(describe(f));
        } else if !PLANTED.iter().any(|p| recovers(f, p)) {
            st.unexpected.push(describe(f));
        }
    }
    st.passed = st.missed.is_empty() && st.false_positives.is_empty() && st.unexpected.is_empty();
    st.findings = findings;
    st
}

/// Run the check against its committed fixture.
pub fn self_test() -> SelfTest {
    self_test_over(&fixture_corpus())
}

/// Is this census the product of a measurement, or of a rotted extractor?
///
/// Stated in `.pmat-ratchet.toml`'s idiom: a metric that measures 0 against a
/// baseline above 0 is UNMEASURABLE, not passed. Returns `None` when the run
/// may be reported.
pub fn plausibility(census: &Census) -> Option<Vacuity> {
    if census.files_scanned == 0 {
        return Some(Vacuity::EmptyCorpus);
    }
    if census.files_scanned > PLAUSIBILITY_FLOOR {
        if census.r1_framed_numerals == 0 {
            return Some(Vacuity::NoFramedNumerals);
        }
        if census.r2_mentions == 0 {
            return Some(Vacuity::NoMentions);
        }
    }
    None
}

/// Run both rules over one in-memory corpus and merge their censuses.
///
/// R2 reads every scanned type; R1 does not read JSON. Rather than walk the
/// tree twice, the corpus is partitioned in place — R1's files first — so R1
/// can be handed a prefix slice and nothing is copied.
///
/// `files_scanned` is R2's number and `r1_files_scanned` is R1's; the two are
/// merged by overwrite, never by addition, because both lanes count the same
/// files.
pub fn run_corpus(
    mut files: Vec<CorpusFile>,
    cfg: &CohortConfig,
) -> (Vec<Finding>, Census, Vec<String>) {
    files.sort_by_key(|f| !corpus::in_r1_corpus(&f.path));
    let r1_len = files.partition_point(|f| corpus::in_r1_corpus(&f.path));

    let r2 = rules::run(&files);
    let r1 = cohort::run_r1(&files[..r1_len], cfg);

    let census = Census {
        files_scanned: files.len(),
        r1_files_scanned: r1_len,
        files_tracked: 0,
        r1_framed_numerals: r1.census.r1_framed_numerals,
        r1_cohorts_min2: r1.census.r1_cohorts_min2,
        r1_cohorts_at_min_sites: r1.census.r1_cohorts_at_min_sites,
        r2_mentions: r2.census.r2_mentions,
        r2_assertive_annotations: r2.census.r2_assertive_annotations,
        suppressed_generated: r1.census.suppressed_generated,
        suppressed_multi_slot: r1.census.suppressed_multi_slot,
        suppressed_derivation: r2.census.suppressed_derivation,
        suppressed_unit_ambiguity: r2.census.suppressed_unit_ambiguity,
        suppressed_unresolved_xref: r2.census.suppressed_unresolved_xref,
        raw_numeric_literals: r2.census.raw_numeric_literals,
        // Filled by `run` from the corpus pass. This function is handed files
        // that already survived the exclusion list, so it cannot count what it
        // never saw — and the literal above is exhaustive on purpose, so a new
        // census field fails the build here rather than shipping as a zero.
        excluded_machine_managed: 0,
        excluded_fixture_tree: 0,
        excluded_changelog: 0,
        unreadable: 0,
        elapsed_ms: 0,
    };

    let mut findings = r1.findings;
    findings.extend(r2.findings);
    (findings, census, r1.warnings)
}

/// The report for a run that produced no usable measurement.
///
/// Findings are dropped rather than printed: when the check cannot vouch for
/// its own rules, a list of things it thinks it found is worse than silence.
fn unmeasurable(
    reason: Vacuity,
    census: Census,
    self_test: SelfTest,
    warnings: Vec<String>,
) -> NumericClaimsReport {
    let mut warnings = warnings;
    warnings.insert(0, reason.reason());
    NumericClaimsReport {
        check: "CB-2104",
        severity: "warn",
        status: Status::Unmeasurable,
        findings: Vec::new(),
        census,
        warnings,
        self_test,
    }
}

/// Run CB-2104 over a repository.
///
/// The self-test comes first, so a rule that has stopped firing is caught
/// before it can report a clean tree. Findings exit 0 whatever their count;
/// only [`Status::Unmeasurable`] exits 2.
pub fn run(root: &Path, cfg: &CohortConfig) -> NumericClaimsReport {
    let started = Instant::now();

    let st = self_test();
    if !st.passed {
        let summary = st.summary();
        let census = Census {
            elapsed_ms: started.elapsed().as_millis(),
            ..Census::default()
        };
        return unmeasurable(Vacuity::SelfTestFailed(summary), census, st, Vec::new());
    }

    let (files, corpus_census) = match corpus::collect(root, R2_PATHSPECS) {
        Ok(v) => v,
        Err(e) => {
            let census = Census {
                elapsed_ms: started.elapsed().as_millis(),
                ..Census::default()
            };
            return unmeasurable(
                Vacuity::CorpusUnreadable(e.to_string()),
                census,
                st,
                Vec::new(),
            );
        }
    };

    let (findings, mut census, warnings) = run_corpus(files, cfg);
    census.files_tracked = corpus_census.tracked;
    census.excluded_machine_managed = corpus_census.excluded_machine_managed;
    census.excluded_fixture_tree = corpus_census.excluded_fixture_tree;
    census.excluded_changelog = corpus_census.excluded_changelog;
    census.unreadable = corpus_census.unreadable;
    census.elapsed_ms = started.elapsed().as_millis();

    if let Some(reason) = plausibility(&census) {
        return unmeasurable(reason, census, st, warnings);
    }

    NumericClaimsReport {
        check: "CB-2104",
        severity: "warn",
        status: Status::Ok,
        findings,
        census,
        warnings,
        self_test: st,
    }
}
