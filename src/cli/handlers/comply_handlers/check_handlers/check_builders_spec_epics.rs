/// Build the spec-epic check (CB-2110).
fn build_spec_epic_checks(
    project_path: &Path,
    comply_config: &crate::models::comply_config::ComplyConfig,
    github_snapshot: Option<&Path>,
) -> Vec<ComplianceCheck> {
    vec![filter_check_by_config(
        check_spec_epics(project_path, comply_config, github_snapshot),
        "cb-2110",
        comply_config,
    )]
}
