//! Quality gate service implementing the Service trait
//!
//! Enforces quality standards across the codebase

#![cfg_attr(coverage_nightly, coverage(off))]
use super::analysis_service::{
    AnalysisInput, AnalysisOperation, AnalysisOptions, AnalysisResults, AnalysisService,
};
use super::service_base::{Service, ServiceMetrics, ValidationError};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Input for quality gate checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGateInput {
    pub path: PathBuf,
    pub checks: Vec<QualityCheck>,
    pub strict: bool,
}

/// Types of quality checks
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QualityCheck {
    Complexity {
        max: u32,
    },
    Satd {
        tolerance: u32,
    },
    DeadCode {
        max_percentage: f64,
    },
    Coverage {
        min: f64,
    },
    Lint,
    Documentation,
    /// Documentation quality enforcement (PMAT-7001)
    /// Validates CLI help text and MCP tool documentation
    DocsEnforcement {
        /// Check CLI command documentation
        check_cli: bool,
        /// Check MCP tool documentation
        check_mcp: bool,
    },
}

/// Output from quality gate checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGateOutput {
    pub passed: bool,
    pub results: Vec<QualityCheckResult>,
    pub summary: QualitySummary,
}

/// Result of individual quality check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityCheckResult {
    pub check: String,
    pub passed: bool,
    pub message: String,
    pub violations: Vec<Violation>,
}

/// Quality violation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    pub file: String,
    pub line: Option<usize>,
    pub severity: Severity,
    pub message: String,
}

/// Violation severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// Summary of quality gate results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualitySummary {
    pub total_checks: usize,
    pub passed_checks: usize,
    pub failed_checks: usize,
    pub total_violations: usize,
    pub error_count: usize,
    pub warning_count: usize,
}

/// The word every other pmat surface already uses for a check that produced no
/// measurement — reused here rather than invented.
///
/// `pmat quality-gate` and the MCP `quality_gate` tool both NAME an unanswered
/// check instead of skipping it: `UnrunCheck { check, path, reason }` in
/// `src/cli/analysis_utilities/quality_gate_suite.rs`, surfaced to clients as
/// `not_measured` / `checks.not_run`. The rule recorded there is the one this
/// module was violating: a check that did not run has not passed, and an empty
/// disclosure list is a positive claim that a verdict left nothing out.
///
/// [`QualityCheckResult`] has no field for that and its shape is fixed by
/// constructors outside this file, so the vocabulary travels in `message` and in
/// the violation [`not_measured`] attaches.
const NOT_MEASURED: &str = "not_measured";

/// One check this service cannot answer, reported as a named failure rather than
/// as a pass.
///
/// #1090 / T7: `check_coverage`, `check_lint` and `check_documentation` each
/// returned a hardcoded `passed: true` under a message asserting success
/// ("Coverage check passed", "No lint violations", "Documentation is up to
/// date") without reading anything, and `check_complexity`/`check_satd`/
/// `check_dead_code` derived their verdicts from `let violations = vec![]`,
/// which `is_empty()` unconditionally. Both agent entry points ask for exactly
/// those stubs with `strict: true`, so `pmat agent`'s gate printed
/// "PASSED / All Toyota Way standards met! / Great work!" for every tree on
/// earth, including one it could not read.
///
/// The attached violation is deliberately [`Severity::Error`]: `Service::process`
/// derives the NON-strict verdict from `error_count` alone, so a not-measured
/// check carrying no violation would still roll up to an overall pass — the same
/// always-true shape one level higher.
fn not_measured(check: String, reason: &str) -> QualityCheckResult {
    let message = format!("{NOT_MEASURED}: {reason}");
    QualityCheckResult {
        violations: vec![Violation {
            file: check.clone(),
            line: None,
            severity: Severity::Error,
            message: message.clone(),
        }],
        check,
        passed: false,
        message,
    }
}

/// Quality gate service
pub struct QualityGateService {
    metrics: Arc<RwLock<ServiceMetrics>>,
    analysis_service: AnalysisService,
}

