/// CB-2113: Commit Traceability — every non-merge commit a pull request adds
/// carries a `Pmat-Ticket:` trailer naming a real, non-terminal roadmap item
/// (goal-mode.md §7, §8.4, §9). Engine: `src/services/commit_traceability/`.
///
/// Three verdict classes (§3.3), and the line between them is the point:
/// * Skip — not a git repository, or a roadmap that was never committed: a
///   structural absence, nothing this rule could have read.
/// * Fail `not_measured:` — an input that was expected and could not be read:
///   git failed, no base branch resolves, the roadmap does not parse, or the
///   roadmap was committed and is now gone (deleting a gate's input is not a
///   way of passing it).
/// * Pass `not_applicable:` — HEAD is the default branch: the commits since
///   the latest `v*` tag are counted and printed but not judged, because until
///   merges are restricted to merge commits a squash rewrites the trailers
///   this rule reads (§8.4). Never a silent pass: the reason and the count
///   are in the message.
///
/// Everything else is measured: the commits `merge-base(base, HEAD)..HEAD`,
/// each one either trailed with an id the roadmap holds in a non-terminal
/// status, or named in the failure with the reason.
pub(crate) fn check_commit_traceability(project_path: &Path) -> ComplianceCheck {
    use crate::models::comply_config::CheckSeverity;
    use crate::services::commit_traceability::{self as ct, Inputs, Range};

    let literal = "CB-2113: Commit Traceability";

    match ct::inputs(project_path) {
        Inputs::NotGit => ComplianceCheck {
            name: literal.to_string(),
            status: CheckStatus::Skip,
            severity: CheckSeverity::Info.into(),
            message: "not a git repository — no commits to trace".to_string(),
        },
        Inputs::NoRoadmap => ComplianceCheck {
            name: literal.to_string(),
            status: CheckStatus::Skip,
            severity: CheckSeverity::Info.into(),
            message: "no docs/roadmaps/roadmap.yaml — this project does not track work in a roadmap".to_string(),
        },
        Inputs::RoadmapDeleted => ComplianceCheck {
            name: literal.to_string(),
            status: CheckStatus::Fail,
            severity: CheckSeverity::Error.into(),
            message: "not_measured: docs/roadmaps/roadmap.yaml was committed and is now gone — deleting a gate's input is not a way of passing it (goal-mode.md doctrine 2)".to_string(),
        },
        Inputs::Ready => match ct::measure(project_path) {
            Err(e) => ComplianceCheck {
                name: literal.to_string(),
                status: CheckStatus::Fail,
                severity: CheckSeverity::Error.into(),
                message: format!("not_measured: {e} — an input the rule expected and could not read is a failure, not a pass (goal-mode.md doctrine 2)"),
            },
            Ok(m) => match m.range {
                Range::DefaultBranch { since } => ComplianceCheck {
                    name: literal.to_string(),
                    status: CheckStatus::Pass,
                    severity: CheckSeverity::Info.into(),
                    message: format!(
                        "not_applicable: HEAD is the default branch; {} of {} non-merge commit(s) since {} carry a Pmat-Ticket trailer — master is not judged until merges are restricted to merge commits (goal-mode.md §8.4)",
                        m.trailered,
                        m.commits,
                        since.unwrap_or_else(|| "the root commit".to_string())
                    ),
                },
                Range::PullRequest { ref base, ref merge_base } => {
                    let mb7 = if merge_base.len() > 7 { &merge_base[..7] } else { merge_base };
                    pr_check(literal, &m, mb7, base)
                }
            }
        }
    }
}

