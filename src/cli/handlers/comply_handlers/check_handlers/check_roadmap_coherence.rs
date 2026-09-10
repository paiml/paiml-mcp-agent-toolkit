/// CB-2115: Roadmap Coherence (goal-mode.md §5.1, §5.2, §5.4; invariant D).
///
/// The open roadmap items and the open, unlabelled GitHub issues are in
/// bijection (§5.1, tolerance zero: COLLISION, ORPHAN-ROADMAP, ORPHAN-GITHUB)
/// and no matched pair has disagreed past the grace window (§5.2, a bound in
/// time: DRIFT). The judgement is the step-3 engine, `work_sync::check`; the
/// inputs and their early verdicts are `roadmap_inputs`. `grace_minutes` is
/// read from `.pmat.yaml` `comply.checks.cb-2115.options`.
pub(crate) fn check_roadmap_coherence(
    project_path: &Path,
    comply_config: &crate::models::comply_config::ComplyConfig,
    github_snapshot: Option<&Path>,
) -> ComplianceCheck {
    let name = "CB-2115: Roadmap Coherence".to_string();
    let inputs = match roadmap_inputs(project_path, comply_config, "cb-2115", github_snapshot) {
        Ok(i) => i,
        Err(mut early) => {
            early.name = name;
            return early;
        }
    };

    let mut grace_minutes = crate::services::work_sync::DEFAULT_GRACE_MINUTES;
    if let Some(val) = comply_config.checks.get("cb-2115").and_then(|c| c.options.get("grace_minutes")) {
        match val.as_i64() {
            Some(i) => grace_minutes = i,
            None => {
                return ComplianceCheck {
                    name,
                    status: CheckStatus::Fail,
                    severity: crate::models::comply_config::CheckSeverity::Error.into(),
                    message: "not_measured: cb-2115 option grace_minutes is not an integer".to_string(),
                };
            }
        }
    }

    let settings = crate::services::work_sync::Settings::new(chrono::Utc::now(), grace_minutes);
    let report = crate::services::work_sync::check(&inputs.roadmap, &inputs.snapshot, &settings);
    let n = report.findings.len();
    let pass = format!(
        "{} open item(s) and {} open issue(s) are in bijection: matched {}, tolerated {} inside the {}-minute grace window (goal-mode.md §5); snapshot: {} taken {}",
        report.open_items, report.open_issues, report.matched, report.tolerated, grace_minutes, inputs.source, inputs.taken_at
    );
    let fail = format!(
        "{n} finding(s) — COLLISION {}, ORPHAN-ROADMAP {}, ORPHAN-GITHUB {}, DRIFT {}: {}; `pmat work sync --check-only` prints the full report — the orphans are fixed by `pmat work sync --direction yaml-to-github|github-to-yaml`, a COLLISION by a human, never by the sync (goal-mode.md §5.4); snapshot: {} taken {}",
        report.count("COLLISION"),
        report.count("ORPHAN-ROADMAP"),
        report.count("ORPHAN-GITHUB"),
        report.count("DRIFT"),
        first_eight(report.findings.iter().map(crate::services::work_sync::Finding::render), n),
        inputs.source,
        inputs.taken_at
    );
    roadmap_verdict(name, report.is_coherent(), pass, fail)
}
