// Included from check.rs — do NOT add `use` imports or `#!` attributes here.

/// CB-2110: Spec Epics (goal-mode.md §4.3, §7 row CB-2110; invariant E —
/// `docs/specifications/*` MUST be linked to a GitHub epic, and tickets).
///
/// Every spec under `docs/specifications/` opens with YAML front-matter
/// (`epic:`, `status:`, `vendors:`); every `active` spec names an open issue
/// labelled `epic` that has at least one sub-issue — membership is GitHub's
/// sub-issue relation, read into the snapshot for every issue labelled
/// `epic`. `superseded` and `historical` are off the epic leg, never off the
/// parse leg, and every verdict names them (§4.3: an escape hatch nobody can
/// see is a hole). The parse leg is offline; the snapshot is loaded only when
/// an active spec names an epic. The judgement is `services::spec_epic`; the
/// files are `list_specs`; the snapshot and its early verdicts are the same
/// preamble the roadmap rules use.
pub(crate) fn check_spec_epics(
    project_path: &Path,
    comply_config: &crate::models::comply_config::ComplyConfig,
    github_snapshot: Option<&Path>,
) -> ComplianceCheck {
    use crate::models::comply_config::CheckSeverity;
    let _ = (project_path, comply_config, github_snapshot);
    ComplianceCheck {
        name: "CB-2110: Spec Epics".to_string(),
        status: CheckStatus::Fail,
        severity: CheckSeverity::Error.into(),
        message: "not_measured: not implemented (PMAT-728 phase 2)".to_string(),
    }
}
