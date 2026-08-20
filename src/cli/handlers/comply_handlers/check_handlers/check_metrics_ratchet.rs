// CB-2102: ratchet baselines.
// Included from check.rs — do NOT add `use` imports or `#!` attributes here.
//
// Engine + falsification suite: src/services/metrics_ratchet/
// Contract: contracts/comply-ratchet-v1.yaml
//
// A threshold is a number somebody guessed once. `.pmat-metrics.toml:45` is
// this repository's own worked example: `max_unwrap_calls = 100`, annotated
// `Current: 570`, in a tree that measures 20,390. Nothing enforced the limit,
// nothing re-measured the comment, and the build was green throughout.
//
// A ratchet needs no such judgement. `.pmat-ratchet.toml` records what each
// metric WAS, together with the exact command that reproduces it; the gate
// asserts only that it has not got worse; a scheduled job lowers a baseline
// whenever the measurement drops; and raising one needs a written
// justification.
//
// Until this check existed, src/services/metrics_ratchet/ was 1,836 lines of
// kernel, config and falsification tests with NO caller anywhere in the tree,
// no `.pmat-ratchet.toml` to read, and two contract files named in its doc
// comments that did not exist. It was the defect class CB-2100 exists to find,
// inside the module written to find it.

/// Cap on how many diagnostic lines the message carries.
const RATCHET_MAX_LINES: usize = 6;

/// CB-2102: Ratchet Baselines.
///
/// Skips only when the project has never declared any baselines — no
/// `.pmat-ratchet.toml` on disk and none in git history. Deleting a committed
/// one is a Fail: removing a gate's input is not a way of passing it. Every
/// other unmeasurable outcome is a failure too — an unreadable or unparsable
/// config, an unknown schema version, a metric whose command did not run, a
/// measurement that is not a number, an empty metric set, or an unreadable
/// previous version of the file (against which a raised baseline would be
/// invisible).
#[provable_contracts_macros::contract("comply-ratchet-v1.yaml", equation = "ratchet")]
pub(crate) fn check_metrics_ratchet(project_path: &Path) -> ComplianceCheck {
    use crate::services::metrics_ratchet::{self, config::Outcome, RatchetStatus};
    let name = "CB-2102: Ratchet Baselines";

    match metrics_ratchet::status(project_path) {
        RatchetStatus::Absent => {
            return ComplianceCheck {
                name: name.into(),
                status: CheckStatus::Skip,
                message: format!(
                    "no {} — this project declares no ratcheted baselines",
                    metrics_ratchet::config::RATCHET_FILE
                ),
                severity: Severity::Info,
            }
        }
        RatchetStatus::Deleted => {
            return ComplianceCheck {
                name: name.into(),
                status: CheckStatus::Fail,
                message: format!(
                    "{} was committed and is now gone — deleting a gate's input is not a way \
                     of passing it",
                    metrics_ratchet::config::RATCHET_FILE
                ),
                severity: Severity::Error,
            }
        }
        RatchetStatus::Present => {}
    }

    let report = match metrics_ratchet::run(project_path) {
        Ok(r) => r,
        Err(e) => {
            return ComplianceCheck {
                name: name.into(),
                status: CheckStatus::Fail,
                message: format!(
                    "{} could not be used ({e}) — an unusable ratchet is a failure, not a pass",
                    metrics_ratchet::config::RATCHET_FILE
                ),
                severity: Severity::Error,
            }
        }
    };

    if report.outcome == Outcome::Ok {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: ratchet_pass_message(&report),
            severity: Severity::Info,
        };
    }
    ComplianceCheck {
        name: name.into(),
        status: CheckStatus::Fail,
        message: ratchet_fail_message(&report),
        severity: Severity::Error,
    }
}

fn ratchet_pass_message(
    report: &crate::services::metrics_ratchet::config::RatchetReport,
) -> String {
    let held: Vec<String> = report
        .metrics
        .iter()
        .map(|m| {
            format!(
                "{}={}/{}",
                m.metric,
                m.observed
                    .map_or_else(|| "?".to_string(), |v| v.to_string()),
                m.baseline
            )
        })
        .collect();
    format!(
        "all {} baseline(s) held (observed/baseline): {}",
        report.metrics.len(),
        held.join(", ")
    )
}

fn ratchet_fail_message(
    report: &crate::services::metrics_ratchet::config::RatchetReport,
) -> String {
    use crate::services::metrics_ratchet::config::Outcome;
    let failed = report
        .metrics
        .iter()
        .filter(|m| m.outcome == Outcome::Fail)
        .count();
    let mut lines: Vec<String> = report.holes.clone();
    lines.extend(report.unjustified_raises.iter().map(|r| {
        format!("baseline raised without a justification — {r}")
    }));
    lines.extend(
        report
            .metrics
            .iter()
            .filter(|m| m.outcome == Outcome::Fail)
            .map(|m| format!("{}: {}", m.metric, m.detail)),
    );

    let mut parts = vec![format!(
        "{failed} of {} ratcheted metric(s) failed",
        report.metrics.len()
    )];
    parts.extend(lines.iter().take(RATCHET_MAX_LINES).cloned());
    if lines.len() > RATCHET_MAX_LINES {
        parts.push(format!("… and {} more", lines.len() - RATCHET_MAX_LINES));
    }
    parts.join(" | ")
}