/// The verdict for a judged pull-request range.
fn pr_check(
    literal: &str,
    m: &crate::services::commit_traceability::Measurement,
    mb7: &str,
    base: &str,
) -> ComplianceCheck {
    use crate::models::comply_config::CheckSeverity;
    if m.commits == 0 {
        ComplianceCheck {
            name: literal.to_string(),
            status: CheckStatus::Pass,
            severity: CheckSeverity::Info.into(),
            message: format!("0 non-merge commit(s) in {}..HEAD (base {}): nothing to judge — measured, not skipped", mb7, base),
        }
    } else if m.findings.is_empty() {
        ComplianceCheck {
            name: literal.to_string(),
            status: CheckStatus::Pass,
            severity: CheckSeverity::Info.into(),
            message: pass_message(m, mb7, base),
        }
    } else {
        let joined = findings_text(&m.findings);
        ComplianceCheck {
            name: literal.to_string(),
            status: CheckStatus::Fail,
            severity: CheckSeverity::Error.into(),
            message: format!("{} of {} non-merge commit(s) in {}..HEAD (base {}) break traceability: {} — {}", m.findings.len(), m.commits, mb7, base, joined, traceability_remedy(&m.findings)),
        }
    }
}

/// The first eight findings, one clause each, `; `-joined.
fn findings_text(findings: &[crate::services::commit_traceability::Finding]) -> String {
    use crate::services::commit_traceability::Violation;
    let mut msgs = vec![];
    for f in findings.iter().take(8) {
        let hash7 = if f.hash.len() > 7 { &f.hash[..7] } else { &f.hash };
        msgs.push(match &f.violation {
            Violation::NoTrailer => format!("{} {}: no Pmat-Ticket trailer", hash7, f.subject),
            Violation::UnknownTicket(id) => format!("{} {}: Pmat-Ticket {} is not in docs/roadmaps/roadmap.yaml", hash7, f.subject, id),
            Violation::TerminalTicket { id, status } => format!("{} {}: Pmat-Ticket {} is {} — work belongs to an open item", hash7, f.subject, id, status),
        });
    }
    let mut joined = msgs.join("; ");
    if findings.len() > 8 {
        joined.push_str(&format!(" (+{} more)", findings.len() - 8));
    }
    joined
}

/// The Pass message for a judged range with no findings; names the bot
/// exemption count when there is one (GH-1356).
fn pass_message(m: &crate::services::commit_traceability::Measurement, mb7: &str, base: &str) -> String {
    if m.bot_exempt == 0 {
        return format!("all {} non-merge commit(s) in {}..HEAD (base {}) carry a Pmat-Ticket trailer naming an open roadmap item", m.commits, mb7, base);
    }
    format!("{} of {} non-merge commit(s) in {}..HEAD (base {}) carry a Pmat-Ticket trailer naming an open roadmap item; {} authored by a GitHub bot account (exempt)", m.trailered, m.commits, mb7, base, m.bot_exempt)
}

/// What to do about the violations present — one clause per kind, so a trailer
/// that is present is never told to add one (PMAT-727: #1252 read "add
/// `Pmat-Ticket: <id>` as a git trailer" about a trailer naming a completed item).
fn traceability_remedy(findings: &[crate::services::commit_traceability::Finding]) -> String {
    let mut clauses: Vec<&str> = Vec::new();
    if findings.iter().any(|f| {
        matches!(
            f.violation,
            crate::services::commit_traceability::Violation::NoTrailer
        )
    }) {
        clauses.push("add `Pmat-Ticket: <id>` to the message's LAST paragraph, beside Co-Authored-By: git reads trailers from the last paragraph only (git commit -m '<subject>' -m $'Pmat-Ticket: PMAT-NNN\\nCo-Authored-By: Name <email>')");
    }
    if findings.iter().any(|f| {
        matches!(
            f.violation,
            crate::services::commit_traceability::Violation::UnknownTicket(_)
        )
    }) {
        clauses.push("name an id that is in docs/roadmaps/roadmap.yaml");
    }
    if findings.iter().any(|f| {
        matches!(
            f.violation,
            crate::services::commit_traceability::Violation::TerminalTicket { .. }
        )
    }) {
        clauses.push("work belongs to an open item: leave a ticket open in its own PR, and let the next ticket's PR mark it completed");
    }
    clauses.join("; ")
}
