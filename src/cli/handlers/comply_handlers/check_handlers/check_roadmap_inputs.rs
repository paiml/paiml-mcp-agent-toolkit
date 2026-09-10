/// The inputs the roadmap-facing rules (CB-2112, CB-2114, CB-2115) judge
/// from, resolved once: the parsed roadmap, the GitHub snapshot, where it
/// came from and when it was taken.
pub(crate) struct RoadmapInputs {
    pub roadmap: crate::models::roadmap::Roadmap,
    pub snapshot: crate::services::work_sync::GithubSnapshot,
    pub source: crate::services::work_sync::github::SnapshotSource,
    /// `snapshot.taken_at`, RFC 3339 to the second, for the messages.
    pub taken_at: String,
}

/// Resolve the roadmap and the snapshot for the rule keyed `rule_key`
/// (`cb-2112`, `cb-2114`, `cb-2115`), or the early verdict when they cannot
/// be: `Err` carries the row, whose `name` the caller sets.
///
/// The early verdicts, each with its reason in the message (goal-mode.md
/// doctrine 2, §3.3):
/// - Skip — no roadmap was ever committed; or the roadmap declares
///   `github_enabled: false`, no repository resolves and no item names an
///   issue, so there is nothing for it to be coherent with. The same line
///   CB-2113 draws: a structural absence.
/// - Fail `not_measured:` — the roadmap was committed and is now gone
///   (deleting a gate's input is not a way of passing it); the roadmap does
///   not parse; the roadmap declares GitHub (or names issues) but no
///   repository resolves; the snapshot
///   cannot be read; or `.pmat.yaml` commits a `snapshot` path under the
///   rule's key. A snapshot FILE may only arrive on the command line
///   (`pmat comply check --github-snapshot <file>`): a path committed in the
///   tree would let every CI run judge a fixture instead of GitHub — a bypass
///   token, which §12 forbids. The three quorum lanes on PMAT-722 all found
///   it; it is refused, never read.
pub(crate) fn roadmap_inputs(
    project_path: &Path,
    comply_config: &crate::models::comply_config::ComplyConfig,
    rule_key: &str,
    github_snapshot: Option<&Path>,
) -> Result<RoadmapInputs, ComplianceCheck> {
    use crate::models::comply_config::CheckSeverity;
    use crate::services::commit_traceability::{self as ct, Inputs};
    use crate::services::work_sync::github::SnapshotSource;

    let skip = |message: String| ComplianceCheck {
        name: String::new(),
        status: CheckStatus::Skip,
        severity: CheckSeverity::Info.into(),
        message,
    };
    let not_measured = |message: String| ComplianceCheck {
        name: String::new(),
        status: CheckStatus::Fail,
        severity: CheckSeverity::Error.into(),
        message: format!("not_measured: {message} — an input the rule expected and could not read is a failure, not a pass (goal-mode.md doctrine 2)"),
    };

    let roadmap_path = project_path.join("docs/roadmaps/roadmap.yaml");
    if !roadmap_path.exists() {
        if matches!(ct::inputs(project_path), Inputs::RoadmapDeleted) {
            return Err(not_measured("docs/roadmaps/roadmap.yaml was committed and is now gone — deleting a gate's input is not a way of passing it".to_string()));
        }
        return Err(skip("no docs/roadmaps/roadmap.yaml — this project does not track work in a roadmap".to_string()));
    }

    let roadmap = match crate::services::roadmap_service::RoadmapService::new(&roadmap_path).load() {
        Ok(r) => r,
        Err(e) => return Err(not_measured(format!("docs/roadmaps/roadmap.yaml does not parse: {e}"))),
    };

    if comply_config.checks.get(rule_key).is_some_and(|c| c.options.contains_key("snapshot")) {
        return Err(not_measured(format!("{rule_key} option `snapshot` in .pmat.yaml is refused — a snapshot file may only be given on the command line (pmat comply check --github-snapshot <file>), never committed in the tree, or the gate would judge a fixture instead of GitHub (goal-mode.md §12)")));
    }

    let names_issues = roadmap.roadmap.iter().filter(|i| i.github_issue.is_some()).count();
    let source = match github_snapshot {
        Some(file) => SnapshotSource::File(project_path.join(file)),
        None => {
            let repo = match &roadmap.github_repo {
                Some(r) => r.clone(),
                None => match crate::cli::handlers::work_handlers::core_handlers::github::detect_github_repo(&project_path.to_path_buf()) {
                    Ok(Some(r)) => r,
                    // The roadmap itself says whether GitHub is in play
                    // (`github_enabled`, default true). Only a roadmap that
                    // declares it OFF, resolves no repository and names no
                    // issue has nothing to be coherent with — a structural
                    // absence (§3.3). One that declares GitHub and cannot
                    // reach it has a missing INPUT, and that is not a pass:
                    // the quorum on PMAT-724 (lanes 2 and 3) found the
                    // earlier Skip reachable by nulling github_repo alone.
                    Ok(None) if names_issues == 0 && !roadmap.github_enabled => {
                        return Err(skip("github_enabled is false, no GitHub repository resolves and no item names a github_issue — this roadmap does not track GitHub, so there is nothing for it to be coherent with: set github_enabled: true and github_repo in docs/roadmaps/roadmap.yaml, or add an origin remote".to_string()));
                    }
                    Ok(None) => {
                        return Err(not_measured(format!("no GitHub repository resolves (github_repo is null and no origin remote) while the roadmap declares github_enabled: {} and {names_issues} item(s) name a github_issue", roadmap.github_enabled)));
                    }
                    Err(e) => return Err(not_measured(format!("detect github repo failed: {e}"))),
                },
            };
            SnapshotSource::Live { repo }
        }
    };

    let snapshot = match source.load() {
        Ok(s) => s,
        Err(e) => return Err(not_measured(format!("snapshot {source}: {e}"))),
    };
    let taken_at = snapshot.taken_at.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);

    Ok(RoadmapInputs { roadmap, snapshot, source, taken_at })
}

