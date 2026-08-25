//! What CB-2104 prints.
//!
//! Two decisions here are not cosmetic.
//!
//! **The census leads and is unconditional.** This check is quiet by design —
//! it fires once on this repository — so the common output is "nothing found",
//! and "nothing found" has to arrive with its working shown. Putting the census
//! first means a reader cannot take the silence at face value without also
//! seeing how many files, numerals and named quantities produced it.
//!
//! **Every site is named `file:line`.** A finding that says a quantity
//! disagrees, without saying where, has told a developer to go and re-run the
//! tool. R1 findings can carry 45 sites; all 45 are printed.
//!
//! Nothing here says a value is *wrong*, or *fabricated*. The rules report
//! disagreement and contradiction; which of two disagreeing numbers is true is
//! a question only an anchor can answer, and CB-2104 has none.

use serde::Serialize;

use super::census::SelfTest;
use super::{Census, Finding, NumericClaimsReport, RuleId, Site, Status};

/// How much of a source line a site prints before it is cut.
const SITE_TEXT_CHARS: usize = 88;

/// `1234567` as `1,234,567`.
fn commas(n: usize) -> String {
    let digits = n.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// A share of the corpus, or `n/a` when there is no denominator.
fn pct(part: usize, whole: usize) -> String {
    if whole == 0 {
        return "n/a".to_string();
    }
    format!("{:.1}%", 100.0 * part as f64 / whole as f64)
}

/// A value as the site wrote it, with separators and without a spurious `.0`.
fn fmt_value(v: f64) -> String {
    if !v.is_finite() {
        return "?".to_string();
    }
    if v.fract() == 0.0 && v.abs() < 1e15 {
        let neg = v < 0.0;
        let body = commas(v.abs() as usize);
        return if neg { format!("-{body}") } else { body };
    }
    format!("{v}")
}

/// Cut on a character boundary; a template can carry a block glyph or an em dash.
fn clip(s: &str, chars: usize) -> String {
    let text = s.trim();
    if text.chars().count() <= chars {
        return text.to_string();
    }
    let head: String = text.chars().take(chars.saturating_sub(1)).collect();
    format!("{head}…")
}

fn plural(n: usize) -> &'static str {
    if n == 1 {
        "finding"
    } else {
        "findings"
    }
}

fn elapsed(ms: u128) -> String {
    if ms < 1000 {
        format!("{ms}ms")
    } else {
        format!("{:.1}s", ms as f64 / 1000.0)
    }
}

fn self_test_line(st: &SelfTest) -> String {
    format!(
        "{} ({}/{} planted defects recovered, {}/{} innocent numbers flagged)",
        if st.passed { "PASS" } else { "FAIL" },
        st.recovered,
        st.planted,
        st.false_positives.len(),
        st.innocent_items
    )
}

/// A finding count, coloured by whether it is news.
///
/// `warn` above zero and `pass` at zero: this check never blocks, so colour is
/// the only thing distinguishing "nothing to report" from "read this" at a
/// glance. Both spellings collapse to the bare number when colour is off, so
/// `--color never` and a pipe still produce plain, greppable text.
fn count_marker(n: usize) -> String {
    let text = n.to_string();
    if n == 0 {
        crate::cli::colors::pass(&text)
    } else {
        crate::cli::colors::warn(&text)
    }
}

fn census_block(out: &mut String, c: &Census, st: &SelfTest) {
    out.push_str("CENSUS  (this block is the proof the check ran)\n");
    out.push_str(&format!(
        "  files scanned        {} of {} tracked  (md/rs/toml/yaml/yml/json; R1 reads {} of them)\n",
        commas(c.files_scanned),
        commas(c.files_tracked),
        commas(c.r1_files_scanned)
    ));
    out.push_str(&format!(
        "  excluded             machine-managed {}   fixture-tree {}   changelog {}   \
         unreadable {}\n",
        commas(c.excluded_machine_managed),
        commas(c.excluded_fixture_tree),
        commas(c.excluded_changelog),
        commas(c.unreadable)
    ));
    out.push_str(&format!(
        "  R1 framed numerals   {}   cohorts n>=2 {}   at the floor {}\n",
        commas(c.r1_framed_numerals),
        commas(c.r1_cohorts_min2),
        commas(c.r1_cohorts_at_min_sites)
    ));
    out.push_str(&format!(
        "  R2 mentions          {}   assertive annotations {}\n",
        commas(c.r2_mentions),
        commas(c.r2_assertive_annotations)
    ));
    out.push_str(&format!(
        "  suppressed           G1 generated {}   G2 multi-slot {}   derivation {}\n",
        commas(c.suppressed_generated),
        commas(c.suppressed_multi_slot),
        commas(c.suppressed_derivation)
    ));
    out.push_str(&format!(
        "                       unit-ambiguity {}   unresolved-xref {}\n",
        commas(c.suppressed_unit_ambiguity),
        commas(c.suppressed_unresolved_xref)
    ));
    if c.suppressed_generated > 0 && c.suppressed_multi_slot == 0 {
        out.push_str(
            "                       G1 runs first, so a 0 beside G2 records that G1 got there \
             first, not that G2 is idle\n",
        );
    }
    out.push_str(&format!(
        "  raw numeric literals {} in the scanned files  ->  R1 reads {}, R2 {}\n",
        commas(c.raw_numeric_literals),
        pct(c.r1_framed_numerals, c.raw_numeric_literals),
        pct(c.r2_mentions, c.raw_numeric_literals)
    ));
    out.push_str(&format!("  self-test fixture    {}\n", self_test_line(st)));
    out.push_str(&format!(
        "  elapsed              {}\n",
        elapsed(c.elapsed_ms)
    ));
}

