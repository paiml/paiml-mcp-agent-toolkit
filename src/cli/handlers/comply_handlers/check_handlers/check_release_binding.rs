/// CB-2114: Release Binding (goal-mode.md §7 row CB-2114; invariants B/F1 —
/// all work grouped by a tagged release).
///
/// Every open item (status not completed or cancelled, §4.2) carries a
/// `release:` that is the bare semver string (§4.1: the `v` is added in
/// exactly one place, the tag), names a milestone with that EXACT title
/// (RR-RELEASE — a rename IS a release rename, and this rule going red
/// naming it is the intended behaviour), that milestone is open (a shipped
/// release cannot hold open work: a tag is cut only when its milestone has
/// zero open issues), and its issue is on that milestone.
/// The roadmap's `release` is a projection `pmat work sync` writes; the
/// milestone is the authority for scope. One finding per item, the first
/// clause that fails. The judgement is `work_sync::linkage::release_binding`.
pub(crate) fn check_release_binding(
    project_path: &Path,
    comply_config: &crate::models::comply_config::ComplyConfig,
    github_snapshot: Option<&Path>,
) -> ComplianceCheck {
    use crate::services::work_sync::linkage::{release_binding, ReleaseFinding};
    let name = "CB-2114: Release Binding".to_string();
    let inputs = match roadmap_inputs(project_path, comply_config, "cb-2114", github_snapshot) {
        Ok(i) => i,
        Err(mut early) => {
            early.name = name;
            return early;
        }
    };
    let report = release_binding(&inputs.roadmap, &inputs.snapshot);
    let n = report.findings.len();
    let count = |class: &str| report.findings.iter().filter(|f| f.class() == class).count();
    let pass = format!(
        "{} open item(s) (status not completed or cancelled, goal-mode.md §4.2) each carry release: naming an open milestone their issue is on: bound {}; snapshot: {} taken {}",
        report.open_items, report.bound, inputs.source, inputs.taken_at
    );
    let fail = format!(
        "{n} finding(s) — NO-RELEASE {}, PREFIXED {}, NO-MILESTONE {}, MILESTONE-CLOSED {}, NOT-ON-MILESTONE {}, NO-ISSUE {}: {}; `pmat work sync --direction github-to-yaml` projects release: from the issue's milestone and is its only writer (goal-mode.md §4.1); a milestone is created, and an issue put on it, on GitHub — the authority for scope; snapshot: {} taken {}",
        count("NO-RELEASE"),
        count("PREFIXED"),
        count("NO-MILESTONE"),
        count("MILESTONE-CLOSED"),
        count("NOT-ON-MILESTONE"),
        count("NO-ISSUE"),
        first_eight(report.findings.iter().map(ReleaseFinding::render), n),
        inputs.source,
        inputs.taken_at
    );
    roadmap_verdict(name, n == 0, pass, fail)
}
