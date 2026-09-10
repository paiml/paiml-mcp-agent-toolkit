/// Build traceability checks
fn build_traceability_checks(project_path: &Path, comply_config: &crate::models::comply_config::ComplyConfig) -> Vec<ComplianceCheck> {
    vec![filter_check_by_config(check_commit_traceability(project_path), "cb-2113", comply_config)]
}
