//! R2 stage 3 — the contradiction rules C1..C5.
//!
//! These rules cannot be wrong about the world, because they never make a claim
//! about it. They only ever assert that the tree contradicts *itself*: a
//! ceiling whose own comment reports a breach, a value restated in units that
//! do not match it, a `const` that says it mirrors a key it does not mirror.
//! There is no oracle, no execution and no network — only consistency over
//! bytes that are already checked in.
//!
//! That is why the rule survived research at 2/2 precision with zero flapping
//! across 400 first-parent commits, and why the one finding that vanished
//! vanished on exactly the commit that fixed it.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::LazyLock;

use regex::Regex;

use super::annotate;
use super::extract::{self, Dim, Mention, Polarity};
use super::{Census, CorpusFile, Finding, R2Outcome, RuleId, Site};

/// A per-site value must differ from the sibling mode by at least this factor.
///
/// A policy tweak differs by a *margin* (90 against 80); a units or semantics
/// error differs by a *factor*. Measured on the audit corpus: every legitimate
/// per-crate override was within 3.3x, and the one real defect was 47.5x.
const DIVERGENCE_FACTOR: f64 = 4.0;

fn rx(pattern: &str) -> Regex {
    Regex::new(pattern).expect("static regex must compile")
}

static SCHEMA_FILES: LazyLock<Regex> = LazyLock::new(|| {
    rx(concat!(
        r"(^|/)(\.pmat-metrics\.toml|\.pmat-ratchet\.toml|pmat\.toml|clippy\.toml|",
        r"rustfmt\.toml|deny\.toml|tarpaulin\.toml|codecov\.ya?ml|\.pmat\.toml)$"
    ))
});

fn site_of(m: &Mention) -> Site {
    Site {
        file: m.file.clone(),
        line: m.line,
        value: m.value,
        text: m.text.clone(),
    }
}

fn basename(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

fn low(canon: &[f64]) -> f64 {
    canon.first().copied().unwrap_or(f64::NAN)
}

/// The unit an observed number inherits when it wrote none.
///
/// A dimensionless count inherits nothing: `Current: 570` beside
/// `max_unwrap_calls` is 570 things, and inventing milliseconds for it would
/// make every count comparison a unit conversion.
fn observed_unit(m: &Mention, written: &str) -> String {
    if m.dim == Dim::Count {
        return String::new();
    }
    if written.is_empty() {
        return m.dim.canonical_unit().to_string();
    }
    written.to_string()
}

/// How much of the ambiguity set actually breaches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Breach {
    /// Every reading breaches. Only this fires.
    All,
    /// Some reading breaches and some does not — the ambiguity acquits.
    Partial,
    /// No reading breaches.
    None,
}

fn breach(pol: Polarity, observed: &[f64], limit: &[f64]) -> Breach {
    if observed.is_empty() || limit.is_empty() {
        return Breach::None;
    }
    let mut breaching = 0usize;
    let mut total = 0usize;
    for o in observed {
        for l in limit {
            total += 1;
            let over = match pol {
                Polarity::Max => *o > l * (1.0 + extract::TOL),
                Polarity::Min => *o < l * (1.0 - extract::TOL),
            };
            if over {
                breaching += 1;
            }
        }
    }
    match breaching {
        0 => Breach::None,
        n if n == total => Breach::All,
        _ => Breach::Partial,
    }
}

// ---------------------------------------------------------------- C1

/// C1 SELF-BREACH — a limit whose own trailing comment reports a value that
/// violates it, under every unit reading.
///
/// The brief's headline defect: `max_unwrap_calls = 100  # Current: 570`. The
/// file declares a ceiling and then, on the same line, records that the tree is
/// 5.7x over it. Nothing needs to be measured to know that one of those two
/// numbers is not true.
fn c1_self_breach(ms: &[Mention], census: &mut Census) -> Vec<Finding> {
    let mut out = Vec::new();
    for m in ms {
        let Some(pol) = m.polarity else { continue };
        if !annotate::assertive(&m.annot) {
            continue;
        }
        for obs in annotate::observations(&m.annot) {
            let unit = observed_unit(m, &obs.unit);
            let observed = extract::to_canon(obs.value, &unit, m.dim);
            match breach(pol, &observed, &m.canon) {
                Breach::All => out.push(c1_finding(m, pol, &obs)),
                Breach::Partial => census.suppressed_unit_ambiguity += 1,
                Breach::None => {}
            }
        }
    }
    out
}

