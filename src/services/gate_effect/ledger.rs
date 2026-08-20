//! The enforcement ledger: one row per CB rule, with a status and a
//! `file:line` citation.
//!
//! Written by CB-2100 rather than by hand. Three hundred hand-maintained rows
//! would be stale the week they were written, and a stale ledger is worse than
//! none — it is a confident wrong answer about what this repository enforces.
//!
//! A rule whose status cannot be established is written `UNREACHABLE`, never
//! left blank. "We could not tell" and "nothing gates it" are both findings,
//! and the ledger says which.

use super::roster::{self, Rule};
use super::GateEffectReport;
use crate::models::comply_config::{CheckSeverity, ComplyConfig};
use std::path::Path;

/// Where the generated ledger lives.
pub const LEDGER_PATH: &str = "docs/status/comply-enforcement-ledger.md";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// Reachable from a required context through an invocation whose failure
    /// can still fail the job.
    Enforced,
    /// An invocation exists in the closure, but nothing it reports can fail the
    /// required check.
    Neutered,
    /// No required context reaches any invocation of this rule.
    Unreachable,
}

impl Status {
    pub fn label(self) -> &'static str {
        match self {
            Status::Enforced => "ENFORCED",
            Status::Neutered => "NEUTERED",
            Status::Unreachable => "UNREACHABLE",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Row {
    pub rule: Rule,
    pub severity: String,
    pub status: Status,
    /// The invocation carrying the rule, or why nothing does.
    pub carrier: String,
}

/// Severity as *declared*, not as guessed. A rule the config does not mention
/// runs at whatever severity its handler hardcodes, which is not statically
/// knowable from here, so it is reported as `unconfigured` rather than
/// invented.
fn severity_of(rule: &Rule, config: &ComplyConfig) -> String {
    match config.checks.get(&rule.config_key()) {
        Some(c) if !c.enabled => "disabled".into(),
        Some(c) => match c.severity {
            CheckSeverity::Critical => "critical".into(),
            CheckSeverity::Error => "error".into(),
            CheckSeverity::Warning => "warning".into(),
            CheckSeverity::Info => "info".into(),
        },
        None => "unconfigured".into(),
    }
}

/// Build the ledger rows. `Err` when the roster is empty: an enforcement ledger
/// over no rules is the vacuous pass this whole rule exists to reject.
pub fn rows(
    project_path: &Path,
    report: &GateEffectReport,
    config: &ComplyConfig,
) -> Result<Vec<Row>, String> {
    let rules = roster::collect(project_path);
    if rules.is_empty() {
        return Err(format!(
            "the comply rule registry under {} could not be enumerated, or is empty — an \
             enforcement ledger over zero rules is vacuous, so this is a failure rather than a \
             clean sheet",
            roster::HANDLER_DIR
        ));
    }
    let (status, carrier) = repo_status(report);
    Ok(rules
        .into_iter()
        .map(|rule| {
            // A rule the registry vouches for but no source line names cannot
            // have its enforcement established at all. That is a finding in its
            // own right, so it is written UNREACHABLE rather than left blank.
            let (status, carrier) = if rule.has_citation() {
                (status, carrier.clone())
            } else {
                (
                    Status::Unreachable,
                    "registered, but no definition site could be found for it".to_string(),
                )
            };
            Row {
                severity: severity_of(&rule, config),
                rule,
                status,
                carrier,
            }
        })
        .collect())
}

/// Every CB rule in this repository is run by the same command, so they share a
/// status: whichever verdict `pmat comply check` itself gets. The moment an
/// invocation restricts the roster (`--checks`, `--only`, …) it stops standing
/// in for the whole set, which the engine already records as a suppression.
fn repo_status(report: &GateEffectReport) -> (Status, String) {
    if let Some(inv) = report.enforcing().next() {
        return (
            Status::Enforced,
            format!(
                "{}:{} step `{}` ({})",
                inv.workflow.display(),
                inv.job_id,
                inv.step,
                inv.via
            ),
        );
    }
    if let Some(inv) = report.neutered().next() {
        return (
            Status::Neutered,
            format!(
                "{}:{} step `{}` — {}",
                inv.workflow.display(),
                inv.job_id,
                inv.step,
                inv.suppressions.join("; ")
            ),
        );
    }
    (
        Status::Unreachable,
        "no required status check reaches any invocation of the rule roster".to_string(),
    )
}

/// Render the ledger.
///
/// Deterministic by construction: no timestamps, no version stamps, rows sorted
/// by rule id. A generated file that changes on every run cannot be diffed, and
/// a ledger nobody diffs is decoration.
#[provable_contracts_macros::contract(
    "comply-gate-effect-v1.yaml",
    equation = "enforcement_ledger"
)]
pub fn render(
    project_path: &Path,
    report: &GateEffectReport,
    config: &ComplyConfig,
) -> Result<String, String> {
    let rows = rows(project_path, report, config)?;
    let mut s = String::new();
    s.push_str("# Comply Enforcement Ledger\n\n");
    s.push_str(
        "GENERATED by CB-2100 (gate-effect verification). Do not hand-edit: regenerate with\n\
         `pmat comply ledger --write`. One row per CB rule declared under\n",
    );
    s.push_str(&format!("`{}`.\n\n", roster::HANDLER_DIR));
    s.push_str(
        "Drift is keyed on this document's DATA — rule id, title, severity, status, carrier, and\n\
         the FILE a rule is declared in. The line number in `Defined at`, and the `Source:`\n\
         provenance label below, are presentation rather than identity: an edit above a rule\n\
         declaration moves the first, and reading the same roots from a different place changes\n\
         the second, and neither is a change to what this repository enforces. The line number is\n\
         therefore advisory and may lag the source until the next `--write`.\n\n",
    );
    push_roots(&mut s, report);
    push_summary(&mut s, &rows);
    push_holes(&mut s, report);
    push_table(&mut s, &rows);
    Ok(s)
}

