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
    use crate::services::commit_traceability::{self as ct, Inputs, Range};
    use crate::models::comply_config::CheckSeverity;

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
                Range::PullRequest { base, merge_base } => {
                    let mb7 = if merge_base.len() > 7 { &merge_base[..7] } else { &merge_base };
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
                            message: format!("all {} non-merge commit(s) in {}..HEAD (base {}) carry a Pmat-Ticket trailer naming an open roadmap item", m.commits, mb7, base),
                        }
                    } else {
                        let mut msgs = vec![];
                        for f in m.findings.iter().take(8) {
                            let hash7 = if f.hash.len() > 7 { &f.hash[..7] } else { &f.hash };
                            match &f.violation {
                                ct::Violation::NoTrailer => {
                                    msgs.push(format!("{} {}: no Pmat-Ticket trailer", hash7, f.subject));
                                },
                                ct::Violation::UnknownTicket(id) => {
                                    msgs.push(format!("{} {}: Pmat-Ticket {} is not in docs/roadmaps/roadmap.yaml", hash7, f.subject, id));
                                },
                                ct::Violation::TerminalTicket { id, status } => {
                                    msgs.push(format!("{} {}: Pmat-Ticket {} is {} — work belongs to an open item", hash7, f.subject, id, status));
                                },
                            }
                        }
                        let mut joined = msgs.join("; ");
                        if m.findings.len() > 8 {
                            joined.push_str(&format!(" (+{} more)", m.findings.len() - 8));
                        }
                        ComplianceCheck {
                            name: literal.to_string(),
                            status: CheckStatus::Fail,
                            severity: CheckSeverity::Error.into(),
                            message: format!("{} of {} non-merge commit(s) in {}..HEAD (base {}) break traceability: {} — add `Pmat-Ticket: <id>` as a git trailer (git commit -m '<subject>' -m 'Pmat-Ticket: PMAT-NNN')", m.findings.len(), m.commits, mb7, base, joined),
                        }
                    }
                }
            }
        }
    }
}