fn c1_finding(m: &Mention, pol: Polarity, obs: &annotate::Observation) -> Finding {
    let word = match pol {
        Polarity::Max => "ceiling",
        Polarity::Min => "floor",
    };
    Finding {
        rule: RuleId::C1,
        quantity: m.key.clone(),
        sites: vec![site_of(m)],
        wrong_floor: 1,
        anchored: false,
        detail: format!(
            "{} declares a {word} of {} and reports {} on the same line",
            m.key, m.value, obs.value
        ),
        evidence: obs.text.clone(),
        fix: format!(
            "either {} is not the limit, or {} is not the observation — \
             declare the quantity once in .pmat-ratchet.toml with the command \
             that reproduces it",
            m.value, obs.value
        ),
    }
}

// ---------------------------------------------------------------- C2

/// C2 RESTATEMENT MISMATCH — the annotation restates the value in
/// same-dimension units and the two disagree by more than 2%.
///
/// Two guards stand in front of it, and both are counted in the census. The
/// observation clauses are stripped first, so a healthy `50 MB (current: 42 MB)`
/// is not read as claiming the limit is 42 MB. Then the derivation guard: an
/// annotation that *computes* the value out of parts is arithmetic, not
/// contradiction.
fn c2_restatement(ms: &[Mention], census: &mut Census) -> Vec<Finding> {
    let mut out = Vec::new();
    for m in ms {
        if !annotate::assertive(&m.annot) {
            continue;
        }
        let head =
            annotate::strip_compound_durations(&annotate::strip_observation_clauses(&m.annot));
        let (mismatches, acquitted) = c2_mismatches(m, &head);
        if acquitted {
            census.suppressed_unit_ambiguity += 1;
        }
        if mismatches.is_empty() {
            continue;
        }
        if annotate::is_derivation(&head, &m.canon) {
            census.suppressed_derivation += 1;
            continue;
        }
        out.extend(mismatches.iter().map(|r| c2_finding(m, r)));
    }
    out
}

fn c2_mismatches(m: &Mention, head: &str) -> (Vec<annotate::Restatement>, bool) {
    let mut mismatches = Vec::new();
    let mut acquitted = false;
    for r in annotate::restatements(head) {
        let Some(d) = extract::dim_of_unit(&r.unit) else {
            continue;
        };
        if d != m.dim {
            continue;
        }
        let restated = extract::to_canon(r.value, &r.unit, d);
        if extract::all_close(&restated, &m.canon) {
            continue;
        }
        if extract::any_close(&restated, &m.canon) {
            acquitted = true;
            continue;
        }
        mismatches.push(r);
    }
    (mismatches, acquitted)
}

fn c2_finding(m: &Mention, r: &annotate::Restatement) -> Finding {
    Finding {
        rule: RuleId::C2,
        quantity: m.key.clone(),
        sites: vec![site_of(m)],
        wrong_floor: 1,
        anchored: false,
        detail: format!(
            "{} = {} but its own comment restates it as {}",
            m.key, m.value, r.text
        ),
        evidence: r.text.clone(),
        fix: "the declaration and its comment cannot both be right — correct one".to_string(),
    }
}

// ---------------------------------------------------------------- C3