fn push_roots(s: &mut String, report: &GateEffectReport) {
    s.push_str("## Required status checks (the reachability roots)\n\n");
    s.push_str(&format!(
        "Source: {}\n\n",
        report.context_source.as_deref().unwrap_or("unresolved")
    ));
    if report.required_contexts.is_empty() {
        s.push_str("- (none resolved — every rule below is UNREACHABLE by construction)\n\n");
        return;
    }
    s.push_str("| Required context | Reaches a rule invocation |\n");
    s.push_str("|---|---|\n");
    for (context, effect) in report.context_effects() {
        s.push_str(&format!("| `{context}` | {} |\n", effect.explain()));
    }
    s.push('\n');
}

fn push_summary(s: &mut String, rows: &[Row]) {
    let count = |st: Status| rows.iter().filter(|r| r.status == st).count();
    s.push_str("## Summary\n\n");
    s.push_str(&format!("- rules: {}\n", rows.len()));
    s.push_str(&format!("- ENFORCED: {}\n", count(Status::Enforced)));
    s.push_str(&format!("- NEUTERED: {}\n", count(Status::Neutered)));
    s.push_str(&format!(
        "- UNREACHABLE: {}\n\n",
        count(Status::Unreachable)
    ));
}

fn push_holes(s: &mut String, report: &GateEffectReport) {
    if report.holes.is_empty() {
        return;
    }
    s.push_str("## Holes\n\n");
    s.push_str("Things that could not be measured. Each one is a failure, not a blank.\n\n");
    for h in &report.holes {
        s.push_str(&format!("- {h}\n"));
    }
    s.push('\n');
}

fn push_table(s: &mut String, rows: &[Row]) {
    s.push_str("## Rules\n\n");
    s.push_str("| Rule | Title | Severity | Status | Enforced by | Defined at |\n");
    s.push_str("|---|---|---|---|---|---|\n");
    for r in rows {
        s.push_str(&format!(
            "| {} | {} | {} | {} | {} | `{}` |\n",
            r.rule.id,
            title_cell(&r.rule),
            r.severity,
            r.status.label(),
            escape(&r.carrier),
            r.rule.citation()
        ));
    }
}