fn site_line(out: &mut String, s: &Site) {
    out.push_str(&format!(
        "      {}:{}  {}  {}\n",
        s.file,
        s.line,
        fmt_value(s.value),
        clip(&s.text, SITE_TEXT_CHARS)
    ));
}

fn finding_block(out: &mut String, f: &Finding) {
    out.push_str(&format!(
        "\n  [{}] {}  {}\n",
        f.rule.as_str(),
        f.rule.title(),
        clip(&f.quantity, 120)
    ));
    out.push_str(&format!("    {}\n", f.detail));
    // The floor, named as a floor. `detail` already says how many sites
    // disagree; repeating it here as "N of them are wrong" would be the one
    // claim no rule in this check can make.
    out.push_str(&format!(
        "    {} site{}; disagreement floor {}; anchored: {}\n",
        f.sites.len(),
        if f.sites.len() == 1 { "" } else { "s" },
        f.wrong_floor,
        if f.anchored { "yes" } else { "no" }
    ));
    for s in &f.sites {
        site_line(out, s);
    }
    if !f.evidence.trim().is_empty() {
        out.push_str(&format!("    evidence: {}\n", clip(&f.evidence, 160)));
    }
    out.push_str(&format!("    FIX: {}\n", f.fix));
}

fn findings_block(out: &mut String, findings: &[Finding]) {
    let (r1, r2): (Vec<&Finding>, Vec<&Finding>) =
        findings.iter().partition(|f| f.rule == RuleId::R1);
    out.push('\n');
    // Coloured through the shared helpers, so `--color` is honoured rather than
    // accepted and ignored. The flag-efficacy sweep booked `comply
    // numeric-claims --color` a NO-OP: byte-identical output with `always` and
    // `never`, because nothing here reached `crate::cli::colors`.
    //
    // A count of zero findings is not the same news as a count above zero, so
    // they take different helpers: `pass` when the rule found nothing, `warn`
    // when it did. This check never blocks, so the colour is the only signal
    // distinguishing the two at a glance.
    out.push_str(&format!(
        "R1  REPLICATED DIVERGENT CLAIM   {} {}\n",
        count_marker(r1.len()),
        plural(r1.len())
    ));
    out.push_str(&format!(
        "R2  CONTRADICTION                {} {}\n",
        count_marker(r2.len()),
        plural(r2.len())
    ));
    for f in r1.into_iter().chain(r2) {
        finding_block(out, f);
    }
}

/// The human-readable report.
pub fn text(report: &NumericClaimsReport) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "{}  numeric-claims  (WARN — advisory, never blocks)\n\n",
        report.check
    ));

    if report.status == Status::Unmeasurable {
        out.push_str("UNMEASURABLE  exit 2\n");
        for w in &report.warnings {
            out.push_str(&format!("  {w}\n"));
        }
        out.push_str(
            "  No result is printed for this run: \"we could not measure it\" must never read \
             as \"it did not regress\".\n\n",
        );
    }

    census_block(&mut out, &report.census, &report.self_test);

    if report.status == Status::Ok {
        if !report.warnings.is_empty() {
            out.push_str("\nWARNINGS\n");
            for w in &report.warnings {
                out.push_str(&format!("  {w}\n"));
            }
        }
        findings_block(&mut out, &report.findings);
    }
    out
}

/// The JSON shape, with the exit code stated rather than inferred.
///
/// Written out field by field rather than with `#[serde(flatten)]`: flatten
/// buffers through serde's private `Content` type, and the census carries a
/// `u128`.
#[derive(Serialize)]
struct JsonReport<'a> {
    check: &'a str,
    severity: &'a str,
    status: Status,
    exit: i32,
    findings: &'a [Finding],
    census: &'a Census,
    warnings: &'a [String],
    self_test: &'a SelfTest,
}

/// A run that did not happen because `.pmat.yaml` turned the rule off.
///
/// Its own document rather than a zeroed report: a census of zeros would read
/// as "measured, and clean", which is the one thing this check refuses to let
/// an absence look like.
#[derive(Serialize)]
struct DisabledReport<'a> {
    check: &'a str,
    severity: &'a str,
    status: &'a str,
    exit: i32,
    reason: String,
}

fn disabled(rule_id: &str) -> DisabledReport<'_> {
    DisabledReport {
        check: "CB-2104",
        severity: "warn",
        status: "DISABLED",
        exit: 0,
        reason: format!("checks.{rule_id}.enabled = false in .pmat.yaml; nothing was scanned"),
    }
}

/// One line for a human when the rule is switched off.
pub fn disabled_text(rule_id: &str) -> String {
    format!(
        "CB-2104  numeric-claims  DISABLED — {}\n",
        disabled(rule_id).reason
    )
}

/// The same, as a document a consumer can branch on. Never a zeroed census.
pub fn disabled_json(rule_id: &str) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&disabled(rule_id))
}

/// The machine-readable report.
pub fn json(report: &NumericClaimsReport) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&JsonReport {
        check: report.check,
        severity: report.severity,
        status: report.status,
        exit: report.exit_code(),
        findings: &report.findings,
        census: &report.census,
        warnings: &report.warnings,
        self_test: &report.self_test,
    })
}
