/// Build the ticket-linkage and release-binding checks (CB-2112, CB-2114).
fn build_ticket_release_checks(
    project_path: &Path,
    comply_config: &crate::models::comply_config::ComplyConfig,
    github_snapshot: Option<&Path>,
) -> Vec<ComplianceCheck> {
    vec![
        filter_check_by_config(
            check_ticket_linkage(project_path, comply_config, github_snapshot),
            "cb-2112",
            comply_config,
        ),
        filter_check_by_config(
            check_release_binding(project_path, comply_config, github_snapshot),
            "cb-2114",
            comply_config,
        ),
    ]
}
