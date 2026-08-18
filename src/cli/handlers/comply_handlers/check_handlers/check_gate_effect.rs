// CB-1411: comply gate-effect verification (EV-1).
// Included from check.rs — do NOT add `use` imports or `#!` attributes here.
//
// Engine + falsification suite: src/services/gate_effect/
// Contract: contracts/comply-gate-effect-v1.yaml
//
// The rule every other "enforced in CI" claim rests on. For each comply rule
// declared severity=error, resolve a path from a REQUIRED STATUS CHECK CONTEXT
// STRING to an invocation of that rule whose non-zero exit can still fail the
// job. Anything short of that — a phantom context, an unreadable reusable
// workflow, `continue-on-error`, `|| true`, an exit code captured and never
// compared — is not enforcement.
//
// The id is CB-1411, not CB-1400 as the EV-1 backlog entry says: CB-1400 has
// shipped since PMAT-032 as "Agent Contract Existence" and reusing the id would
// have deleted a live rule. See the module docs in src/services/gate_effect.

/// Cap on how many diagnostic lines the message carries. The full detail is in
/// the report the engine returns; the check message is a summary.
const GATE_EFFECT_MAX_LINES: usize = 6;

/// CB-1411: Comply Gate Effect.
///
/// Skips only when the project has no `.github/workflows` at all — there is
/// then no GitHub Actions gate to verify. Every other unmeasurable outcome is a
/// failure: an unresolvable required-check list, an unparsable workflow, zero
/// jobs, or a required context that resolves into a workflow this repository
/// cannot read.
#[provable_contracts_macros::contract("comply-gate-effect-v1.yaml", equation = "gate_effect")]
pub(crate) fn check_comply_gate_effect(
    project_path: &Path,
    comply_config: &crate::models::comply_config::ComplyConfig,
) -> ComplianceCheck {
    let name = "CB-1411: Comply Gate Effect";
    if !project_path.join(".github").join("workflows").is_dir() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No .github/workflows/ — no GitHub Actions gate to verify".into(),
            severity: Severity::Info,
        };
    }

    let report = crate::services::gate_effect::analyze(project_path, comply_config);
    if report.passed() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: gate_effect_pass_message(&report),
            severity: Severity::Info,
        };
    }
    ComplianceCheck {
        name: name.into(),
        status: CheckStatus::Fail,
        message: gate_effect_fail_message(&report),
        severity: Severity::Error,
    }
}

fn gate_effect_pass_message(report: &crate::services::gate_effect::GateEffectReport) -> String {
    let sites: Vec<String> = report
        .enforcing()
        .map(|i| format!("{}:{} ({})", i.workflow.display(), i.job_id, i.via))
        .collect();
    format!(
        "all {} severity=error rule(s) reachable from required check(s) [{}] via {}",
        report.rules.len(),
        report.required_contexts.join(", "),
        sites.join(", ")
    )
}

fn gate_effect_fail_message(report: &crate::services::gate_effect::GateEffectReport) -> String {
    let mut parts = vec![format!(
        "{} severity=error rule(s) unreachable from required check(s) [{}]",
        report.unreachable_rules.len(),
        if report.required_contexts.is_empty() {
            "unresolved".to_string()
        } else {
            report.required_contexts.join(", ")
        }
    )];
    parts.extend(report.holes.iter().take(GATE_EFFECT_MAX_LINES).cloned());
    for inv in report.neutered().take(GATE_EFFECT_MAX_LINES) {
        parts.push(format!(
            "{}:{} step `{}` invokes comply but {}",
            inv.workflow.display(),
            inv.job_id,
            inv.step,
            inv.suppressions.join("; ")
        ));
    }
    if !report.unreachable_rules.is_empty() {
        parts.push(format!("unreachable: {}", report.unreachable_rules.join(", ")));
    }
    parts.join(" | ")
}