/// The row a roadmap-facing rule returns once it has judged: Pass with the
/// measurement, or Fail (Error) with the first eight findings rendered, the
/// rest counted, and the fixer named.
fn roadmap_verdict(name: String, findings_empty: bool, pass: String, fail: String) -> ComplianceCheck {
    use crate::models::comply_config::CheckSeverity;
    if findings_empty {
        ComplianceCheck { name, status: CheckStatus::Pass, severity: CheckSeverity::Info.into(), message: pass }
    } else {
        ComplianceCheck { name, status: CheckStatus::Fail, severity: CheckSeverity::Error.into(), message: fail }
    }
}

/// The first eight renderings joined, then `(+k more)` — a message that
/// silently dropped the ninth finding would read as eight.
fn first_eight(renderings: impl Iterator<Item = String>, total: usize) -> String {
    let shown: Vec<String> = renderings.take(8).collect();
    let more = if total > 8 { format!(" (+{} more)", total - 8) } else { String::new() };
    format!("{}{more}", shown.join("; "))
}

/// `CLASS n` for every finding class with a non-zero count, in first-seen
/// order — never a class with 0. A header that enumerated every class made
/// `message.contains("PREFIXED")` true on ANY failure, and mutant M5 on
/// PMAT-724 (the capital-V arm dropped) survived a test and a control arm
/// that were both reading that header.
fn class_counts<'a>(classes: impl Iterator<Item = &'a str>) -> String {
    let mut order: Vec<&str> = Vec::new();
    let mut counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for c in classes {
        if !counts.contains_key(c) {
            order.push(c);
        }
        *counts.entry(c).or_insert(0) += 1;
    }
    order.iter().map(|c| format!("{c} {}", counts.get(c).copied().unwrap_or(0))).collect::<Vec<_>>().join(", ")
}
