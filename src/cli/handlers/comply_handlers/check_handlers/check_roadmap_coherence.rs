/// CB-2115: Roadmap Coherence (goal-mode.md §5.1, §5.2, §5.4).
pub(crate) fn check_roadmap_coherence(
    project_path: &Path,
    comply_config: &crate::models::comply_config::ComplyConfig,
    github_snapshot: Option<&Path>,
) -> ComplianceCheck {
    use crate::services::commit_traceability::{self as ct, Inputs};
    let mut check = ComplianceCheck {
        name: "CB-2115: Roadmap Coherence".to_string(),
        status: CheckStatus::Skip,
        severity: crate::models::comply_config::CheckSeverity::Info.into(),
        message: String::new(),
    };

    let roadmap_path = project_path.join("docs/roadmaps/roadmap.yaml");
    if !roadmap_path.exists() {
        // The same line CB-2113 draws: a roadmap that was never committed is a
        // structural absence (Skip); one that was committed and is now gone is
        // a deleted input, and deleting a gate's input is not a way of passing
        // it (goal-mode.md doctrine 2).
        if matches!(ct::inputs(project_path), Inputs::RoadmapDeleted) {
            check.status = CheckStatus::Fail;
            check.severity = crate::models::comply_config::CheckSeverity::Error.into();
            check.message = "not_measured: docs/roadmaps/roadmap.yaml was committed and is now gone — deleting a gate's input is not a way of passing it (goal-mode.md doctrine 2)".to_string();
            return check;
        }
        check.message = "no docs/roadmaps/roadmap.yaml — this project does not track work in a roadmap".to_string();
        return check;
    }

    let roadmap = match crate::services::roadmap_service::RoadmapService::new(&roadmap_path).load() {
        Ok(r) => r,
        Err(e) => {
            check.status = CheckStatus::Fail;
            check.severity = crate::models::comply_config::CheckSeverity::Error.into();
            check.message = format!("not_measured: docs/roadmaps/roadmap.yaml does not parse: {e} — an input the rule expected and could not read is a failure, not a pass (goal-mode.md doctrine 2)");
            return check;
        }
    };

    let mut grace_minutes = crate::services::work_sync::DEFAULT_GRACE_MINUTES;

    if let Some(config) = comply_config.checks.get("cb-2115") {
        // A snapshot FILE may only arrive on the command line
        // (`pmat comply check --github-snapshot <file>`): a path committed in
        // .pmat.yaml would let the gate judge a fixture instead of GitHub on
        // every CI run — a bypass token, which goal-mode.md §12 forbids. The
        // three quorum lanes on PMAT-722 all found it; refused, never read.
        if config.options.contains_key("snapshot") {
            check.status = CheckStatus::Fail;
            check.severity = crate::models::comply_config::CheckSeverity::Error.into();
            check.message = "not_measured: cb-2115 option `snapshot` in .pmat.yaml is refused — a snapshot file may only be given on the command line (pmat comply check --github-snapshot <file>), never committed in the tree, or the gate would judge a fixture instead of GitHub (goal-mode.md §12)".to_string();
            return check;
        }
        if let Some(val) = config.options.get("grace_minutes") {
            if let Some(i) = val.as_i64() {
                grace_minutes = i;
            } else {
                check.status = CheckStatus::Fail;
                check.severity = crate::models::comply_config::CheckSeverity::Error.into();
                check.message = "not_measured: cb-2115 option grace_minutes is not an integer".to_string();
                return check;
            }
        }
    }

    let names_issues = roadmap.roadmap.iter().filter(|i| i.github_issue.is_some()).count();
    let source = if let Some(s) = github_snapshot {
        crate::services::work_sync::github::SnapshotSource::File(project_path.join(s))
    } else {
        let repo = match &roadmap.github_repo {
            Some(r) => r.clone(),
            None => {
                match crate::cli::handlers::work_handlers::core_handlers::github::detect_github_repo(&project_path.to_path_buf()) {
                    Ok(Some(r)) => r,
                    Ok(None) if names_issues == 0 => {
                        check.message = "no GitHub repository and no item names a github_issue — nothing for the roadmap to be coherent with: set github_repo in docs/roadmaps/roadmap.yaml or add an origin remote".to_string();
                        return check;
                    }
                    Ok(None) => {
                        // Items name issues, so GitHub is an input this rule expected;
                        // a repository that no longer resolves is not a pass.
                        check.status = CheckStatus::Fail;
                        check.severity = crate::models::comply_config::CheckSeverity::Error.into();
                        check.message = format!("not_measured: no GitHub repository resolves (github_repo is null and no origin remote) while {names_issues} item(s) name a github_issue — an input the rule expected and could not read is a failure, not a pass (goal-mode.md doctrine 2)");
                        return check;
                    }
                    Err(e) => {
                        check.status = CheckStatus::Fail;
                        check.severity = crate::models::comply_config::CheckSeverity::Error.into();
                        check.message = format!("not_measured: detect github repo failed: {e} — an input the rule expected and could not read is a failure, not a pass (goal-mode.md doctrine 2)");
                        return check;
                    }
                }
            }
        };
        crate::services::work_sync::github::SnapshotSource::Live { repo }
    };

    let snapshot = match source.load() {
        Ok(s) => s,
        Err(e) => {
            check.status = CheckStatus::Fail;
            check.severity = crate::models::comply_config::CheckSeverity::Error.into();
            check.message = format!("not_measured: snapshot {source}: {e} — an input the rule expected and could not read is a failure, not a pass (goal-mode.md doctrine 2)");
            return check;
        }
    };

    let settings = crate::services::work_sync::Settings::new(chrono::Utc::now(), grace_minutes);
    let report = crate::services::work_sync::check(&roadmap, &snapshot, &settings);

    let taken_at = snapshot.taken_at.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);

    if report.is_coherent() {
        check.status = CheckStatus::Pass;
        check.message = format!(
            "{} open item(s) and {} open issue(s) are in bijection: matched {}, tolerated {} inside the {}-minute grace window (goal-mode.md §5); snapshot: {} taken {}",
            report.open_items, report.open_issues, report.matched, report.tolerated, grace_minutes, source, taken_at
        );
    } else {
        check.status = CheckStatus::Fail;
        check.severity = crate::models::comply_config::CheckSeverity::Error.into();
        let n = report.findings.len();
        let mut renderings = Vec::new();
        for f in &report.findings {
            if renderings.len() < 8 {
                renderings.push(f.render());
            } else {
                break;
            }
        }
        let more = if n > 8 {
            format!(" (+{} more)", n - 8)
        } else {
            String::new()
        };
        check.message = format!(
            "{} finding(s) — COLLISION {}, ORPHAN-ROADMAP {}, ORPHAN-GITHUB {}, DRIFT {}: {}{}; `pmat work sync --check-only` prints the full report — the orphans are fixed by `pmat work sync --direction yaml-to-github|github-to-yaml`, a COLLISION by a human, never by the sync (goal-mode.md §5.4); snapshot: {} taken {}",
            n,
            report.count("COLLISION"),
            report.count("ORPHAN-ROADMAP"),
            report.count("ORPHAN-GITHUB"),
            report.count("DRIFT"),
            renderings.join("; "),
            more,
            source,
            taken_at
        );
    }

    check
}
