// CB-2101 builder. Included from check.rs — no `use` imports here.

/// CB-2101: does every number `.pmat-metrics.toml` writes down actually bound
/// anything this tree can measure?
fn build_coherence_checks(
    project_path: &Path,
    comply_config: &crate::models::comply_config::ComplyConfig,
) -> Vec<ComplianceCheck> {
    vec![filter_check_by_config(
        check_threshold_coherence(project_path),
        "cb-2101",
        comply_config,
    )]
}
