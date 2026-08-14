// THE pass/fail rule for every quality-gate surface, in ONE place.
//
// This rule had four implementations. The MCP `quality_gate` entry points
// filtered advisory findings out of the verdict while both CLI entry points ran
// `results.passed = violations.is_empty()`, so ONE `// TODO` in ONE file came
// back `passed:true, blocking_violations:0` over MCP and `passed:false` from
// `pmat quality-gate` — byte-identical finding, byte-identical
// `severity:"info"`, opposite verdicts. Every surface now calls the functions
// below; there is nowhere left for the two answers to diverge.

/// The one severity that is advisory: reported in `violations`, never
/// verdict-bearing.
pub const ADVISORY_SEVERITY: &str = "info";

/// Does a finding of this severity decide the pass/fail verdict?
///
/// `error` and `warning` are actionable and fail the gate; `info` is advisory.
/// An unrecognised severity is verdict-bearing — fail closed, because an
/// unclassified finding must never be silently demoted to advice.
#[must_use]
pub fn severity_is_verdict_bearing(severity: &str) -> bool {
    severity != ADVISORY_SEVERITY
}

/// Does this finding decide the pass/fail verdict?
#[must_use]
pub fn is_verdict_bearing(violation: &QualityViolation) -> bool {
    severity_is_verdict_bearing(&violation.severity)
}

/// The same rule for surfaces that carry findings as JSON (the MCP
/// `quality_gate` tool), so the two encodings cannot drift apart.
///
/// A finding with no `severity` field at all is verdict-bearing: fail closed.
#[must_use]
pub fn json_is_verdict_bearing(violation: &serde_json::Value) -> bool {
    violation
        .get("severity")
        .and_then(serde_json::Value::as_str)
        .is_none_or(severity_is_verdict_bearing)
}

/// How many of these findings decide the verdict.
///
/// Stated rather than inferred: `violations` legitimately contains rows that did
/// NOT decide the verdict, so `passed:true` beside a non-empty list is not a
/// contradiction as long as this count is reported next to it.
#[must_use]
pub fn blocking_violation_count(violations: &[QualityViolation]) -> usize {
    violations.iter().filter(|v| is_verdict_bearing(v)).count()
}

/// The verdict for a set of findings: pass iff nothing verdict-bearing was found.
///
/// Callers that ALSO have a measurement to defend (a TDG score, a parse result)
/// must AND their own condition in — this function answers only "did the
/// findings block?", never "was anything measured?".
#[must_use]
pub fn violations_pass(violations: &[QualityViolation]) -> bool {
    blocking_violation_count(violations) == 0
}
