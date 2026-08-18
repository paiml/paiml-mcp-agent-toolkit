// CB-2100: comply gate-effect verification.
// Included from check.rs — do NOT add `use` imports or `#!` attributes here.
//
// Engine + falsification suite: src/services/gate_effect/
// Contract: contracts/comply-gate-effect-v1.yaml
//
// The rule every other "enforced in CI" claim rests on. For each comply rule
// declared severity=error, resolve a path from the UNION of the repository's
// required status check CONTEXT STRINGS to an invocation of that rule whose
// non-zero exit can still fail the job. Anything short of that — a phantom
// context, an unreadable reusable workflow, `continue-on-error`, `|| true`, an
// exit code captured and never compared, a wrapper that prints a failure
// verdict and exits 0, a job that can never succeed, a compile mistaken for an
// execution — is not enforcement.
//
// The id is CB-2100. Every id the source backlog proposed (CB-1400, CB-1403,
// CB-1404, CB-1211, CB-1300, CB-1302, CB-1305) collides with a live rule, and
// CB-21xx was audited free across src/, contracts/, docs/, .github/ and the
// whole git history.

/// Cap on how many diagnostic lines the message carries. The full detail is in
/// the report the engine returns; the check message is a summary.
const GATE_EFFECT_MAX_LINES: usize = 6;

/// CB-2100: Comply Gate Effect.
///
/// Skips only when the project has no `.github/workflows` at all — there is
/// then no GitHub Actions gate to verify. Every other unmeasurable outcome is a
/// failure: an unresolvable required-check list, an unparsable workflow, zero
/// jobs, a required context that resolves into a workflow this repository
/// cannot read, or an enforcement ledger that does not match what the engine
/// computes.
#[provable_contracts_macros::contract("comply-gate-effect-v1.yaml", equation = "gate_effect")]
pub(crate) fn check_comply_gate_effect(
    project_path: &Path,
    comply_config: &crate::models::comply_config::ComplyConfig,
) -> ComplianceCheck {
    let name = "CB-2100: Comply Gate Effect";
    if !project_path.join(".github").join("workflows").is_dir() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No .github/workflows/ — no GitHub Actions gate to verify".into(),
            severity: Severity::Info,
        };
    }

    let report = crate::services::gate_effect::analyze(project_path, comply_config);
    let ledger = gate_effect_ledger_drift(project_path, &report, comply_config);
    if report.passed() && ledger.is_none() {
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
        message: gate_effect_fail_message(&report, ledger),
        severity: Severity::Error,
    }
}

/// `Some(reason)` when the committed enforcement ledger is missing or stale.
///
/// Only repositories that *define* CB rules owe a ledger; a fleet repo that
/// merely runs `pmat comply check` has no roster of its own to account for. A
/// repository that does define them and cannot produce a ledger fails: an
/// unwritable status is exactly the "we could not measure it" case this rule
/// refuses to let pass.
fn gate_effect_ledger_drift(
    project_path: &Path,
    report: &crate::services::gate_effect::GateEffectReport,
    comply_config: &crate::models::comply_config::ComplyConfig,
) -> Option<String> {
    use crate::services::gate_effect::{ledger, roster};
    if !roster::defines_rules(project_path) {
        return None;
    }
    let expected = match ledger::render(project_path, report, comply_config) {
        Ok(text) => text,
        Err(e) => return Some(e),
    };
    match ledger::committed(project_path) {
        None => Some(format!(
            "{} is missing — CB-2100 generates it; run `pmat comply ledger --write`",
            ledger::LEDGER_PATH
        )),
        Some(found) if found != expected => Some(format!(
            "{} is stale (regenerate with `pmat comply ledger --write`)",
            ledger::LEDGER_PATH
        )),
        Some(_) => None,
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

fn gate_effect_fail_message(
    report: &crate::services::gate_effect::GateEffectReport,
    ledger: Option<String>,
) -> String {
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
    for (context, carries) in report.context_carries() {
        if !carries {
            parts.push(format!(
                "required check `{context}` reaches no rule invocation, so it gates nothing in \
                 the CB roster"
            ));
        }
    }
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
    parts.extend(ledger);
    parts.join(" | ")
}
