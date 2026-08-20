// CB-2101: threshold coherence.
// Included from check.rs — do NOT add `use` imports or `#!` attributes here.
//
// Engine + falsification suite: src/services/metrics_ratchet/
// Contract: contracts/comply-threshold-coherence-v1.yaml
//
// CB-2102 asks whether a metric got worse. CB-2101 asks the prior question
// nobody was asking: does the number written down mean anything at all?
//
// Two ways for it not to. A threshold can be VIOLATED — breached at HEAD while
// the build is green, which is worse than having no threshold, because it reads
// as enforcement and enforces nothing. Or it can be VACUOUS — so far from the
// measurement that no movement the ratchet tolerates could ever reach it, which
// is a number that will be green on the day the project is abandoned.
//
// `.pmat-metrics.toml:45` is this repository's own VIOLATED case:
// `max_unwrap_calls = 100`, annotated `Current: 570`, in a tree that measures
// 20,390 by the predicate `.pmat-ratchet.toml` pins. Fourteen of the other
// sixteen thresholds in the same file are read by nothing whatsoever.

/// Cap on how many diagnostic detail lines the message carries. The
/// classification roll-up is NOT capped: "every threshold is classified" is the
/// rule's definition of done, and a truncated roll-up would not be one.
const COHERENCE_MAX_DETAIL_LINES: usize = 6;

/// CB-2101: Threshold Coherence.
///
/// Skips only when the project has never adopted the rule — no
/// `.pmat-ratchet.toml` on disk and none in git history — which is the same
/// door CB-2102 uses, so a project cannot be half-enrolled. Once a ratchet file
/// has been committed, every scalar in a declared threshold section of
/// `.pmat-metrics.toml` must carry a binding: an unbound threshold is a number
/// enforced by nothing, and reporting it as anything but a failure would make
/// this check the very artefact it audits.
///
/// Deleting the ratchet file is a Fail, not a Skip. Every other unmeasurable
/// outcome is a failure too: an unreadable or unparsable config on either side,
/// a gate whose metric produced no measurement, or a section of
/// `.pmat-metrics.toml` that appears in neither the threshold list nor the
/// declared-non-threshold list.
#[provable_contracts_macros::contract("comply-threshold-coherence-v1.yaml", equation = "audit")]
pub(crate) fn check_threshold_coherence(project_path: &Path) -> ComplianceCheck {
    use crate::services::metrics_ratchet::{self, config::Outcome, RatchetStatus};
    let name = "CB-2101: Threshold Coherence";

    match metrics_ratchet::status(project_path) {
        RatchetStatus::Absent => {
            return ComplianceCheck {
                name: name.into(),
                status: CheckStatus::Skip,
                message: format!(
                    "no {} — this project declares no threshold bindings",
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
                    "{} was committed and is now gone — deleting the file that binds every \
                     threshold is not a way of passing the audit",
                    metrics_ratchet::config::RATCHET_FILE
                ),
                severity: Severity::Error,
            }
        }
        RatchetStatus::Present => {}
    }

    let report = match metrics_ratchet::run_coherence(project_path) {
        Ok(r) => r,
        Err(e) => {
            return ComplianceCheck {
                name: name.into(),
                status: CheckStatus::Fail,
                message: format!(
                    "the threshold audit could not run ({e}) — an audit that did not happen \
                     is not an audit that passed"
                ),
                severity: Severity::Error,
            }
        }
    };

    // An audit with nothing in it cannot fail. That is the shape CB-2102 was
    // found carrying (`evaluate_ratchet` folded Ok over an empty map) and it is
    // refused here rather than inherited.
    if report.thresholds.is_empty() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Fail,
            message: format!(
                "{} classified no thresholds — an empty audit passes forever and is not a gate",
                metrics_ratchet::config::METRICS_FILE
            ),
            severity: Severity::Error,
        };
    }

    let (status, severity) = match report.outcome {
        Outcome::Ok => (CheckStatus::Pass, Severity::Info),
        Outcome::Warn => (CheckStatus::Warn, Severity::Warning),
        Outcome::Fail => (CheckStatus::Fail, Severity::Error),
    };
    ComplianceCheck {
        name: name.into(),
        status,
        message: coherence_message(&report),
        severity,
    }
}

/// The roll-up every run carries: one `key=CLASS` for every classified
/// threshold, in key order, uncapped.
fn coherence_roll_up(
    report: &crate::services::metrics_ratchet::config::CoherenceReport,
) -> String {
    report
        .thresholds
        .iter()
        .map(|t| format!("{}={}", t.key, t.classification.as_str()))
        .collect::<Vec<_>>()
        .join(", ")
}

fn coherence_message(
    report: &crate::services::metrics_ratchet::config::CoherenceReport,
) -> String {
    use crate::services::metrics_ratchet::config::Outcome;
    let counts = |c: crate::services::metrics_ratchet::kernel::Classification| {
        report
            .thresholds
            .iter()
            .filter(|t| t.classification == c)
            .count()
    };
    use crate::services::metrics_ratchet::kernel::Classification;
    let mut parts = vec![format!(
        "{} threshold(s): {} FIRING, {} VIOLATED, {} VACUOUS",
        report.thresholds.len(),
        counts(Classification::Firing),
        counts(Classification::Violated),
        counts(Classification::Vacuous),
    )];

    let mut lines: Vec<String> = report
        .undeclared_sections
        .iter()
        .map(|s| format!("section [{s}] is in neither threshold_sections nor non_threshold_sections"))
        .collect();
    lines.extend(
        report
            .thresholds
            .iter()
            .filter(|t| t.outcome == Outcome::Fail)
            .map(|t| format!("{}: {}", t.key, t.detail)),
    );
    parts.extend(lines.iter().take(COHERENCE_MAX_DETAIL_LINES).cloned());
    if lines.len() > COHERENCE_MAX_DETAIL_LINES {
        parts.push(format!("… and {} more", lines.len() - COHERENCE_MAX_DETAIL_LINES));
    }
    parts.push(coherence_roll_up(report));
    parts.join(" | ")
}
