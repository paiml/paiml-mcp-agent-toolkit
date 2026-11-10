// GS-03, GS-04, GS-05, GS-12, GS-14 (#1673): the wave-2 detectors of `services::gate_smells`.
// Included from check.rs — no `use` imports here.

const GS_WAVE2: &[(&str, &str)] = &[
    ("GS-03", "Live External Read In PR Job"),
    ("GS-04", "Gate Self-Test On Product Path"),
    ("GS-05", "Roll-Up On Heavy Runner"),
    ("GS-12", "Unreachable Shared Workflow Sha"),
    ("GS-14", "Repo-Global Check On PR Path"),
];

/// One hit line per finding of `id`: `file:job message`.
fn gs_wave2_hits(report: &crate::services::gate_smells::Report, id: &str) -> Vec<String> {
    report
        .findings
        .iter()
        .filter(|f| f.id == id)
        .map(|f| format!("{}:{} {}", f.file, f.job, f.message))
        .collect()
}

/// Skip, never Pass, when a detector could not measure (L23).
fn gs_wave2_skip(id: &str, title: &str, why: String) -> ComplianceCheck {
    ComplianceCheck {
        name: format!("{id}: {title}"),
        status: CheckStatus::Skip,
        message: why,
        severity: Severity::Info,
    }
}

/// GS-03, GS-04, GS-05, GS-12 and GS-14 for one repository, in id order.
/// GS-12 needs git reachability of a shared-workflow sha; no resolver is wired
/// here, so a repo that pins one reports it as not measured, never as a pass.
fn gs_wave2_checks(project_path: &Path) -> Vec<ComplianceCheck> {
    let dir = project_path.join(".github/workflows");
    let scanned = crate::services::gate_smells::scan_workflows(&dir, &|_, _| None);
    GS_WAVE2
        .iter()
        .map(|(id, title)| match &scanned {
            Err(e) => gs_wave2_skip(id, title, format!("decline: no readable workflows ({e})")),
            Ok(report) => {
                let hits = gs_wave2_hits(report, id);
                match (hits.is_empty(), report.not_measured.first()) {
                    (true, Some(why)) => gs_wave2_skip(id, title, format!("not_measured: {why}")),
                    _ => gs_verdict(id, title, &hits),
                }
            }
        })
        .collect()
}
