/// Build the API-measured runner-smell checks (RS-13 runs per merge, RS-14
/// cancelled share, RS-15 queue p95; pmat#1673). `repo` is the resolved
/// GitHub repository (`Ok(None)`: this tree has none, a structural absence
/// that Skips; `Err`: the lookup itself failed, which is `not_measured`).
/// The read goes through the injected `api`; a failed or rate-limited read
/// is `not_measured` on all three rows, never a pass.
fn build_runner_api_smells_checks(
    comply_config: &crate::models::comply_config::ComplyConfig,
    repo: Result<Option<String>, String>,
    api: &dyn crate::services::runner_api_smells::ActionsApi,
    now: chrono::DateTime<chrono::Utc>,
) -> Vec<ComplianceCheck> {
    use crate::models::comply_config::CheckSeverity;
    use crate::services::runner_api_smells as ras;

    const ROWS: [(&str, &str); 3] = [
        ("rs-13", "RS-13: Runs per Merge"),
        ("rs-14", "RS-14: Cancelled Run Share"),
        ("rs-15", "RS-15: Run Queue Time p95"),
    ];
    let uniform = |status: CheckStatus, severity: CheckSeverity, message: String| -> Vec<ComplianceCheck> {
        ROWS.iter()
            .map(|(key, name)| {
                filter_check_by_config(
                    ComplianceCheck { name: (*name).to_string(), status, severity: severity.into(), message: message.clone() },
                    key,
                    comply_config,
                )
            })
            .collect()
    };
    let not_measured = |why: String| {
        uniform(
            CheckStatus::Fail,
            CheckSeverity::Error,
            format!("not_measured: {why} — an input the rule expected and could not read is a failure, not a pass"),
        )
    };

    let repo = match repo {
        Ok(Some(r)) => r,
        Ok(None) => return uniform(CheckStatus::Skip, CheckSeverity::Info, "no GitHub repository resolves — there is no Actions history to measure".to_string()),
        Err(e) => return not_measured(format!("detect github repo failed: {e}")),
    };

    let lookup = |key: &str| -> Option<Option<f64>> {
        ROWS.iter().find_map(|(rk, _)| comply_config.checks.get(*rk).and_then(|c| c.options.get(key)).map(serde_yaml_ng::Value::as_f64))
    };
    let thresholds = match ras::Thresholds::from_lookup(&lookup) {
        Ok(t) => t,
        Err(e) => return not_measured(e),
    };

    let findings = ras::evaluate(api, &repo, now - chrono::Duration::hours(24), &thresholds);
    findings
        .into_iter()
        .zip(ROWS.iter())
        .map(|(f, (key, name))| {
            let (status, severity, message) = match f.verdict {
                ras::Verdict::Pass => (CheckStatus::Pass, CheckSeverity::Info, f.message),
                ras::Verdict::Warn => (CheckStatus::Warn, CheckSeverity::Warning, f.message),
                ras::Verdict::Fail => (CheckStatus::Fail, CheckSeverity::Error, f.message),
                ras::Verdict::NotMeasured => (CheckStatus::Fail, CheckSeverity::Error, format!("not_measured: {}", f.message)),
            };
            filter_check_by_config(ComplianceCheck { name: (*name).to_string(), status, severity: severity.into(), message }, key, comply_config)
        })
        .collect()
}

/// The live group thunk: resolve the repo from the origin remote and read it
/// through the `gh` CLI.
fn build_runner_api_smells_live(project_path: &Path, comply_config: &crate::models::comply_config::ComplyConfig) -> Vec<ComplianceCheck> {
    let repo = crate::cli::handlers::work_handlers::core_handlers::github::detect_github_repo(&project_path.to_path_buf()).map_err(|e| e.to_string());
    build_runner_api_smells_checks(comply_config, repo, &crate::services::runner_api_smells::GhCli, chrono::Utc::now())
}
