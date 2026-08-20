// CB-2102 builder. Included from check.rs — no `use` imports here.

/// CB-2102: has any ratcheted metric got worse than the value this repository
/// last agreed to?
fn build_ratchet_checks(
    project_path: &Path,
    comply_config: &crate::models::comply_config::ComplyConfig,
) -> Vec<ComplianceCheck> {
    vec![filter_check_by_config(
        check_metrics_ratchet(project_path),
        "cb-2102",
        comply_config,
    )]
}