/// C3 ARITHMETIC — a stated headroom percentage the numbers do not support.
///
/// `binary_max_bytes = 50_000_000  # 50 MB (current: 42 MB, 16% headroom)` is
/// arithmetically sound. When the same shape is copied forward and the
/// observation is updated but the percentage is not, it stops being sound, and
/// the line silently misreports how close the budget is to breaking.
fn c3_arithmetic(ms: &[Mention]) -> Vec<Finding> {
    let mut out = Vec::new();
    for m in ms {
        if m.annot.trim().is_empty() {
            continue;
        }
        let Some((claimed, text)) = annotate::headroom(&m.annot) else {
            continue;
        };
        let Some(obs) = annotate::observations(&m.annot).into_iter().next() else {
            continue;
        };
        let unit = observed_unit(m, &obs.unit);
        let current = extract::to_canon(obs.value, &unit, m.dim);
        if headroom_supported(claimed, &m.canon, &current) {
            continue;
        }
        out.push(c3_finding(m, claimed, &m.canon, &current, &text));
    }
    out
}

/// Both readings of "headroom" are accepted — `(L−C)/L` and `(L−C)/C` — because
/// repositories use both and the rule must not adjudicate English.
fn headroom_supported(claimed: f64, limit: &[f64], current: &[f64]) -> bool {
    let tolerance = 1.0f64.max(0.15 * claimed);
    for l in limit {
        for c in current {
            if *l <= 0.0 {
                continue;
            }
            let of_limit = (l - c) / l * 100.0;
            let of_current = if *c != 0.0 { (l - c) / c * 100.0 } else { 1e9 };
            if (of_limit - claimed).abs() <= tolerance || (of_current - claimed).abs() <= tolerance
            {
                return true;
            }
        }
    }
    false
}

fn c3_finding(m: &Mention, claimed: f64, limit: &[f64], current: &[f64], text: &str) -> Finding {
    let l = low(limit);
    let c = low(current);
    let actual = if l > 0.0 {
        (l - c) / l * 100.0
    } else {
        f64::NAN
    };
    Finding {
        rule: RuleId::C3,
        quantity: m.key.clone(),
        sites: vec![site_of(m)],
        wrong_floor: 1,
        anchored: false,
        detail: format!(
            "{} claims {claimed}% headroom; its own limit and observation give {actual:.1}%",
            m.key
        ),
        evidence: text.to_string(),
        fix: "recompute the percentage, or correct the observation it is derived from".to_string(),
    }
}

// ---------------------------------------------------------------- C4

/// C4 UNJUSTIFIED DIVERGENCE — one key of one schema, held by three or more
/// sibling files, where a site differs from the mode by a factor and offers no
/// reason.
///
/// The audit's real instance: a repository's root `codecov.yml` carried
/// `threshold: 95%` where both siblings said `2%`, one line under a comment
/// about requiring 95% *coverage*. In codecov's schema `threshold` is the
/// allowed **drop**, so 95 there does not tighten the gate — it disables it.
fn c4_divergence(ms: &[Mention]) -> Vec<Finding> {
    let mut domains: BTreeMap<(String, String), Vec<usize>> = BTreeMap::new();
    for (i, m) in ms.iter().enumerate() {
        if SCHEMA_FILES.is_match(&m.file) {
            domains
                .entry((basename(&m.file).to_string(), m.key.clone()))
                .or_default()
                .push(i);
        }
    }
    let mut out = Vec::new();
    for ((bn, key), idxs) in &domains {
        out.extend(c4_domain(ms, bn, key, idxs));
        out.extend(c4b_authority(ms, bn, key, idxs));
    }
    out
}

/// The modal value of a domain, with its share, or `None` when no convention
/// has formed. A "mode" over two sites is arbitrary: either could be called the
/// outlier, so the rule requires three sites and a modal value held by two.
fn domain_mode(ms: &[Mention], idxs: &[usize]) -> Option<(f64, usize, usize)> {
    let mut counts: BTreeMap<u64, (f64, usize)> = BTreeMap::new();
    for i in idxs {
        let v = low(&ms[*i].canon);
        let e = counts.entry(v.to_bits()).or_insert((v, 0));
        e.1 += 1;
    }
    if counts.len() < 2 {
        return None;
    }
    let total: usize = counts.values().map(|(_, c)| *c).sum();
    let (mode, count) = counts
        .values()
        .max_by(|a, b| a.1.cmp(&b.1).then(b.0.total_cmp(&a.0)))
        .copied()?;
    if total < 3 || count < 2 || (count as f64) / (total as f64) < 0.5 {
        return None;
    }
    Some((mode, count, total))
}

