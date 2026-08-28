//! R1 — REPLICATED DIVERGENT CLAIM.
//!
//! > *N files contain the same sentence with a number in it, and the numbers
//! > disagree. At most one can be right, so at least N − m are wrong.*
//!
//! The specimen: aprender carries `Part of the [Aprender](…) monorepo — 70
//! workspace crates.` in 45 files, `75` in five more, and `cargo metadata
//! --no-deps` says 78. Sixty-nine of those files are per-crate READMEs, so ~69
//! crates.io package pages publish a false crate count. Nothing else in the
//! toolchain can see it: CB-2101 and the ratchet both require a human to
//! *declare* a number before it is watched, and nobody declares the copy.
//!
//! ## Cohort identity is TEMPLATE identity
//!
//! The key is the whole source line with every numeral blanked. Two weaker
//! identities were built and measured, and both were rejected:
//!
//! ```text
//! bare noun ("tests")            1,434 pmat claims, 132 distinct values —
//!                                almost all legitimate local statements
//! 2-word label ("clippy warnings")  0/4 precision: "0 clippy warnings" in a
//!                                summary against "180+ clippy warnings" in
//!                                release notes is two times, not two answers
//! ```
//!
//! ## Two guards, each derived from an audited false positive
//!
//! Without them this rule scored 1/3. Each was ablated independently and each
//! kills both false positives on its own while leaving the true positive
//! standing, so both ship on ([`Guards`] exists to keep that ablation runnable,
//! not because either is optional):
//!
//! ```text
//! G1 generated-file      a machine-written number cannot be a false claim
//! G2 single-varying-slot several varying slots means per-entity parameterised
//!                        data, not one quantity restated N times
//! ```
//!
//! G2 is the one that repairs the design's central deduction. `// Total: 657
//! preconditions, 20 postconditions, 0 invariants from 293 contracts` varies in
//! two slots because a generator writes both from one YAML file; the
//! precondition count and the contract count are not competing answers to one
//! question, so "at most one can be right" simply does not apply. The research
//! lane's own output contained that counterexample.
//!
//! ## What this rule refuses to say
//!
//! It reports a **disagreement floor** and `anchored: false`. It never reports
//! which value is right, because it cannot know: automatic anchor resolution
//! was built and measured at 8 labels on aprender and 0 on pmat, and it missed
//! the one anchor that mattered — aprender's README cites `cargo metadata
//! --no-deps`, which names a method without printing a number. R1 finds where a
//! quantity needs declaring; a human still has to declare it.
//!
//! It is also blind to the unanimous-and-wrong claim, which is the larger
//! defect: aprender has a *second* template, n = 28, every site saying `70`,
//! every one wrong. R1 needs disagreement and sees none of them.

use super::corpus::{classify_generated, collect, CorpusError, Generated, R1_PATHSPECS};
use super::frame::{frame_file, numeral_tokens, scan_numerals};
use super::{Census, Finding, RuleId, Site};
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::Path;
use std::sync::LazyLock;

/// Default `--min-sites`: seven distinct files.
///
/// Load-bearing, and the cliff was measured. At `n >= 3` — exactly what a
/// maintainer reaches for when a check never fires — output goes to five
/// findings on pmat and five on aprender, of which **one** is real. The nine
/// rejects are step numbers in per-language example workflows, sprint records
/// written weeks apart, and three tickets about three different modules.
pub const DEFAULT_MIN_SITES: usize = 7;

/// Measured precision at the floor a maintainer is most likely to try.
const LOWERED_PRECISION: &str = "1/10";

/// The two guards, switchable so each can be ablated in a test.
///
/// Both default on. `--include-generated` turns G1 off for debugging and must
/// print [`guards_warning`]; nothing turns G2 off outside the ablation tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Guards {
    /// G1: skip sites in machine-written files.
    pub generated: bool,
    /// G2: skip templates that vary in more than one numeral slot.
    pub single_varying_slot: bool,
}

impl Default for Guards {
    fn default() -> Self {
        Self {
            generated: true,
            single_varying_slot: true,
        }
    }
}

/// Stage-4's singleton-shaped distribution gate, plus the guards.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CohortConfig {
    /// Minimum distinct files carrying the template.
    pub min_sites: usize,
    /// Maximum distinct values in the varying slot. More than a handful of
    /// answers is a list, not a disagreement.
    pub max_distinct: usize,
    /// Minimum share held by the most common value. A near-even split is two
    /// populations, not one claim and its stale copies.
    pub min_modal_share: f64,
    /// Guard switches.
    pub guards: Guards,
}

