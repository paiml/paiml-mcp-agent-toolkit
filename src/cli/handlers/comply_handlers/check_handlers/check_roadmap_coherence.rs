pub(crate) fn check_roadmap_coherence(
    project_path: &std::path::Path,
    comply_config: &crate::models::comply_config::ComplyConfig,
    github_snapshot: Option<&std::path::Path>,
) -> crate::cli::handlers::comply_handlers::check_handlers::types::ComplianceCheck {
    let name = "CB-2115: Roadmap Coherence".to_string();

    let inputs = match roadmap_inputs(project_path, comply_config, "cb-2115", github_snapshot) {
        Ok(i) => i,
        Err(mut e) => {
            e.name = name;
            return e;
        }
    };

    let mut grace_minutes = crate::services::work_sync::DEFAULT_GRACE_MINUTES;
    if let Some(config) = comply_config.checks.get("cb-2115") {
        if let Some(val) = config.options.get("grace_minutes") {
            if let Some(i) = val.as_i64() {
                grace_minutes = i;
            } else {
                return crate::cli::handlers::comply_handlers::check_handlers::types::ComplianceCheck {
                    name,
                    status: crate::cli::handlers::comply_handlers::check_handlers::types::CheckStatus::Fail,
                    severity: crate::models::comply_config::CheckSeverity::Error.into(),
                    message: "not_measured: cb-2115 option grace_minutes is not an integer".to_string(),
                };
            }
        }
    }

    let settings = crate::services::work_sync::Settings::new(chrono::Utc::now(), grace_minutes);
    let report = crate::services::work_sync::check(&inputs.roadmap, &inputs.snapshot, &settings);

    let mut check = crate::cli::handlers::comply_handlers::check_handlers::types::ComplianceCheck {
        name,
        status: crate::cli::handlers::comply_handlers::check_handlers::types::CheckStatus::Skip,
        severity: crate::models::comply_config::CheckSeverity::Info.into(),
        message: String::new(),
    };

    if report.is_coherent() {
        check.status = crate::cli::handlers::comply_handlers::check_handlers::types::CheckStatus::Pass;
        check.message = format!(
            "{} open item(s) and {} open issue(s) are in bijection: matched {}, tolerated {} inside the {}-minute grace window (goal-mode.md §5); snapshot: {} taken {}",
            report.open_items, report.open_issues, report.matched, report.tolerated, grace_minutes, inputs.source, inputs.taken_at
        );
    } else {
        check.status = crate::cli::handlers::comply_handlers::check_handlers::types::CheckStatus::Fail;
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
            inputs.source,
            inputs.taken_at
        );
    }

    check
}