impl QualityGateService {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(ServiceMetrics::default())),
            analysis_service: AnalysisService::new(),
        }
    }

    /// Complexity is not measured here, and is no longer guessed at.
    ///
    /// This used to build an [`AnalysisInput`], call the analysis service,
    /// discard the answer with `let _output = …`, and then report
    /// "All functions within complexity limit" off `let violations = vec![]`.
    /// Mapping the discarded answer faithfully would not have fixed it: the
    /// analyzer on the other end of that call was itself three hardcoded
    /// constants for every path (see `AnalysisService::analyze_complexity`), so
    /// the fix would only have moved the fabrication one frame down the stack.
    /// The call is gone with it.
    async fn check_complexity(&self, _path: &Path, max: u32) -> Result<QualityCheckResult> {
        Ok(not_measured(
            format!("Complexity (max: {max})"),
            "this service has no complexity analyzer wired in; run \
             `pmat quality-gate --checks complexity` or `pmat analyze complexity`, \
             which read the tree",
        ))
    }

    /// SATD, from the detector's own findings.
    ///
    /// The analysis was already being run and then thrown away; the verdict came
    /// from `let violations = vec![]`, so `violations.len() <= tolerance` was
    /// true at every tolerance including zero. The output is bound now, and a
    /// tree with no source files in it is reported as not measured rather than
    /// as a clean bill of health — a count that is 0 for an empty directory and
    /// 0 for an unreadable one distinguishes nothing.
    async fn check_satd(&self, path: &Path, tolerance: u32) -> Result<QualityCheckResult> {
        let check = format!("SATD (tolerance: {tolerance})");
        let input = AnalysisInput {
            operation: AnalysisOperation::Satd,
            path: path.to_path_buf(),
            options: AnalysisOptions::default(),
        };

        let output = self.analysis_service.process(input).await?;
        let AnalysisResults::Satd(satd) = &output.results else {
            return Ok(not_measured(
                check,
                "the analysis service answered a SATD request with a different analysis",
            ));
        };

        if satd.total_files == 0 {
            return Ok(not_measured(
                check,
                "the SATD detector found no source files under this path, so there was \
                 nothing to read",
            ));
        }

        let passed = satd.violations.len() <= tolerance as usize;
        // A finding inside tolerance is still a finding and is still reported,
        // but it must not fail the non-strict verdict, which `Service::process`
        // computes from `error_count`.
        let severity = if passed {
            Severity::Warning
        } else {
            Severity::Error
        };
        let violations: Vec<Violation> = satd
            .violations
            .iter()
            .map(|v| Violation {
                file: v.file.clone(),
                line: Some(v.line),
                severity: severity.clone(),
                message: format!("{}: {}", v.category, v.comment.trim()),
            })
            .collect();

        let message = if violations.is_empty() {
            format!("No SATD comments in {} file(s)", satd.total_files)
        } else if passed {
            format!(
                "{} SATD comment(s) in {} file(s), within tolerance {tolerance}",
                violations.len(),
                satd.total_files
            )
        } else {
            format!(
                "{} SATD comment(s) in {} file(s) exceed tolerance {tolerance}",
                violations.len(),
                satd.total_files
            )
        };

        Ok(QualityCheckResult {
            check,
            passed,
            message,
            violations,
        })
    }

    /// Dead code, from the analyzer's own percentage.
    ///
    /// `let percentage = 0.0;` sat under the comment "Would be calculated from
    /// actual results" directly beneath a completed analysis whose
    /// `dead_code_percentage` was being dropped, and `0.0 <= max_percentage`
    /// passes at every threshold a caller can express. The real field is read
    /// now, and an empty tree is not measured rather than 0%.
    async fn check_dead_code(
        &self,
        path: &Path,
        max_percentage: f64,
    ) -> Result<QualityCheckResult> {
        let check = format!("Dead Code (max: {max_percentage}%)");
        let input = AnalysisInput {
            operation: AnalysisOperation::DeadCode,
            path: path.to_path_buf(),
            options: AnalysisOptions::default(),
        };

        let output = self.analysis_service.process(input).await?;
        let AnalysisResults::DeadCode(dead) = &output.results else {
            return Ok(not_measured(
                check,
                "the analysis service answered a dead-code request with a different analysis",
            ));
        };

        if dead.total_files == 0 {
            return Ok(not_measured(
                check,
                "the dead-code analyzer found no source files under this path, so its 0% is \
                 an empty denominator and not a measurement",
            ));
        }

        let percentage = dead.dead_code_percentage;
        let passed = percentage <= max_percentage;
        let severity = if passed {
            Severity::Warning
        } else {
            Severity::Error
        };
        let violations: Vec<Violation> = dead
            .unused_items
            .iter()
            .map(|item| Violation {
                file: item.file.clone(),
                line: Some(item.line),
                severity: severity.clone(),
                message: format!("unused {}: {}", item.item_type, item.item),
            })
            .collect();

        let message = if passed {
            format!(
                "Dead code {percentage:.1}% ({} item(s) over {} file(s)), within the \
                 {max_percentage}% limit",
                dead.dead_code_count, dead.total_files
            )
        } else {
            format!(
                "Dead code {percentage:.1}% ({} item(s) over {} file(s)) exceeds the \
                 {max_percentage}% limit",
                dead.dead_code_count, dead.total_files
            )
        };

        Ok(QualityCheckResult {
            check,
            passed,
            message,
            violations,
        })
    }

    /// Coverage: nothing here reads a coverage report, so nothing here may pass
    /// one. Previously `passed: true` with the message "Coverage check passed".
    async fn check_coverage(&self, _path: &Path, min: f64) -> Result<QualityCheckResult> {
        Ok(not_measured(
            format!("Coverage (min: {min}%)"),
            "this service reads no coverage report; produce one with `cargo llvm-cov` and \
             judge it with `pmat quality-gate --checks coverage`, which fails on an absent \
             report instead of scoring it as zero",
        ))
    }

    /// Lint: no linter is invoked from this process. Previously `passed: true`
    /// with the message "No lint violations".
    async fn check_lint(&self, _path: &Path) -> Result<QualityCheckResult> {
        Ok(not_measured(
            "Lint".to_string(),
            "this service runs no linter; run `pmat verify`, which runs clippy exactly as \
             `ci / lint` runs it",
        ))
    }

    /// Documentation: previously `passed: true` with "Documentation is up to
    /// date", beside [`QualityCheck::DocsEnforcement`] in this same file, which
    /// is genuinely implemented and which neither agent entry point asks for.
    async fn check_documentation(&self, _path: &Path) -> Result<QualityCheckResult> {
        Ok(not_measured(
            "Documentation".to_string(),
            "this service checks no documentation; ask for `QualityCheck::DocsEnforcement`, \
             which is implemented below, or run `pmat quality-gate --checks sections`",
        ))
    }

    /// Check documentation quality enforcement (PMAT-7001)
    async fn check_docs_enforcement(
        &self,
        _path: &Path,
        check_cli: bool,
        check_mcp: bool,
    ) -> Result<QualityCheckResult> {
        use crate::docs_enforcement::mcp_checker::load_mcp_tool_definitions;
        use crate::docs_enforcement::mcp_checker::validate_mcp_documentation;

        let mut violations = Vec::new();
        let mut passed = true;

        // Check MCP documentation
        if check_mcp {
            match load_mcp_tool_definitions() {
                Ok(tools) => {
                    for tool in tools {
                        match validate_mcp_documentation(&tool) {
                            Ok(report) if !report.is_valid() => {
                                passed = false;
                                let tool_name = format!("MCP tool: {}", tool.name);
                                for issue in report.issues {
                                    violations.push(Violation {
                                        file: tool_name.clone(),
                                        line: None,
                                        severity: Severity::Error,
                                        message: issue,
                                    });
                                }
                            }
                            Ok(_) => {}
                            Err(e) => {
                                passed = false;
                                violations.push(Violation {
                                    file: format!("MCP tool: {}", tool.name),
                                    line: None,
                                    severity: Severity::Error,
                                    message: format!("Validation error: {}", e),
                                });
                            }
                        }
                    }
                }
                Err(e) => {
                    passed = false;
                    violations.push(Violation {
                        file: "MCP".to_string(),
                        line: None,
                        severity: Severity::Error,
                        message: format!("Failed to load MCP tools: {}", e),
                    });
                }
            }
        }

        // Check CLI documentation
        if check_cli {
            // For now, we rely on the test suite for CLI validation
            // Future: Could add runtime CLI validation here
            violations.push(Violation {
                file: "CLI".to_string(),
                line: None,
                severity: Severity::Info,
                message: "CLI documentation validated via test suite (cargo test --test cli_docs_enforcement -- --ignored)".to_string(),
            });
        }

        let error_count = violations
            .iter()
            .filter(|v| matches!(v.severity, Severity::Error))
            .count();
        let message = if passed {
            format!(
                "Documentation enforcement passed (MCP: {}, CLI: {})",
                if check_mcp { "checked" } else { "skipped" },
                if check_cli { "info" } else { "skipped" }
            )
        } else {
            format!("{} documentation issues found", error_count)
        };

        Ok(QualityCheckResult {
            check: "Documentation Enforcement (PMAT-7001)".to_string(),
            passed,
            message,
            violations,
        })
    }
}

