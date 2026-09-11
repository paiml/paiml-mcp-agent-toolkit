// Included from check.rs — do NOT add `use` imports or `#!` attributes here.

/// CB-2111: Spec Reviews (goal-mode.md §6, §7 row CB-2111; invariant E.1).
///
/// RED stub (PMAT-1299): a row with no judgement, so every rule test fails at
/// its assertion.
pub(crate) fn check_spec_reviews(project_path: &Path) -> ComplianceCheck {
    use crate::models::comply_config::CheckSeverity;
    let _ = project_path;
    ComplianceCheck {
        name: "CB-2111: Spec Reviews".to_string(),
        status: CheckStatus::Pass,
        severity: CheckSeverity::Info.into(),
        message: String::new(),
    }
}
