#![cfg_attr(coverage_nightly, coverage(off))]
//! Types, enums, and structs for lint hotspot analysis

use crate::cli::LintHotspotOutputFormat;
use serde::{Deserialize, Serialize, Serializer};
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

/// Serialize a `HashMap` keyed by path in sorted key order.
///
/// DETERMINISM: `serde_json` emits `HashMap` entries in hash order, so two runs
/// over the same tree produced the same data under a different JSON key order.
/// Collecting into a `BTreeMap` first makes the rendered map byte-identical run
/// to run.
fn serialize_paths_sorted<S>(
    map: &HashMap<PathBuf, FileSummary>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let ordered: BTreeMap<&PathBuf, &FileSummary> = map.iter().collect();
    serde::Serialize::serialize(&ordered, serializer)
}

/// Default `--max-density`: the quality gate fails above 5 violations per 100
/// lines of code.
///
/// #699: this used to be spelled `5.0` in two unconnected places (the clap
/// `default_value_t` and `contracts::contract_definitions::default_max_density`)
/// while `check_quality_gates` compares against `violations / sloc`. 5 lint
/// violations per LINE is unreachable, so the documented gate never fired:
/// a fixture hotspot measuring `defect_density: 2.0` (200 violations per 100
/// lines) reported `"passed": true` and exited 0. Both interfaces now read
/// this one constant so they cannot drift apart again.
pub const DEFAULT_MAX_DENSITY: f64 = 0.05;

/// Parameters for lint hotspot analysis
pub struct LintHotspotParams {
    pub project_path: PathBuf,
    pub file: Option<PathBuf>,
    pub format: LintHotspotOutputFormat,
    pub max_density: f64,
    pub min_confidence: f64,
    pub enforce: bool,
    pub dry_run: bool,
    pub enforcement_metadata: bool,
    pub output: Option<PathBuf>,
    pub perf: bool,
    pub clippy_flags: String,
    pub top_files: usize,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
}

/// Lint hotspot analysis result
#[derive(Debug, Serialize, Deserialize)]
pub struct LintHotspotResult {
    pub hotspot: LintHotspot,
    pub all_violations: Vec<ViolationDetail>,
    #[serde(serialize_with = "serialize_paths_sorted")]
    pub summary_by_file: HashMap<PathBuf, FileSummary>,
    pub total_project_violations: usize,
    pub enforcement: Option<EnforcementMetadata>,
    pub refactor_chain: Option<RefactorChain>,
    pub quality_gate: QualityGateStatus,
}

/// The identified hotspot file
#[derive(Debug, Serialize, Deserialize)]
pub struct LintHotspot {
    pub file: PathBuf,
    pub defect_density: f64,
    pub total_violations: usize,
    pub sloc: usize,
    pub severity_distribution: SeverityDistribution,
    pub top_lints: Vec<(String, usize)>,
    pub detailed_violations: Vec<ViolationDetail>,
}

/// How to read `defect_density`, emitted beside the hotspot in every JSON
/// format.
///
/// #924 (residual): `defect_density` is `total_violations / sloc`, and it
/// legitimately exceeds 1.0 — several findings land on one counted line
/// (`clippy::doc_markdown` fires once per token, and a single `if x.len() == 0 {
/// return true } else { return false }` line draws `len_zero`, `needless_bool`,
/// `needless_return` and `redundant_else` at once). A reader with no way to see
/// that reasonably concludes `> 1.0` is a bug and stops trusting the number.
///
/// Rather than assert a bound the data violates, the report SHOWS the
/// arithmetic: how many findings there are, how many distinct lines carry them,
/// and the most any single line carries. `violations_per_violating_line` above
/// 1.0 is the direct explanation of a density above 1.0.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DensityBasis {
    /// The formula, spelled out, so no consumer has to infer the unit.
    pub formula: &'static str,
    /// Numerator: the same figure as `hotspot.total_violations`.
    pub total_violations: usize,
    /// Denominator: the same figure as `hotspot.sloc`.
    pub sloc: usize,
    /// Findings that carry a source location; the two line figures below are
    /// computed over these. Equal to `total_violations` except for findings
    /// counted by severity that carry no primary span.
    pub located_violations: usize,
    /// Distinct lines that carry at least one located finding.
    pub distinct_violating_lines: usize,
    /// The most findings any one line carries.
    pub max_violations_on_one_line: usize,
    /// `located_violations / distinct_violating_lines`. Above 1.0 exactly when
    /// findings share lines — which is why the density may exceed 1.0.
    pub violations_per_violating_line: f64,
    /// True when `defect_density > 1.0`, with `explanation` saying why that is
    /// not a defect.
    pub density_exceeds_one: bool,
    /// How the hotspot was CHOSEN, spelled out. It is not the density, and a
    /// report that showed only the density would look like it had ranked wrong.
    pub ranking_formula: &'static str,
    /// The value of `ranking_formula` for this file.
    pub ranking_score: f64,
    /// One sentence a human can read without the field docs.
    pub explanation: String,
}