#[async_trait::async_trait]
impl Service for QualityGateService {
    type Input = QualityGateInput;
    type Output = QualityGateOutput;
    type Error = anyhow::Error;

    async fn process(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        let start = std::time::Instant::now();
        let mut results = Vec::new();

        for check in &input.checks {
            let result = match check {
                QualityCheck::Complexity { max } => {
                    self.check_complexity(&input.path, *max).await?
                }
                QualityCheck::Satd { tolerance } => {
                    self.check_satd(&input.path, *tolerance).await?
                }
                QualityCheck::DeadCode { max_percentage } => {
                    self.check_dead_code(&input.path, *max_percentage).await?
                }
                QualityCheck::Coverage { min } => self.check_coverage(&input.path, *min).await?,
                QualityCheck::Lint => self.check_lint(&input.path).await?,
                QualityCheck::Documentation => self.check_documentation(&input.path).await?,
                QualityCheck::DocsEnforcement {
                    check_cli,
                    check_mcp,
                } => {
                    self.check_docs_enforcement(&input.path, *check_cli, *check_mcp)
                        .await?
                }
            };
            results.push(result);
        }

        let duration = start.elapsed();
        let mut metrics = self.metrics.write().await;

        // Calculate summary
        let total_checks = results.len();
        let passed_checks = results.iter().filter(|r| r.passed).count();
        let failed_checks = total_checks - passed_checks;
        let total_violations: usize = results.iter().map(|r| r.violations.len()).sum();
        let error_count = results
            .iter()
            .flat_map(|r| &r.violations)
            .filter(|v| matches!(v.severity, Severity::Error))
            .count();
        let warning_count = results
            .iter()
            .flat_map(|r| &r.violations)
            .filter(|v| matches!(v.severity, Severity::Warning))
            .count();

        let passed = if input.strict {
            failed_checks == 0
        } else {
            error_count == 0
        };

        metrics.record_request(duration, passed);

        Ok(QualityGateOutput {
            passed,
            results,
            summary: QualitySummary {
                total_checks,
                passed_checks,
                failed_checks,
                total_violations,
                error_count,
                warning_count,
            },
        })
    }

