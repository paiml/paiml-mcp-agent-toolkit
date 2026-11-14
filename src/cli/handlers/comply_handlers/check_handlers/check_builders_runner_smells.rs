// Builder for the RS-01..RS-12 runner-smells group (pmat#1673). Included from check.rs.

fn build_runner_smells_checks(
    project_path: &Path,
    comply_config: &crate::models::comply_config::ComplyConfig,
) -> Vec<ComplianceCheck> {
    check_runner_smells(project_path)
        .into_iter()
        .map(|check| {
            let id = check_id(&check.name).to_string();
            filter_check_by_config(check, &id, comply_config)
        })
        .collect()
}
