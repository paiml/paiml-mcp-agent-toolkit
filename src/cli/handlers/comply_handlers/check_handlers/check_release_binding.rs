/// CB-2114: Release Binding (goal-mode.md §7 row CB-2114, invariants B/F1).
///
/// RED stub: judges nothing. The implementation phase replaces the body.
pub(crate) fn check_release_binding(
    project_path: &Path,
    comply_config: &crate::models::comply_config::ComplyConfig,
    github_snapshot: Option<&Path>,
) -> ComplianceCheck {
    let _ = (project_path, comply_config, github_snapshot);
    ComplianceCheck {
        name: "CB-2114: Release Binding".to_string(),
        status: CheckStatus::Skip,
        severity: crate::models::comply_config::CheckSeverity::Info.into(),
        message: "stub".to_string(),
    }
}
