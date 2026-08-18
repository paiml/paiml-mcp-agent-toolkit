// CB-1411 builder. Included from check.rs — no `use` imports here.

/// CB-1411: does a required status check actually reach the error-severity
/// comply rules (EV-1)?
fn build_gate_effect_checks(
    project_path: &Path,
    comply_config: &crate::models::comply_config::ComplyConfig,
) -> Vec<ComplianceCheck> {
    vec![filter_check_by_config(
        check_comply_gate_effect(project_path, comply_config),
        "cb-1411",
        comply_config,
    )]
}