impl Default for CohortConfig {
    fn default() -> Self {
        Self {
            min_sites: DEFAULT_MIN_SITES,
            max_distinct: 3,
            min_modal_share: 0.60,
            guards: Guards::default(),
        }
    }
}

/// One line that carries at least one framed numeral.
///
/// The unit of cohorting is the LINE, not the numeral: one framed numeral makes
/// the line a candidate, and then every numeral on it becomes a slot. That is
/// what lets G2 see co-variation at all.
#[derive(Debug, Clone, PartialEq)]
pub struct ClaimLine {
    /// Repo-relative path.
    pub file: String,
    /// 1-indexed line number.
    pub line: usize,
    /// The trimmed source line.
    pub context: String,
    /// G1 stamp for the file this line came from.
    pub generated: Option<Generated>,
}

/// One distinct value inside a cohort's varying slot.
#[derive(Debug, Clone)]
pub struct ValueGroup {
    /// The numeral as written.
    pub value: String,
    /// How many sites say it.
    pub count: usize,
    /// The sites that say it, in path order.
    pub sites: Vec<Site>,
}

/// One R1 finding.
#[derive(Debug, Clone)]
pub struct ReplicatedDivergence {
    /// The sentence template, every numeral blanked.
    pub template: String,
    /// Which numeral slot disagrees.
    pub slot: usize,
    /// Participating sites.
    pub n_sites: usize,
    /// Distinct files among them.
    pub n_files: usize,
    /// Lower bound on how many sites are wrong. Never a count of wrong sites.
    pub wrong_floor: usize,
    /// The values, most common first.
    pub values: Vec<ValueGroup>,
    /// Always false. R1 cannot resolve an anchor and must not imply it can.
    pub anchored: bool,
    /// Roundness of the modal value, 0..=5. An annotation only — roundness was
    /// measured NOT to separate true claims from false ones in this corpus, so
    /// it never gates a finding.
    pub modal_roundness: u8,
}

/// R1's half of the census. Suppression counters included, because a guard that
/// can hide a finding must say how often it did.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CohortCensus {
    /// Lines carrying at least one framed numeral.
    pub candidate_lines: usize,
    /// Of those, lines that produced a usable template.
    pub templated_lines: usize,
    /// Cohorts with at least two sites.
    pub cohorts_min2: usize,
    /// Cohorts reaching `min_sites` distinct files.
    pub cohorts_at_min_sites: usize,
    /// Sites G1 suppressed by filename convention.
    pub suppressed_generated_name: usize,
    /// Sites G1 suppressed by a DO-NOT-EDIT marker.
    pub suppressed_generated_marker: usize,
    /// Slots G2 suppressed as co-varying data.
    pub suppressed_multi_varying_slot: usize,
}

/// Everything R1 produces over one corpus.
#[derive(Debug, Clone, Default)]
pub struct R1Outcome {
    /// R1 findings, in report order.
    pub findings: Vec<Finding>,
    /// R1's half of the shared census. `files_scanned` is set to the number of
    /// files R1 was handed — the same number R2 reports — so a merge must
    /// overwrite that field rather than add it.
    pub census: Census,
    /// Precision warnings for a lowered floor or a disabled guard.
    pub warnings: Vec<String>,
}

/// What a blanked numeral looks like in a template.
const SLOT: &str = "\u{2588}";

static WORDS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"[A-Za-z]{3,}").expect("WORDS is a compile-time constant pattern")
});

/// A template needs this many real words before it is a sentence rather than a
/// row of punctuation and digits.
const MIN_TEMPLATE_WORDS: usize = 3;

/// Replace every numeral in a line with [`SLOT`].
pub fn blank_numerals(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut last = 0usize;
    for sp in scan_numerals(line) {
        out.push_str(&line[last..sp.start]);
        out.push_str(SLOT);
        last = sp.end;
    }
    out.push_str(&line[last..]);
    out
}

/// Is this template a sentence with at least one quantity in it?
fn usable_template(template: &str) -> bool {
    template.contains(SLOT) && WORDS.find_iter(template).count() >= MIN_TEMPLATE_WORDS
}

