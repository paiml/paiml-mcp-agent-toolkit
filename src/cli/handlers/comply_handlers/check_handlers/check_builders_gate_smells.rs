// GS-01, GS-02, GS-07, GS-13 builder (#1673). Included from check.rs — no `use` imports here.

/// The gate-smell checks, each passed through the repository's `.pmat.yaml` filter.
fn build_gate_smells_checks(
    project_path: &Path,
    comply_config: &crate::models::comply_config::ComplyConfig,
) -> Vec<ComplianceCheck> {
    check_gate_smells(project_path)
        .into_iter()
        .map(|check| {
            let id = check_id(&check.name).to_string();
            filter_check_by_config(check, &id, comply_config)
        })
        .collect()
}