    fn validate_input(&self, input: &Self::Input) -> Result<(), ValidationError> {
        if !input.path.exists() {
            return Err(ValidationError::InvalidValue {
                field: "path".to_string(),
                reason: "Path does not exist".to_string(),
            });
        }

        if input.checks.is_empty() {
            return Err(ValidationError::MissingField {
                field: "checks".to_string(),
            });
        }

        Ok(())
    }

    fn metrics(&self) -> ServiceMetrics {
        self.metrics.blocking_read().clone()
    }

    fn name(&self) -> &'static str {
        "QualityGateService"
    }
}

impl Default for QualityGateService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod quality_gate_service_tests {
    //! Covers QualityGateService validate_input + name + simple
    //! constructor paths in services/quality_gate_service.rs
    //! (188 uncov on broad, 0% cov). The async check_* methods that
    //! delegate to AnalysisService are skipped (they require fixtures).
    use super::*;

    #[test]
    fn test_quality_gate_service_new_creates_default_instance() {
        let svc = QualityGateService::new();
        assert_eq!(svc.name(), "QualityGateService");
    }

    #[test]
    fn test_quality_gate_service_default_matches_new() {
        let svc = QualityGateService::default();
        assert_eq!(svc.name(), "QualityGateService");
    }

    #[test]
    fn test_validate_input_missing_path_errors() {
        let svc = QualityGateService::new();
        let input = QualityGateInput {
            path: PathBuf::from("/tmp/pmat_nope_quality_xyz_0xC0FFEE"),
            checks: vec![QualityCheck::Lint],
            strict: false,
        };
        let err = svc.validate_input(&input).unwrap_err();
        assert!(matches!(err, ValidationError::InvalidValue { .. }));
    }