fn c4_domain(ms: &[Mention], bn: &str, key: &str, idxs: &[usize]) -> Vec<Finding> {
    let files: BTreeSet<&str> = idxs.iter().map(|i| ms[*i].file.as_str()).collect();
    if files.len() < 2 {
        return Vec::new();
    }
    let Some((mode, count, total)) = domain_mode(ms, idxs) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for i in idxs {
        let m = &ms[*i];
        let v = low(&m.canon);
        let (lo, hi) = if v < mode { (v, mode) } else { (mode, v) };
        let ratio = if lo > 0.0 { hi / lo } else { f64::INFINITY };
        if v == mode || ratio < DIVERGENCE_FACTOR || annotate::has_rationale(&m.annot, key) {
            continue;
        }
        out.push(Finding {
            rule: RuleId::C4,
            quantity: key.to_string(),
            sites: vec![site_of(m)],
            wrong_floor: 1,
            anchored: false,
            detail: format!(
                "{key} = {v} here, {mode} in {count} of {total} sibling {bn} files \
                 ({ratio:.1}x), with no stated reason"
            ),
            evidence: m.annot.clone(),
            fix: format!("state why this {bn} differs, or bring it back to {mode}"),
        });
    }
    out
}

/// C4b AUTHORITY CLASH — two sites cite the *same* named authority for
/// *different* values. Whatever the authority says, it cannot say both.
fn c4b_authority(ms: &[Mention], bn: &str, key: &str, idxs: &[usize]) -> Vec<Finding> {
    let mut cited: BTreeMap<String, BTreeMap<u64, usize>> = BTreeMap::new();
    for i in idxs {
        let m = &ms[*i];
        for a in annotate::authorities(&m.annot) {
            cited
                .entry(a)
                .or_default()
                .insert(low(&m.canon).to_bits(), *i);
        }
    }
    cited
        .iter()
        .filter(|(_, sites)| sites.len() > 1)
        .map(|(authority, sites)| Finding {
            rule: RuleId::C4,
            quantity: key.to_string(),
            sites: sites.values().map(|i| site_of(&ms[*i])).collect(),
            wrong_floor: 1,
            anchored: false,
            detail: format!(
                "{} sites of {bn} cite {authority:?} for {} different values of {key}",
                sites.len(),
                sites.len()
            ),
            evidence: authority.clone(),
            fix: format!("re-read {authority} and correct the sites that misquote it"),
        })
        .collect()
}

// ---------------------------------------------------------------- C5

/// C5 NAMED CROSS-REFERENCE — an annotation asserts equality with a named
/// declaration elsewhere, and the two values differ.
///
/// This is the rule that finds the specimen the whole check was built around:
/// `src/tests/binary_size.rs:40` declares `50 * 1024 * 1024` and says in the
/// same comment that it is aligned with `.pmat-metrics.toml`'s
/// `binary_max_bytes`, which is 50,000,000. The gap is 2,428,800 bytes under
/// every unit reading.
///
/// It fires only when the name resolves to **exactly one** declaration. An
/// ambiguous name identifies no quantity, so resolving it would invent a
/// contradiction between two unrelated keys — every unresolved reference is
/// counted in the census rather than guessed at.
fn c5_named_xref(ms: &[Mention], census: &mut Census) -> Vec<Finding> {
    let index = build_index(ms);
    let mut out = Vec::new();
    for (i, m) in ms.iter().enumerate() {
        let text = format!("{} {}", m.annot, m.block);
        if text.trim().is_empty() {
            continue;
        }
        for xref in annotate::xrefs(&text) {
            match resolve(ms, &index, i, &xref) {
                Resolved::One(t) => {
                    if ms[t].dim == m.dim && !extract::any_close(&m.canon, &ms[t].canon) {
                        out.push(c5_finding(m, &ms[t], &xref));
                    }
                }
                Resolved::Ambiguous => census.suppressed_unresolved_xref += 1,
                Resolved::Ignored => {}
            }
        }
    }
    out
}