/// The Title cell for one rule — **never** empty.
///
/// A blank cell tells the reader nothing, and worse, it tells them nothing in a
/// way that looks deliberate. Six rows shipped blank, and "this rule has no
/// title", "the check never names itself" and "the scanner lost the
/// declaration" are three different findings that all render as the same empty
/// space. So a rule with no discoverable title says UNIDENTIFIED and says why.
pub fn title_cell(rule: &Rule) -> String {
    if rule.title.trim().is_empty() {
        return format!(
            "**UNIDENTIFIED** — registered as `{}`, but no `{}: <title>` declaration was found \
             in the handler sources, so nothing this check reports at runtime can be tied back \
             to this row",
            rule.config_key(),
            rule.id
        );
    }
    escape(&rule.title)
}

/// Markdown tables end a cell at `|`, and rule titles contain them.
fn escape(s: &str) -> String {
    s.replace('|', "\\|")
}

/// Read the committed ledger, if there is one.
pub fn committed(project_path: &Path) -> Option<String> {
    std::fs::read_to_string(project_path.join(LEDGER_PATH)).ok()
}

// ── identity ────────────────────────────────────────────────────────────────

/// What the identity substitutes for the provenance label. A fixed string, so
/// that a ledger which has *lost* the line still differs from one that has it.
const PROVENANCE_MASK: &str = "Source: (provenance — not part of the ledger identity)";

/// Prefix of the one rendered line that names where the root list came from.
const PROVENANCE_PREFIX: &str = "Source: ";

/// Has the committed ledger drifted from what the engine computes?
///
/// Drift is a difference in the ledger's [`identity`], not in its bytes.
pub fn drifted(committed: &str, expected: &str) -> bool {
    identity(committed) != identity(expected)
}

/// The ledger's **identity**: what it actually asserts, with the two
/// presentation-only fields masked out.
///
/// Drift used to be a byte compare of the rendering, which made CB-2100 go red
/// for two reasons that say nothing about what this repository enforces:
///
/// * **a line number moved.** Every citation is `file:line`, so a two-line
///   clippy restructure *above* a rule declaration was drift. A gate that
///   reddens on edits it does not measure is a gate people learn to regenerate
///   reflexively, and a reflex carries no signal — which is how a real drift
///   would eventually be regenerated away unread.
/// * **the provenance label changed.** Supplying the identical required
///   contexts through `PMAT_REQUIRED_STATUS_CHECKS` instead of the committed
///   manifest rewrote one `Source:` line and nothing else. The roots were the
///   same; only the label differed.
///
/// So identity keys on the data — rule id, title, severity, status, carrier,
/// and the **file** a rule is declared in. Never the line, never the label.
///
/// The line is still *printed*: a citation you cannot navigate to is not a
/// citation. It is simply not part of what the ledger claims, and it may
/// therefore lag the source until the next `pmat comply ledger --write`. That
/// is the trade, stated: an advisory line number that can be stale, in exchange
/// for a gate that only fires on enforcement changing.
pub fn identity(document: &str) -> String {
    document
        .lines()
        .map(|line| {
            if line.starts_with(PROVENANCE_PREFIX) {
                PROVENANCE_MASK.to_string()
            } else {
                mask_line_numbers(line)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// `` `src/x.rs:167` `` → `` `src/x.rs` ``.
///
/// Only inside backticks, and only when the suffix is entirely digits — the
/// carrier column spells an invocation site `workflow:job_id`, unbackticked and
/// with a job id that is not a number, and masking that would blind the ledger
/// to a rule changing which job carries it.
fn mask_line_numbers(line: &str) -> String {
    line.split('`')
        .enumerate()
        .map(|(i, part)| {
            if i % 2 == 1 {
                strip_line_suffix(part)
            } else {
                part.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("`")
}

fn strip_line_suffix(inner: &str) -> String {
    match inner.rsplit_once(':') {
        Some((head, tail)) if !tail.is_empty() && tail.chars().all(|c| c.is_ascii_digit()) => {
            head.to_string()
        }
        _ => inner.to_string(),
    }
}
