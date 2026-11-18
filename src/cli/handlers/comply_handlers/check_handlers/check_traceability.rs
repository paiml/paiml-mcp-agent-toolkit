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
                Range::PullRequest { base, merge_base } => pull_request_verdict(&m, &base, &merge_base),
            }
        }
    }
}

