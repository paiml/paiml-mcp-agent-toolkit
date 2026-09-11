/// Build the spec-review check (CB-2111).
fn build_spec_review_checks(
    project_path: &Path,
    comply_config: &crate::models::comply_config::ComplyConfig,
) -> Vec<ComplianceCheck> {
    vec![filter_check_by_config(
        check_spec_reviews(project_path),
        "cb-2111",
        comply_config,
    )]
}
