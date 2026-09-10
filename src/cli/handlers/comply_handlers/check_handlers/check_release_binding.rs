
pub(crate) fn check_release_binding(
    project_path: &Path,
    comply_config: &crate::models::comply_config::ComplyConfig,
    github_snapshot: Option<&Path>,
) -> ComplianceCheck {
    let name = "CB-2114: Release Binding".to_string();

    let inputs = match roadmap_inputs(project_path, comply_config, "cb-2114", github_snapshot) {
        Ok(i) => i,
        Err(mut e) => {
            e.name = name;
            return e;
        }
    };

    let report = crate::services::work_sync::linkage::release_binding(&inputs.roadmap, &inputs.snapshot);

    let mut check = ComplianceCheck {
        name,
        status: CheckStatus::Skip,
        severity: crate::models::comply_config::CheckSeverity::Error.into(),
        message: String::new(),
    };

    if report.findings.is_empty() {
        check.status = CheckStatus::Pass;
        check.message = format!(
            "{} open item(s) (status not completed or cancelled, goal-mode.md §4.2) each carry release: naming an existing milestone their issue is on: bound {}; snapshot: {} taken {}",
            report.open_items, report.bound, inputs.source, inputs.taken_at
        );
    } else {
        check.status = CheckStatus::Fail;
        let n = report.findings.len();
        let mut renderings = Vec::new();
        let mut count_no_release = 0;
        let mut count_prefixed = 0;
        let mut count_no_milestone = 0;
        let mut count_not_on_milestone = 0;
        let mut count_no_issue = 0;

        for f in &report.findings {
            match f {
                crate::services::work_sync::linkage::ReleaseFinding::NoRelease{..} => count_no_release += 1,
                crate::services::work_sync::linkage::ReleaseFinding::Prefixed{..} => count_prefixed += 1,
                crate::services::work_sync::linkage::ReleaseFinding::NoMilestone{..} => count_no_milestone += 1,
                crate::services::work_sync::linkage::ReleaseFinding::NotOnMilestone{..} => count_not_on_milestone += 1,
                crate::services::work_sync::linkage::ReleaseFinding::NoIssue{..} => count_no_issue += 1,
            }
            if renderings.len() < 8 {
                renderings.push(f.render());
            }
        }
        
        let more = if n > 8 {
            format!(" (+{} more)", n - 8)
        } else {
            String::new()
        };
        
        check.message = format!(
            "{} finding(s) — NO-RELEASE {}, PREFIXED {}, NO-MILESTONE {}, NOT-ON-MILESTONE {}, NO-ISSUE {}: {}{}; `pmat work sync --direction github-to-yaml` projects release: from the issue's milestone and is its only writer (goal-mode.md §4.1); a milestone is created, and an issue put on it, on GitHub — the authority for scope; snapshot: {} taken {}",
            n, count_no_release, count_prefixed, count_no_milestone, count_not_on_milestone, count_no_issue,
            renderings.join("; "), more, inputs.source, inputs.taken_at
        );
    }

    check
}