fn build_index(ms: &[Mention]) -> BTreeMap<String, Vec<usize>> {
    let mut index: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (i, m) in ms.iter().enumerate() {
        index.entry(m.key.clone()).or_default().push(i);
        if let Some(leaf) = m.key.rsplit('.').next() {
            if leaf != m.key {
                index.entry(leaf.to_string()).or_default().push(i);
            }
        }
    }
    index
}

enum Resolved {
    One(usize),
    Ambiguous,
    Ignored,
}

fn resolve(
    ms: &[Mention],
    index: &BTreeMap<String, Vec<usize>>,
    from: usize,
    xref: &annotate::Xref,
) -> Resolved {
    let Some(key) = xref.key.as_deref() else {
        return Resolved::Ambiguous;
    };
    let Some(candidates) = index.get(key) else {
        return Resolved::Ambiguous;
    };
    // A single generic token — `threshold`, `limit` — names no quantity.
    if extract::norm_key(key).len() < 2 && xref.file.is_none() {
        return Resolved::Ignored;
    }
    let here = (&ms[from].file, ms[from].line);
    let matching: Vec<usize> = candidates
        .iter()
        .copied()
        .filter(|c| match xref.file.as_deref() {
            Some(f) => basename(f) == basename(&ms[*c].file),
            None => true,
        })
        .filter(|c| !(&ms[*c].file == here.0 && ms[*c].line == here.1))
        .collect();
    let distinct: BTreeSet<(&str, usize)> = matching
        .iter()
        .map(|c| (ms[*c].file.as_str(), ms[*c].line))
        .collect();
    match (distinct.len(), matching.first()) {
        (1, Some(first)) => Resolved::One(*first),
        _ => Resolved::Ambiguous,
    }
}

fn c5_finding(m: &Mention, t: &Mention, xref: &annotate::Xref) -> Finding {
    let delta = m.value - t.value;
    Finding {
        rule: RuleId::C5,
        quantity: t.key.clone(),
        sites: vec![site_of(m), site_of(t)],
        wrong_floor: 1,
        anchored: false,
        detail: format!(
            "{}:{} says {} = {} is the same quantity as {}:{} {} = {}; they differ by {}",
            m.file, m.line, m.key, m.value, t.file, t.line, t.key, t.value, delta
        ),
        evidence: xref.text.clone(),
        fix: format!(
            "make {} equal {}, or delete the claim that they are aligned",
            m.key, t.key
        ),
    }
}

// ---------------------------------------------------------------- driver

/// Run C1..C5 over an in-memory corpus.
///
/// Rules are applied in identifier order and their findings concatenated, so
/// output is deterministic for a deterministic corpus. Nothing here opens a
/// path: file discovery is the caller's business.
pub fn run(files: &[CorpusFile]) -> R2Outcome {
    let started = std::time::Instant::now();
    let mut census = Census::default();
    let mut mentions = Vec::new();
    for f in files {
        census.files_scanned += 1;
        census.raw_numeric_literals += extract::raw_literal_count(&f.text);
        mentions.extend(extract::extract_file(&f.path, &f.text));
    }
    census.r2_mentions = mentions.len();
    census.r2_assertive_annotations = mentions
        .iter()
        .filter(|m| annotate::assertive(&m.annot))
        .count();

    let mut findings = c1_self_breach(&mentions, &mut census);
    findings.extend(c2_restatement(&mentions, &mut census));
    findings.extend(c3_arithmetic(&mentions));
    findings.extend(c4_divergence(&mentions));
    findings.extend(c5_named_xref(&mentions, &mut census));

    census.elapsed_ms = started.elapsed().as_millis();
    R2Outcome { findings, census }
}