/// Detailed violation information for rewriting
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ViolationDetail {
    pub file: PathBuf,
    pub line: u32,
    pub column: u32,
    pub end_line: u32,
    pub end_column: u32,
    pub lint_name: String,
    pub message: String,
    pub severity: String,
    pub suggestion: Option<String>,
    pub machine_applicable: bool,
}

/// File-level summary
#[derive(Debug, Serialize, Deserialize)]
pub struct FileSummary {
    pub total_violations: usize,
    pub errors: usize,
    pub warnings: usize,
    pub sloc: usize,
    pub defect_density: f64,
}

/// Severity distribution of violations
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SeverityDistribution {
    pub error: usize,
    pub warning: usize,
    pub suggestion: usize,
    pub note: usize,
}

/// Enforcement metadata for quality gates
#[derive(Debug, Serialize, Deserialize)]
pub struct EnforcementMetadata {
    pub enforcement_score: f64,
    pub requires_enforcement: bool,
    pub estimated_fix_time: u32,
    pub automation_confidence: f64,
    pub enforcement_priority: u8,
}

/// Refactor chain for automated fixes
#[derive(Debug, Serialize, Deserialize)]
pub struct RefactorChain {
    pub id: String,
    pub estimated_reduction: usize,
    pub automation_confidence: f64,
    pub steps: Vec<RefactorStep>,
}

/// Individual refactor step
#[derive(Debug, Serialize, Deserialize)]
pub struct RefactorStep {
    pub id: String,
    pub lint: String,
    pub confidence: f64,
    pub impact: usize,
    pub description: String,
}

/// Quality gate status
#[derive(Debug, Serialize, Deserialize)]
pub struct QualityGateStatus {
    pub passed: bool,
    pub violations: Vec<QualityViolation>,
    pub blocking: bool,
}

/// Individual quality violation
#[derive(Debug, Serialize, Deserialize)]
pub struct QualityViolation {
    pub rule: String,
    pub threshold: f64,
    pub actual: f64,
    pub severity: String,
}

/// File metrics for analysis
#[derive(Debug, Default)]
pub(crate) struct FileMetrics {
    pub(crate) violations: HashMap<String, usize>,
    pub(crate) severity_counts: SeverityDistribution,
    pub(crate) sloc: usize,
    pub(crate) detailed_violations: Vec<ViolationDetail>,
}