/// Candidate lines from one file, and how many framed numerals produced them.
///
/// A line appears at most once however many of its numerals were framed: the
/// line is the cohort unit.
pub fn claim_lines(path: &str, text: &str) -> (Vec<ClaimLine>, usize) {
    let framed = frame_file(path, text);
    let generated = classify_generated(path, text);
    let mut seen = HashSet::new();
    let lines = framed
        .iter()
        .filter(|n| seen.insert(n.line))
        .map(|n| ClaimLine {
            file: n.file.clone(),
            line: n.line,
            context: n.context.clone(),
            generated,
        })
        .collect();
    (lines, framed.len())
}

/// Roundness, 0..=5. Attached to a finding, never used to gate one.
fn roundness(v: f64) -> u8 {
    if v.fract() != 0.0 || v == 0.0 {
        return 0;
    }
    let n = v.abs() as u64;
    let mut r = 0u8;
    for (base, pts) in [
        (1_000_000u64, 5u8),
        (100_000, 5),
        (10_000, 4),
        (1_000, 4),
        (100, 3),
        (50, 2),
        (25, 2),
        (10, 2),
        (5, 1),
    ] {
        if n.is_multiple_of(base) {
            r = r.max(pts);
        }
    }
    if n >= 8 && n & (n - 1) == 0 {
        r = r.max(3);
    }
    r
}

/// The R1 rule: template identity, per-slot comparison, both guards, then the
/// singleton-shaped distribution gate.
/// Guard G1, and the census row it owes.
///
/// Split out of `find_replicated_divergence` because the two questions are
/// separate — WHETHER a line is machine-generated, and WHAT the cohort rule does
/// with the ones that survive — and folding them together put a three-arm match
/// on the same cognitive budget as the cohort walk. `pmat verify` measured the
/// combined function at 28 against a limit of 25.
///
/// The suppression is COUNTED, not silent: a guard that drops lines without
/// saying how many is indistinguishable from a guard that never fires, which is
/// the defect this whole check exists to report.
fn suppressed_as_generated(
    line: &ClaimLine,
    cfg: &CohortConfig,
    census: &mut CohortCensus,
) -> bool {
    if !cfg.guards.generated {
        return false;
    }
    match line.generated {
        Some(Generated::ByName) => {
            census.suppressed_generated_name += 1;
            true
        }
        Some(Generated::ByMarker) => {
            census.suppressed_generated_marker += 1;
            true
        }
        None => false,
    }
}

pub fn find_replicated_divergence(
    lines: &[ClaimLine],
    cfg: &CohortConfig,
) -> (Vec<ReplicatedDivergence>, CohortCensus) {
    let mut census = CohortCensus::default();
    let mut cohorts: BTreeMap<String, Vec<&ClaimLine>> = BTreeMap::new();

    let mut ordered: Vec<&ClaimLine> = lines.iter().collect();
    ordered.sort_by(|a, b| a.file.cmp(&b.file).then(a.line.cmp(&b.line)));
    ordered.dedup_by(|a, b| a.file == b.file && a.line == b.line);
    census.candidate_lines = ordered.len();

    for line in ordered {
        if suppressed_as_generated(line, cfg, &mut census) {
            continue;
        }
        let template = blank_numerals(&line.context);
        if !usable_template(&template) {
            continue;
        }
        census.templated_lines += 1;
        cohorts.entry(template).or_default().push(line);
    }

    let mut findings = Vec::new();
    for (template, sites) in cohorts {
        let files: BTreeSet<&str> = sites.iter().map(|s| s.file.as_str()).collect();
        if files.len() >= 2 {
            census.cohorts_min2 += 1;
        }
        if files.len() < cfg.min_sites {
            continue;
        }
        census.cohorts_at_min_sites += 1;
        findings.extend(divergent_slots(
            &template,
            &sites,
            files.len(),
            cfg,
            &mut census,
        ));
    }

    findings.sort_by(|a, b| {
        b.n_sites
            .cmp(&a.n_sites)
            .then(b.wrong_floor.cmp(&a.wrong_floor))
            .then(a.template.cmp(&b.template))
            .then(a.slot.cmp(&b.slot))
    });
    (findings, census)
}

