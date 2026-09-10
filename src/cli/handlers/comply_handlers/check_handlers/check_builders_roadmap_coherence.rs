/// Build roadmap-coherence checks (CB-2115)
fn build_roadmap_coherence_checks(project_path: &Path, comply_config: &crate::models::comply_config::ComplyConfig, github_snapshot: Option<&Path>) -> Vec<ComplianceCheck> {
    vec![filter_check_by_config(check_roadmap_coherence(project_path, comply_config, github_snapshot), "cb-2115", comply_config)]
}
