/// CB-2112: Ticket Linkage (goal-mode.md §7 row CB-2112, invariant A).
///
/// RED stub: judges nothing. The implementation phase replaces the body.
pub(crate) fn check_ticket_linkage(
    project_path: &Path,
    comply_config: &crate::models::comply_config::ComplyConfig,
    github_snapshot: Option<&Path>,
) -> ComplianceCheck {
    let _ = (project_path, comply_config, github_snapshot);
    ComplianceCheck {
        name: "CB-2112: Ticket Linkage".to_string(),
        status: CheckStatus::Skip,
        severity: crate::models::comply_config::CheckSeverity::Info.into(),
        message: "stub".to_string(),
    }
}
