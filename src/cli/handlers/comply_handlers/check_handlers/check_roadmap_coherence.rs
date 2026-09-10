/// CB-2115: Roadmap Coherence — RED stub; the rule is implemented in the next commit.
pub(crate) fn check_roadmap_coherence(
    _project_path: &Path,
    _comply_config: &crate::models::comply_config::ComplyConfig,
) -> ComplianceCheck {
    ComplianceCheck {
        name: "CB-2115: Roadmap Coherence".to_string(),
        status: CheckStatus::Skip,
        severity: crate::models::comply_config::CheckSeverity::Info.into(),
        message: "RED stub: CB-2115 is not implemented".to_string(),
    }
}
