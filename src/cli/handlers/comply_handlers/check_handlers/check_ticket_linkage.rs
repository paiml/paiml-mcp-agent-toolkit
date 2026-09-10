/// CB-2112: Ticket Linkage (goal-mode.md §7 row CB-2112; invariant A — no
/// work without a ticket linked to the roadmap).
///
/// Every open item (status not completed or cancelled, §4.2) names a
/// `github_issue` that exists, is open, and whose number is the item's
/// numeric tail (an open issue labelled `no-roadmap` still links here — the
/// label is §5.1's and CB-2115's business, one place per judgement): the id `pmat work add
/// --github-issue N` mints is `<PREFIX>-N` (PMAT-714, #1240), so the tail is
/// the one link two branches cannot mint twice. One finding per item, the
/// first clause that fails. The judgement is `work_sync::linkage::
/// ticket_linkage`; the inputs and their early verdicts are `roadmap_inputs`.
pub(crate) fn check_ticket_linkage(
    project_path: &Path,
    comply_config: &crate::models::comply_config::ComplyConfig,
    github_snapshot: Option<&Path>,
) -> ComplianceCheck {
    use crate::services::work_sync::linkage::{ticket_linkage, LinkFinding};
    let name = "CB-2112: Ticket Linkage".to_string();
    let inputs = match roadmap_inputs(project_path, comply_config, "cb-2112", github_snapshot) {
        Ok(i) => i,
        Err(mut early) => {
            early.name = name;
            return early;
        }
    };
    let report = ticket_linkage(&inputs.roadmap, &inputs.snapshot);
    let n = report.findings.len();
    let pass = format!(
        "{} open item(s) (status not completed or cancelled, goal-mode.md §4.2) each name an open issue numbered as the item's numeric tail: linked {}; snapshot: {} taken {}",
        report.open_items, report.linked, inputs.source, inputs.taken_at
    );
    let fail = format!(
        "{n} finding(s) — {}: {}; an item is minted from its issue by `pmat work add --github-issue N` (#1240), which is why the number must be the tail — an item minted before that whose issue is not its tail needs re-minting under its issue (no sync renames an id); `pmat work sync --direction yaml-to-github` opens an issue for an unlinked item but cannot make the number match; snapshot: {} taken {}",
        class_counts(report.findings.iter().map(LinkFinding::class)),
        first_eight(report.findings.iter().map(LinkFinding::render), n),
        inputs.source,
        inputs.taken_at
    );
    roadmap_verdict(name, n == 0, pass, fail)
}