/// Clippy message structure
#[derive(Debug, Deserialize)]
pub(crate) struct ClippyMessage {
    pub(crate) reason: Option<String>,
    pub(crate) message: Option<DiagnosticMessage>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DiagnosticMessage {
    pub(crate) level: String,
    pub(crate) message: String,
    pub(crate) code: Option<DiagnosticCode>,
    pub(crate) spans: Vec<DiagnosticSpan>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DiagnosticCode {
    pub(crate) code: String,
}

// #679 ROOT CAUSE (silent under-report): this struct used to carry
//   #[serde(default, rename = "text")] _text: Vec<DiagnosticText>
// where `DiagnosticText` declared fields literally named `_text`,
// `_highlight_start` and `_highlight_end` — names cargo's JSON never emits
// (they are `text`, `highlight_start`, `highlight_end`). Every span that
// carried source text therefore failed to deserialize, which failed the whole
// `ClippyMessage`, which the parsers dropped with `Ok(None)`.
//
// Observed wrong value: `analyze lint-hotspot --file src/lib.rs` on a fixture
// with 20 real clippy findings reported 12 — exactly the 12 that had an EMPTY
// `text` array (clippy::cargo_common_metadata). All 8 code lints (ptr_arg,
// needless_bool, needless_return, len_zero, needless_range_loop,
// must_use_candidate, missing_const_for_fn) were discarded.
//
// The text is not used by any consumer, so the field is gone entirely; serde
// ignores unknown JSON keys by default.
#[derive(Debug, Deserialize)]
pub(crate) struct DiagnosticSpan {
    pub(crate) file_name: String,
    pub(crate) line_start: u32,
    pub(crate) line_end: u32,
    pub(crate) column_start: u32,
    pub(crate) column_end: u32,
    #[serde(default)]
    pub(crate) is_primary: bool,
    #[serde(default)]
    pub(crate) suggested_replacement: Option<String>,
    #[serde(default)]
    pub(crate) suggestion_applicability: Option<String>,
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    // ===================
    // ViolationDetail Tests
    // ===================

    #[test]
    fn test_violation_detail_struct() {
        let violation = ViolationDetail {
            file: PathBuf::from("src/main.rs"),
            line: 10,
            column: 5,
            end_line: 10,
            end_column: 20,
            lint_name: "unused_variable".to_string(),
            message: "unused variable: x".to_string(),
            severity: "warning".to_string(),
            suggestion: Some("remove the variable".to_string()),
            machine_applicable: true,
        };

        assert_eq!(violation.file, PathBuf::from("src/main.rs"));
        assert_eq!(violation.line, 10);
        assert!(violation.machine_applicable);
    }

    // ===================
    // FileSummary Tests
    // ===================

    #[test]
    fn test_file_summary_struct() {
        let summary = FileSummary {
            total_violations: 15,
            errors: 5,
            warnings: 10,
            sloc: 200,
            defect_density: 0.075,
        };

        assert_eq!(summary.total_violations, 15);
        assert_eq!(summary.errors, 5);
        assert!((summary.defect_density - 0.075).abs() < 0.001);
    }

    // ===================
    // SeverityDistribution Tests
    // ===================

    #[test]
    fn test_severity_distribution_default() {
        let dist = SeverityDistribution::default();
        assert_eq!(dist.error, 0);
        assert_eq!(dist.warning, 0);
        assert_eq!(dist.suggestion, 0);
        assert_eq!(dist.note, 0);
    }

    // ===================
    // QualityGateStatus Tests
    // ===================

    #[test]
    fn test_quality_gate_status_passed() {
        let status = QualityGateStatus {
            passed: true,
            violations: vec![],
            blocking: false,
        };

        assert!(status.passed);
        assert!(status.violations.is_empty());
    }

    // ===================
    // QualityViolation Tests
    // ===================

    #[test]
    fn test_quality_violation_struct() {
        let violation = QualityViolation {
            rule: "max_density".to_string(),
            threshold: 0.1,
            actual: 0.25,
            severity: "blocking".to_string(),
        };

        assert_eq!(violation.rule, "max_density");
        assert!((violation.threshold - 0.1).abs() < 0.001);
        assert!((violation.actual - 0.25).abs() < 0.001);
    }

    // ===================
    // LintHotspot Tests
    // ===================

    #[test]
    fn test_lint_hotspot_struct() {
        let hotspot = LintHotspot {
            file: PathBuf::from("src/complex.rs"),
            defect_density: 0.2,
            total_violations: 40,
            sloc: 200,
            severity_distribution: SeverityDistribution {
                error: 10,
                warning: 25,
                suggestion: 5,
                note: 0,
            },
            top_lints: vec![
                ("unused_variable".to_string(), 15),
                ("clippy::too_many_arguments".to_string(), 8),
            ],
            detailed_violations: vec![],
        };

        assert_eq!(hotspot.file, PathBuf::from("src/complex.rs"));
        assert_eq!(hotspot.total_violations, 40);
        assert_eq!(hotspot.top_lints.len(), 2);
    }
}
