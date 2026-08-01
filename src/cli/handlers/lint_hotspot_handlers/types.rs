#![cfg_attr(coverage_nightly, coverage(off))]
//! Types, enums, and structs for lint hotspot analysis

use crate::cli::LintHotspotOutputFormat;
use serde::{Deserialize, Serialize, Serializer};
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

/// Serialize a `HashMap` keyed by path in sorted key order.
///
/// `summary_by_file` is a `HashMap`, and `serde_json` walks it in hash order,
/// which Rust randomises per process. That made
/// `analyze lint-hotspot --format enforcement-json` emit a *different byte
/// stream on every run for identical input* — unusable for CI diffing. Sorting
/// on the way out costs one BTreeMap build and makes the document reproducible.
fn serialize_map_sorted<S, V>(map: &HashMap<PathBuf, V>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    V: Serialize,
{
    let ordered: BTreeMap<&PathBuf, &V> = map.iter().collect();
    ordered.serialize(serializer)
}

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
    #[serde(serialize_with = "serialize_map_sorted")]
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

#[derive(Debug, Deserialize)]
pub(crate) struct DiagnosticSpan {
    pub(crate) file_name: String,
    pub(crate) line_start: u32,
    pub(crate) line_end: u32,
    pub(crate) column_start: u32,
    pub(crate) column_end: u32,
    #[serde(default)]
    pub(crate) is_primary: bool,
    #[serde(default, rename = "text")]
    pub(crate) _text: Vec<DiagnosticText>,
    #[serde(default)]
    pub(crate) suggested_replacement: Option<String>,
    #[serde(default)]
    pub(crate) suggestion_applicability: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DiagnosticText {
    pub(crate) _text: String,
    pub(crate) _highlight_start: u32,
    pub(crate) _highlight_end: u32,
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