/// Every slot of one cohort that disagrees in a singleton-shaped way.
/// The modal value of one slot, if it clears the share gate.
///
/// A tie is broken by the greatest token — `max_by_key` returns the LAST
/// maximum and the map iterates in ascending key order — so the report cannot
/// churn between runs. That branch is unreachable for anything emitted: two
/// values tied at `c` each would need `c/n >= 0.60` and `2c <= n` at once, and
/// `1.2n <= 2c <= n` has no solution. Pinned by
/// `a_modal_tie_can_never_pass_the_share_gate`.
fn modal_value(
    counts: &BTreeMap<&str, usize>,
    n: usize,
    cfg: &CohortConfig,
) -> Option<(String, usize)> {
    let (value, count) = counts
        .iter()
        .max_by_key(|(_, c)| **c)
        .map(|(v, c)| ((*v).to_string(), *c))?;
    let share = (count as f64) / (n as f64);
    (share >= cfg.min_modal_share).then_some((value, count))
}

/// Guard G2: a template whose OTHER numeric slots also vary is co-varying
/// machine-generated data, not a replicated claim.
///
/// Split out for the same reason G1 was: `pmat verify` measured the combined
/// `divergent_slots` at cognitive 26 against a limit of 25. The ablation this
/// guard came from is recorded in the spec — it independently removes both of
/// the cohort rule's false positives and touches neither true positive.
fn suppressed_multi_varying(
    slot: usize,
    width: usize,
    distinct: &[usize],
    cfg: &CohortConfig,
    census: &mut CohortCensus,
) -> bool {
    if !cfg.guards.single_varying_slot {
        return false;
    }
    let others_vary = (0..width).any(|o| o != slot && distinct[o] > 1);
    if others_vary {
        census.suppressed_multi_varying_slot += 1;
    }
    others_vary
}

fn divergent_slots(
    template: &str,
    sites: &[&ClaimLine],
    n_files: usize,
    cfg: &CohortConfig,
    census: &mut CohortCensus,
) -> Vec<ReplicatedDivergence> {
    let slots: Vec<Vec<String>> = sites.iter().map(|s| numeral_tokens(&s.context)).collect();
    let Some(width) = slots.iter().map(Vec::len).min() else {
        return Vec::new();
    };
    let distinct: Vec<usize> = (0..width)
        .map(|j| {
            slots
                .iter()
                .map(|s| s[j].as_str())
                .collect::<BTreeSet<_>>()
                .len()
        })
        .collect();

    let mut out = Vec::new();
    for j in 0..width {
        let n = sites.len();
        let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
        for s in &slots {
            *counts.entry(s[j].as_str()).or_default() += 1;
        }
        if counts.len() < 2 || counts.len() > cfg.max_distinct {
            continue;
        }
        // A tie is broken by the greatest token — `max_by_key` returns the last
        // maximum and the map iterates in ascending key order — so the report
        // cannot churn between runs. That branch is unreachable for anything
        // emitted: two values tied at c each would need c/n >= 0.60 and
        // 2c <= n at once, and 1.2n <= 2c <= n has no solution. Pinned by
        // `a_modal_tie_can_never_pass_the_share_gate`.
        let Some((modal_value, modal_count)) = modal_value(&counts, n, cfg) else {
            continue;
        };
        if suppressed_multi_varying(j, width, &distinct, cfg, census) {
            continue;
        }
        out.push(ReplicatedDivergence {
            template: template.to_string(),
            slot: j,
            n_sites: n,
            n_files,
            wrong_floor: n - modal_count,
            values: value_groups(sites, &slots, j),
            anchored: false,
            modal_roundness: super::frame::parse_numeral(&modal_value)
                .map(roundness)
                .unwrap_or(0),
        });
    }
    out
}

/// The distinct values of one slot, most common first, each with its sites.
fn value_groups(sites: &[&ClaimLine], slots: &[Vec<String>], j: usize) -> Vec<ValueGroup> {
    let mut by_value: BTreeMap<&str, Vec<Site>> = BTreeMap::new();
    for (site, tokens) in sites.iter().zip(slots) {
        by_value.entry(tokens[j].as_str()).or_default().push(Site {
            file: site.file.clone(),
            line: site.line,
            value: super::frame::parse_numeral(&tokens[j]).unwrap_or(f64::NAN),
            text: site.context.clone(),
        });
    }
    let mut groups: Vec<ValueGroup> = by_value
        .into_iter()
        .map(|(value, mut sites)| {
            sites.sort_by(|a, b| a.file.cmp(&b.file).then(a.line.cmp(&b.line)));
            ValueGroup {
                value: value.to_string(),
                count: sites.len(),
                sites,
            }
        })
        .collect();
    groups.sort_by(|a, b| b.count.cmp(&a.count).then(a.value.cmp(&b.value)));
    groups
}