    #[test]
    fn test_validate_input_empty_checks_errors() {
        let tmp = tempfile::tempdir().unwrap();
        let svc = QualityGateService::new();
        let input = QualityGateInput {
            path: tmp.path().to_path_buf(),
            checks: vec![],
            strict: false,
        };
        let err = svc.validate_input(&input).unwrap_err();
        assert!(matches!(err, ValidationError::MissingField { .. }));
    }

    #[test]
    fn test_validate_input_existing_path_with_checks_ok() {
        let tmp = tempfile::tempdir().unwrap();
        let svc = QualityGateService::new();
        let input = QualityGateInput {
            path: tmp.path().to_path_buf(),
            checks: vec![QualityCheck::Lint, QualityCheck::Documentation],
            strict: true,
        };
        assert!(svc.validate_input(&input).is_ok());
    }

    #[test]
    fn test_quality_check_complexity_serde_roundtrip() {
        let check = QualityCheck::Complexity { max: 20 };
        let json = serde_json::to_string(&check).unwrap();
        let decoded: QualityCheck = serde_json::from_str(&json).unwrap();
        assert!(matches!(decoded, QualityCheck::Complexity { max: 20 }));
    }

    #[test]
    fn test_quality_check_satd_serde_roundtrip() {
        let check = QualityCheck::Satd { tolerance: 5 };
        let json = serde_json::to_string(&check).unwrap();
        let decoded: QualityCheck = serde_json::from_str(&json).unwrap();
        assert!(matches!(decoded, QualityCheck::Satd { tolerance: 5 }));
    }

    #[test]
    fn test_quality_check_dead_code_serde_roundtrip() {
        let check = QualityCheck::DeadCode {
            max_percentage: 5.0,
        };
        let json = serde_json::to_string(&check).unwrap();
        let decoded: QualityCheck = serde_json::from_str(&json).unwrap();
        assert!(matches!(
            decoded,
            QualityCheck::DeadCode { max_percentage } if (max_percentage - 5.0).abs() < 1e-6
        ));
    }

    #[test]
    fn test_quality_check_lint_serde_roundtrip() {
        let check = QualityCheck::Lint;
        let json = serde_json::to_string(&check).unwrap();
        let decoded: QualityCheck = serde_json::from_str(&json).unwrap();
        assert!(matches!(decoded, QualityCheck::Lint));
    }

    #[test]
    fn test_quality_summary_default_values() {
        // QualitySummary should be constructible with all-zero defaults.
        let s = QualitySummary {
            total_checks: 0,
            passed_checks: 0,
            failed_checks: 0,
            total_violations: 0,
            error_count: 0,
            warning_count: 0,
        };
        assert_eq!(s.total_checks, 0);
    }

    // ---- #1090 / T7: a gate that measured nothing must never report a pass ----
    //
    // Every test below fails against the pre-change file, where `check_coverage`,
    // `check_lint` and `check_documentation` were literal `passed: true` and the
    // other three derived their verdicts from `let violations = vec![]` /
    // `let percentage = 0.0`. Nothing in this module could return `passed: false`
    // except `check_docs_enforcement`, which neither agent entry point requests.

    /// The three checks that were hardcoded `passed: true`, plus complexity,
    /// whose analyzer was three hardcoded constants.
    ///
    /// Each is a pure function of its arguments now, so this needs no fixture.
    #[tokio::test]
    async fn the_checks_with_no_implementation_report_not_measured_not_a_pass() {
        let svc = QualityGateService::new();
        let path = Path::new(".");

        let cases = [
            svc.check_coverage(path, 80.0)
                .await
                .expect("coverage check runs"),
            svc.check_lint(path).await.expect("lint check runs"),
            svc.check_documentation(path)
                .await
                .expect("documentation check runs"),
            svc.check_complexity(path, 20)
                .await
                .expect("complexity check runs"),
        ];

        for result in cases {
            assert!(
                !result.passed,
                "a check with no implementation behind it must not pass: {result:?}"
            );
            assert!(
                result.message.contains(NOT_MEASURED),
                "the reason must be named in the vocabulary the shipped gate uses: {result:?}"
            );
            assert!(
                result
                    .violations
                    .iter()
                    .any(|v| matches!(v.severity, Severity::Error)),
                "without an Error-severity violation the non-strict roll-up in \
                 `Service::process` still reports an overall pass: {result:?}"
            );
        }
    }

