
pub(crate) struct RoadmapInputs {
    pub roadmap: crate::models::roadmap::Roadmap,
    pub snapshot: crate::services::work_sync::GithubSnapshot,
    pub source: crate::services::work_sync::github::SnapshotSource,
    pub taken_at: String,
}

pub(crate) fn roadmap_inputs(
    project_path: &std::path::Path,
    comply_config: &crate::models::comply_config::ComplyConfig,
    rule_key: &str,
    github_snapshot: Option<&std::path::Path>,
) -> Result<RoadmapInputs, crate::cli::handlers::comply_handlers::check_handlers::types::ComplianceCheck> {
    let mut check = crate::cli::handlers::comply_handlers::check_handlers::types::ComplianceCheck {
        name: String::new(),
        status: crate::cli::handlers::comply_handlers::check_handlers::types::CheckStatus::Skip,
        severity: crate::models::comply_config::CheckSeverity::Info.into(),
        message: String::new(),
    };

    let roadmap_path = project_path.join("docs/roadmaps/roadmap.yaml");
    if !roadmap_path.exists() {
        if matches!(crate::services::commit_traceability::inputs(project_path), crate::services::commit_traceability::Inputs::RoadmapDeleted) {
            check.status = crate::cli::handlers::comply_handlers::check_handlers::types::CheckStatus::Fail;
            check.severity = crate::models::comply_config::CheckSeverity::Error.into();
            check.message = "not_measured: docs/roadmaps/roadmap.yaml was committed and is now gone — deleting a gate's input is not a way of passing it (goal-mode.md doctrine 2)".to_string();
            return Err(check);
        }
        check.message = "no docs/roadmaps/roadmap.yaml — this project does not track work in a roadmap".to_string();
        return Err(check);
    }

    let roadmap = match crate::services::roadmap_service::RoadmapService::new(&roadmap_path).load() {
        Ok(r) => r,
        Err(e) => {
            check.status = crate::cli::handlers::comply_handlers::check_handlers::types::CheckStatus::Fail;
            check.severity = crate::models::comply_config::CheckSeverity::Error.into();
            check.message = format!("not_measured: docs/roadmaps/roadmap.yaml does not parse: {e} — an input the rule expected and could not read is a failure, not a pass (goal-mode.md doctrine 2)");
            return Err(check);
        }
    };

    if let Some(config) = comply_config.checks.get(rule_key) {
        if config.options.contains_key("snapshot") {
            check.status = crate::cli::handlers::comply_handlers::check_handlers::types::CheckStatus::Fail;
            check.severity = crate::models::comply_config::CheckSeverity::Error.into();
            check.message = format!("not_measured: {} option `snapshot` in .pmat.yaml is refused — a snapshot file may only be given on the command line (pmat comply check --github-snapshot <file>), never committed in the tree, or the gate would judge a fixture instead of GitHub (goal-mode.md §12)", rule_key);
            return Err(check);
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
                        return Err(check);
                    }
                    Ok(None) => {
                        check.status = crate::cli::handlers::comply_handlers::check_handlers::types::CheckStatus::Fail;
                        check.severity = crate::models::comply_config::CheckSeverity::Error.into();
                        check.message = format!("not_measured: no GitHub repository resolves (github_repo is null and no origin remote) while {names_issues} item(s) name a github_issue — an input the rule expected and could not read is a failure, not a pass (goal-mode.md doctrine 2)");
                        return Err(check);
                    }
                    Err(e) => {
                        check.status = crate::cli::handlers::comply_handlers::check_handlers::types::CheckStatus::Fail;
                        check.severity = crate::models::comply_config::CheckSeverity::Error.into();
                        check.message = format!("not_measured: detect github repo failed: {e} — an input the rule expected and could not read is a failure, not a pass (goal-mode.md doctrine 2)");
                        return Err(check);
                    }
                }
            }
        };
        crate::services::work_sync::github::SnapshotSource::Live { repo }
    };

    let snapshot = match source.load() {
        Ok(s) => s,
        Err(e) => {
            check.status = crate::cli::handlers::comply_handlers::check_handlers::types::CheckStatus::Fail;
            check.severity = crate::models::comply_config::CheckSeverity::Error.into();
            check.message = format!("not_measured: snapshot {source}: {e} — an input the rule expected and could not read is a failure, not a pass (goal-mode.md doctrine 2)");
            return Err(check);
        }
    };

    let taken_at = snapshot.taken_at.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);

    Ok(RoadmapInputs {
        roadmap,
        snapshot,
        source,
        taken_at,
    })
}
