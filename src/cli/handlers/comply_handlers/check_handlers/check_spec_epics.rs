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
/// preamble the roadmap rules use. A `docs/specifications` that was
/// committed and is now gone is `not_measured`, never Skip (§12).
pub(crate) fn check_spec_epics(
    project_path: &Path,
    comply_config: &crate::models::comply_config::ComplyConfig,
    github_snapshot: Option<&Path>,
) -> ComplianceCheck {
    use crate::models::comply_config::CheckSeverity;
    use crate::services::spec_epic::{parse_specs, bind_epics};
    
    let name = "CB-2110: Spec Epics".to_string();
    
    let skip = |message: String| ComplianceCheck {
        name: name.clone(),
        status: CheckStatus::Skip,
        severity: CheckSeverity::Info.into(),
        message,
    };
    
    let not_measured = |message: String| ComplianceCheck {
        name: name.clone(),
        status: CheckStatus::Fail,
        severity: CheckSeverity::Error.into(),
        message: format!("not_measured: {message} — an input the rule expected and could not read is a failure, not a pass (goal-mode.md doctrine 2)"),
    };
    
    // Never committed ⇒ absent by design: Skip. Committed at any point and
    // now gone (or emptied) ⇒ deleting a gate's input is not a way of passing
    // it — the line the roadmap preamble and CB-2102 draw for their inputs;
    // the quorum on PMAT-728 found the first cut skipping on `rm -rf
    // docs/specifications` (§12: a bypass). An unreadable history is
    // treated as "never": a repository with no history has nothing to have
    // deleted.
    let absent = || {
        match crate::services::metrics_ratchet::history::was_ever_committed(project_path, crate::services::spec_epic::SPECS_DIR) {
            Ok(true) => not_measured("docs/specifications was committed and is now gone — deleting a gate's input is not a way of passing it".to_string()),
            Ok(false) | Err(_) => skip("no docs/specifications — this project keeps no specifications, so there is no spec ↔ epic edge to check".to_string()),
        }
    };
    let specs = match list_specs(project_path) {
        Ok(s) if s.is_empty() => return absent(),
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return absent(),
        Err(e) => return not_measured(format!("docs/specifications could not be read: {e}")),
    };
    
    let parsed = parse_specs(&specs);
    let exempt = parsed.render_exempt();
    let epics_named = parsed.epics_named();
    
    let mut findings = parsed.findings.clone();
    let snapshot_clause;
    
    if epics_named.is_empty() {
        snapshot_clause = "not needed — no active spec names an epic".to_string();
    } else {
        let roadmap_path = project_path.join("docs/roadmaps/roadmap.yaml");
        let repo_hint = if roadmap_path.exists() {
            crate::services::roadmap_service::RoadmapService::new(&roadmap_path).load().ok().and_then(|r| r.github_repo)
        } else {
            None
        };
        
        match github_inputs(project_path, comply_config, "cb-2110", github_snapshot, repo_hint, true, epics_named.len()) {
            Ok(gh_inputs) => {
                snapshot_clause = format!("{} taken {}", gh_inputs.source, gh_inputs.taken_at);
                let mut epic_findings = bind_epics(&parsed, &gh_inputs.snapshot);
                findings.append(&mut epic_findings);
            }
            Err(mut early) => {
                early.name = name;
                early.message.push_str(&format!("; exempt from the epic leg: {exempt}"));
                return early;
            }
        }
    }
    
    findings.sort_by(|a, b| a.spec().cmp(b.spec()));
    
    let mut linked = 0;
    for (path, _fm) in parsed.active() {
        if !findings.iter().any(|f| f.spec() == path) {
            linked += 1;
        }
    }
    
    let specs_count = specs.len();
    let active_count = parsed.active().count();
    
    if findings.is_empty() {
        let pass = format!("{specs_count} spec(s) under docs/specifications; {active_count} active each name an open epic (labelled epic, ≥1 sub-issue, goal-mode.md §4.3): linked {linked}; exempt from the epic leg: {exempt}; snapshot: {snapshot_clause}");
        roadmap_verdict(name, true, pass, String::new())
    } else {
        let is_unmeasured = findings.iter().any(|f| f.is_unmeasured());
        
        let classes = findings.iter().map(|f| f.class());
        let classes_str = class_counts(classes);
        let first_eight_str = first_eight(findings.iter().map(|f| f.render()), findings.len());
        
        let mut fail = format!("{} finding(s) — {classes_str}: {first_eight_str}; an epic is an open issue labelled epic whose sub-issues are the spec's tickets (goal-mode.md §4.3): create it on GitHub, add the tickets' issues as sub-issues, write epic: N in the spec's front-matter; status: historical or superseded exempts a spec from the epic leg, never the parse leg, and every exempt spec is named here; exempt from the epic leg: {exempt}; snapshot: {snapshot_clause}", findings.len());
        
        if is_unmeasured {
            fail = format!("not_measured: {fail}");
        }
        
        roadmap_verdict(name, false, String::new(), fail)
    }
}