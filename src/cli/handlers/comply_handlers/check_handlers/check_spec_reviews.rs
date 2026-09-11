// Included from check.rs — do NOT add `use` imports or `#!` attributes here.

/// CB-2111: Spec Reviews (goal-mode.md §6, §7 row CB-2111; invariant E.1 —
/// every `active` spec has a review artifact whose hash matches and whose
/// roles all PASS).
///
/// The rule reads a FILE per active spec, `docs/audits/spec-<slug>-review.json`,
/// and never invokes a model (§6.1), so it runs offline, forks included. The
/// judgement is `services::spec_review::judge`; the specs are `list_specs`, and
/// their front-matter is `services::spec_epic`'s parse leg, so the vendor roles
/// a spec's `vendors:` adds are the ones CB-2110 reads. A spec whose
/// front-matter does not parse cannot say which roles its review needs:
/// UNJUDGEABLE, never skipped. `historical` and `superseded` specs are exempt,
/// and every verdict names them. A `docs/specifications` that was committed
/// and is now gone is `not_measured`, never Skip (§12).
///
/// What it buys, exactly (§6.2): a skipped review becomes an auditable lie
/// somebody wrote down instead of an omission nobody can see. It is not
/// evidence that the review happened.
pub(crate) fn check_spec_reviews(project_path: &Path) -> ComplianceCheck {
    use crate::models::comply_config::CheckSeverity;
    use crate::services::spec_epic::{parse_specs, SpecStatus, SPECS_DIR};
    use crate::services::spec_review::{artifact_path, judge, ReviewFinding};

    let name = "CB-2111: Spec Reviews".to_string();

    let skip = |message: String| ComplianceCheck {
        name: name.clone(),
        status: CheckStatus::Skip,
        severity: CheckSeverity::Info.into(),
        message,
    };

    let not_measured = |message: String| {
        ComplianceCheck {
        name: name.clone(),
        status: CheckStatus::Fail,
        severity: CheckSeverity::Error.into(),
        message: format!("not_measured: {message} — an input the rule expected and could not read is a failure, not a pass (goal-mode.md doctrine 2)"),
    }
    };

    // The line CB-2110 draws: never committed ⇒ absent by design (Skip);
    // committed and now gone ⇒ deleting a gate's input is not a way of
    // passing it. An unreadable history is treated as "never".
    let absent = || {
        match crate::services::metrics_ratchet::history::was_ever_committed(project_path, SPECS_DIR) {
        Ok(true) => not_measured("docs/specifications was committed and is now gone — deleting a gate's input is not a way of passing it".to_string()),
        Ok(false) | Err(_) => skip("no docs/specifications — this project keeps no specifications, so there is no spec to review".to_string()),
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
    let mut findings: Vec<ReviewFinding> = Vec::new();
    let mut active = 0usize;
    let mut reviewed = 0usize;

    // Two active specs whose paths flatten to one slug share an artifact
    // path, and a review can name only one of them: name the collision
    // rather than judge a review against the wrong spec.
    let mut sharing: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    for (path, front) in &parsed.specs {
        if front.status == SpecStatus::Active {
            sharing
                .entry(artifact_path(path))
                .or_default()
                .push(path.clone());
        }
    }

    // One pass in path order (list_specs sorts), so the findings need no sort.
    for input in &specs {
        let Some((path, front)) = parsed.specs.iter().find(|(p, _)| *p == input.path) else {
            let why = parsed
                .findings
                .iter()
                .find(|f| f.spec() == input.path)
                .map_or_else(|| "no front-matter".to_string(), |f| f.render());
            findings.push(ReviewFinding::Unjudgeable {
                spec: input.path.clone(),
                why,
            });
            continue;
        };
        if front.status != SpecStatus::Active {
            continue;
        }
        active += 1;
        let artifact = artifact_path(path);
        if let Some(all) = sharing.get(&artifact).filter(|all| all.len() > 1) {
            let with = all
                .iter()
                .filter(|p| *p != path)
                .cloned()
                .collect::<Vec<_>>()
                .join(", ");
            findings.push(ReviewFinding::SlugCollision {
                spec: path.clone(),
                artifact,
                with,
            });
            continue;
        }
        let text = match std::fs::read_to_string(project_path.join(&artifact)) {
            Ok(text) => Some(text),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => {
                findings.push(ReviewFinding::BadReview {
                    spec: path.clone(),
                    artifact,
                    reason: format!("cannot be read: {e}"),
                });
                continue;
            }
        };
        let mut found = judge(path, &input.text, &front.vendors, text.as_deref());
        if found.is_empty() {
            reviewed += 1;
        }
        findings.append(&mut found);
    }

    if findings.is_empty() {
        let pass = format!("{} spec(s) under docs/specifications; {active} active each carry a current review (docs/audits/spec-<slug>-review.json: the spec's sha256 now, a plan sha256, one PASS lane per required role, partial false — goal-mode.md §6): reviewed {reviewed}; exempt: {exempt}", specs.len());
        roadmap_verdict(name, true, pass, String::new())
    } else {
        let classes = class_counts(findings.iter().map(ReviewFinding::class));
        let shown = first_eight(findings.iter().map(ReviewFinding::render), findings.len());
        let fail = format!("{} finding(s) — {classes}: {shown}; a review is docs/audits/spec-<slug>-review.json (goal-mode.md §6.1): produce it (a quorum, or by hand: it is JSON), then `pmat spec review --record <json>` validates and stages it; editing a spec stales its review; exempt: {exempt}", findings.len());
        roadmap_verdict(name, false, String::new(), fail)
    }
}