    /// The roll-up, not just the individual rows.
    ///
    /// This is the exact shape both agent entry points ask for (minus SATD and
    /// dead code, which need a fixture), and the shape that printed
    /// "PASSED / All Toyota Way standards met! / Great work!". `strict: false`
    /// is the weaker of the two verdicts, so it pins both.
    #[tokio::test]
    async fn a_gate_that_measured_nothing_does_not_roll_up_to_passed() {
        use super::Service;
        let tmp = tempfile::tempdir().expect("tempdir");
        let svc = QualityGateService::new();
        let input = QualityGateInput {
            path: tmp.path().to_path_buf(),
            checks: vec![
                QualityCheck::Complexity { max: 20 },
                QualityCheck::Coverage { min: 80.0 },
                QualityCheck::Lint,
                QualityCheck::Documentation,
            ],
            strict: false,
        };

        let out = svc.process(input).await.expect("gate runs");

        assert!(
            !out.passed,
            "four unmeasured checks are not a passing quality gate: {out:?}"
        );
        assert_eq!(
            out.summary.passed_checks, 0,
            "none of these four measured anything: {out:?}"
        );
        assert_eq!(
            out.summary.error_count, 4,
            "each unmeasured check discloses itself once: {out:?}"
        );
    }

    /// SATD: the detector's findings have to reach the caller.
    ///
    /// The marker is assembled from arguments at runtime so this fixture does
    /// not plant a debt marker in pmat's own tree — the same dodge
    /// `satd_detector_tests_extraction.rs` uses.
    #[tokio::test]
    async fn satd_check_reports_the_markers_the_detector_actually_found() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let fixture = tmp.path().join("planted.rs");
        std::fs::write(
            &fixture,
            format!(
                "// {}: planted by a regression test\nfn f() {{}}\n",
                "FIXME"
            ),
        )
        .expect("write fixture");

        let svc = QualityGateService::new();
        let result = svc
            .check_satd(tmp.path(), 0)
            .await
            .expect("satd check runs");

        assert!(
            !result.passed,
            "one marker against a tolerance of zero must fail: {result:?}"
        );
        assert!(
            !result.violations.is_empty(),
            "the detector's finding must reach the gate instead of being discarded: {result:?}"
        );
        assert!(
            result.violations.iter().any(|v| v.file.contains("planted")),
            "the violation must name the file it came from: {result:?}"
        );
    }

    /// An empty tree is a denominator of zero, not a clean bill of health.
    #[tokio::test]
    async fn satd_over_a_tree_with_no_source_files_is_not_measured() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let svc = QualityGateService::new();

        let result = svc
            .check_satd(tmp.path(), 0)
            .await
            .expect("satd check runs");

        assert!(!result.passed, "nothing was read: {result:?}");
        assert!(result.message.contains(NOT_MEASURED), "{result:?}");
    }

    /// Same rule for dead code, whose stub reported a literal `0.0` percentage
    /// under the message "Dead code percentage: 0.0%".
    #[tokio::test]
    async fn dead_code_over_a_tree_with_no_source_files_is_not_measured() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let svc = QualityGateService::new();

        let result = svc
            .check_dead_code(tmp.path(), 10.0)
            .await
            .expect("dead code check runs");

        assert!(
            !result.passed,
            "0 dead lines out of 0 files is not below any limit: {result:?}"
        );
        assert!(result.message.contains(NOT_MEASURED), "{result:?}");
    }

    #[tokio::test]
    async fn test_process_validates_and_processes_empty_failsafe() {
        // Empty checks → validate_input fails with MissingField.
        // We exercise process() through Service trait method dispatch.
        use super::Service;
        let tmp = tempfile::tempdir().unwrap();
        let svc = QualityGateService::new();
        // Via process(), with empty checks, returns success (just empty results).
        // validate_input is called separately by callers.
        let input = QualityGateInput {
            path: tmp.path().to_path_buf(),
            checks: vec![],
            strict: false,
        };
        let result = svc.process(input).await.unwrap();
        assert_eq!(result.summary.total_checks, 0);
        assert!(result.passed, "no checks → vacuously passed");
    }
}