/// Precision warning for a `--min-sites` below the default.
///
/// The check is quiet by design, and a quiet check gets its threshold lowered
/// by the next maintainer. This is what stops that from silently costing 90% of
/// the precision.
pub fn min_sites_warning(min_sites: usize) -> Option<String> {
    (min_sites < DEFAULT_MIN_SITES).then(|| {
        format!(
            "WARNING: --min-sites {min_sites} is below the default {DEFAULT_MIN_SITES}; \
             --min-sites 3 measured {LOWERED_PRECISION} precision on the reference corpus"
        )
    })
}

/// Precision warning for a disabled guard.
pub fn guards_warning(guards: &Guards) -> Option<String> {
    let mut off = Vec::new();
    if !guards.generated {
        off.push("G1 generated-file");
    }
    if !guards.single_varying_slot {
        off.push("G2 single-varying-slot");
    }
    (!off.is_empty()).then(|| {
        format!(
            "WARNING: guard(s) disabled ({}); with either guard off the reference \
             corpus measured 1/3 precision",
            off.join(", ")
        )
    })
}

/// Render an R1 finding in the shared shape.
///
/// `wrong_floor` travels as a floor and `anchored` as `false`; the prose says
/// *disagree*, never *wrong value*, because the rule cannot know which site is
/// the wrong one.
pub fn to_finding(d: &ReplicatedDivergence) -> Finding {
    let spread = d
        .values
        .iter()
        .map(|g| format!("{} x{}", g.value, g.count))
        .collect::<Vec<_>>()
        .join(", ");
    Finding {
        rule: RuleId::R1,
        quantity: d.template.clone(),
        sites: d.values.iter().flat_map(|g| g.sites.clone()).collect(),
        wrong_floor: d.wrong_floor,
        anchored: false,
        detail: format!(
            "{} sites in {} files state this quantity and disagree ({spread}); \
             at least {} of them do not match the tree",
            d.n_sites, d.n_files, d.wrong_floor
        ),
        evidence: d.template.clone(),
        fix: format!(
            "declare this quantity once — `.pmat-ratchet.toml [metric.<name>]` with the \
             command that reproduces it — then reconcile the {} copies against it",
            d.n_sites
        ),
    }
}

/// Run R1 over an in-memory corpus.
///
/// The caller owns file discovery; R1 owns what the bytes say. That split is
/// what lets every test in this module run from string literals.
pub fn run_r1(files: &[super::CorpusFile], cfg: &CohortConfig) -> R1Outcome {
    let mut lines = Vec::new();
    let mut framed = 0usize;
    for f in files {
        let (l, n) = claim_lines(&f.path, &f.text);
        lines.extend(l);
        framed += n;
    }
    let (divergences, cc) = find_replicated_divergence(&lines, cfg);

    let census = Census {
        files_scanned: files.len(),
        r1_framed_numerals: framed,
        r1_cohorts_min2: cc.cohorts_min2,
        r1_cohorts_at_min_sites: cc.cohorts_at_min_sites,
        suppressed_generated: cc.suppressed_generated_name + cc.suppressed_generated_marker,
        suppressed_multi_slot: cc.suppressed_multi_varying_slot,
        ..Census::default()
    };
    R1Outcome {
        findings: divergences.iter().map(to_finding).collect(),
        census,
        warnings: [
            min_sites_warning(cfg.min_sites),
            guards_warning(&cfg.guards),
        ]
        .into_iter()
        .flatten()
        .collect(),
    }
}

/// Read a repository and run R1 over it.
///
/// The one impure function in this module. An unreadable or non-git tree is an
/// error, never an empty result: "we could not measure it" must not reach the
/// same verdict as "there is nothing to report".
pub fn scan_repo(root: &Path, cfg: &CohortConfig) -> Result<R1Outcome, CorpusError> {
    let (files, corpus_census) = collect(root, R1_PATHSPECS)?;
    let mut outcome = run_r1(&files, cfg);
    outcome.census.files_tracked = corpus_census.tracked;
    Ok(outcome)
}
