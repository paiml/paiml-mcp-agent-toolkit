#![allow(unused)]
#![cfg_attr(coverage_nightly, coverage(off))]
//! SATD (Self-Admitted Technical Debt) Analysis Facade
//!
//! Provides a simplified interface for SATD detection and analysis.

use crate::services::service_registry::ServiceRegistry;
use anyhow::Result;
use serde::Serialize;
use std::path::Path;
use std::sync::Arc;

/// Request for SATD analysis
#[derive(Debug, Clone)]
pub struct SatdAnalysisRequest {
    pub path: std::path::PathBuf,
    pub strict_mode: bool,
    pub include_tests: bool,
    /// Extended mode: detects euphemisms like placeholder, stub, "for now" (issue #149)
    pub extended: bool,
}

/// Result of SATD analysis
#[derive(Debug, Clone, Serialize)]
pub struct SatdAnalysisResult {
    pub total_files: usize,
    pub violations: Vec<SatdViolation>,
    pub summary: String,
    /// What the walk found and deliberately did not read.
    ///
    /// Without this a caller sees only the violations, so "this tree is clean"
    /// and "almost every file in this tree was skipped" are the same output.
    #[serde(default)]
    pub skipped: crate::services::satd_detector::SkipCounts,
}

/// Individual SATD violation
#[derive(Debug, Clone, Serialize)]
pub struct SatdViolation {
    pub file_path: String,
    pub line_number: usize,
    pub violation_type: String,
    pub message: String,
    pub severity: SatdSeverity,
}

/// SATD severity levels
#[derive(Debug, Clone, Serialize)]
pub enum SatdSeverity {
    Critical,
    High,
    Medium,
    Low,
}

/// Facade for SATD analysis operations
#[derive(Clone)]
pub struct SatdFacade {
    registry: Arc<ServiceRegistry>,
}

impl SatdFacade {
    /// Create a new SATD facade
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(registry: Arc<ServiceRegistry>) -> Self {
        Self { registry }
    }

    /// Perform SATD analysis on a project
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn analyze_project(
        &self,
        request: SatdAnalysisRequest,
    ) -> Result<SatdAnalysisResult> {
        use crate::services::satd_detector::SATDDetector;

        // Priority: strict_mode > extended > default
        let detector = if request.strict_mode {
            SATDDetector::new_strict()
        } else if request.extended {
            SATDDetector::new_extended()
        } else {
            SATDDetector::new()
        };

        // A single FILE is not a directory to walk. `analyze comprehensive
        // --file src/lib.rs` handed that path straight to `analyze_directory`,
        // whose walk over a file yields nothing: the report came back "Found 0
        // SATD violations in 0 files" with quality_score 100.0 for a file whose
        // TODO the same analysis reports under `-p`.
        if request.path.is_file() {
            let content = tokio::fs::read_to_string(&request.path).await?;
            let satd_items = detector
                .extract_from_content(&content, &request.path)
                .map_err(|e| anyhow::anyhow!("SATD analysis of {:?} failed: {e}", request.path))?;
            return Ok(Self::build_result(&satd_items));
        }

        // Run analysis based on request parameters.
        //
        // The second argument of `analyze_directory_with_tests` IS the
        // include-tests switch. It used to be handed `request.strict_mode`
        // (strict is already applied by the constructor above), so
        // `analyze satd --include-tests` took the with-tests branch and then
        // told the detector *not* to include tests: a `// TODO:` in `tests/`
        // stayed invisible with the flag and without it alike.
        let (satd_items, skipped) = detector
            .analyze_directory_with_stats(&request.path, request.include_tests)
            .await?;

        Ok(Self::build_result_with_skips(&satd_items, skipped))
    }

    /// Convert detector items into the facade's result shape.
    ///
    /// Shared by the directory walk and the single-file path so both report the
    /// same counts for the same debt.
    fn build_result(
        satd_items: &[crate::services::satd_detector::TechnicalDebt],
    ) -> SatdAnalysisResult {
        Self::build_result_with_skips(satd_items, Default::default())
    }

    fn build_result_with_skips(
        satd_items: &[crate::services::satd_detector::TechnicalDebt],
        skipped: crate::services::satd_detector::SkipCounts,
    ) -> SatdAnalysisResult {
        let mut result = Self::build_result_inner(satd_items);
        // The summary sentence carries the scope too, not just the JSON: the
        // text output is what a human actually reads, and "Found 0 SATD
        // violations in 0 files" was the exact sentence that made a walk which
        // skipped everything look like a clean tree (#923).
        if let Some(note) = skipped.note() {
            result.summary = format!("{} ({note})", result.summary);
        }
        result.skipped = skipped;
        result
    }

    fn build_result_inner(
        satd_items: &[crate::services::satd_detector::TechnicalDebt],
    ) -> SatdAnalysisResult {
        let violations: Vec<SatdViolation> = satd_items
            .iter()
            .map(|item| {
                let severity = match item.severity {
                    crate::services::satd_detector::Severity::Critical => SatdSeverity::Critical,
                    crate::services::satd_detector::Severity::High => SatdSeverity::High,
                    crate::services::satd_detector::Severity::Medium => SatdSeverity::Medium,
                    crate::services::satd_detector::Severity::Low => SatdSeverity::Low,
                };

                SatdViolation {
                    file_path: item.file.display().to_string(),
                    line_number: item.line as usize,
                    violation_type: format!("{:?}", item.category),
                    message: item.text.clone(),
                    severity,
                }
            })
            .collect();

        let total_files = violations
            .iter()
            .map(|v| &v.file_path)
            .collect::<std::collections::HashSet<_>>()
            .len();

        let summary = format!(
            "Found {} SATD violations in {} files",
            violations.len(),
            total_files
        );

        SatdAnalysisResult {
            total_files,
            violations,
            summary,
            skipped: Default::default(),
        }
    }

    /// Analyze a single file for SATD
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub async fn analyze_file<P: AsRef<Path>>(&self, path: P) -> Result<SatdAnalysisResult> {
        let request = SatdAnalysisRequest {
            path: path.as_ref().to_path_buf(),
            strict_mode: false,
            include_tests: true,
            extended: false,
        };

        self.analyze_project(request).await
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::service_registry::ServiceRegistry;

    #[tokio::test]
    async fn test_satd_facade_creation() {
        let registry = Arc::new(ServiceRegistry::new());
        let _facade = SatdFacade::new(registry);
    }

    fn facade() -> SatdFacade {
        SatdFacade::new(Arc::new(ServiceRegistry::new()))
    }

    fn crate_with_test_debt() -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(dir.path().join("src")).expect("src");
        std::fs::create_dir_all(dir.path().join("tests")).expect("tests");
        std::fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"st\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .expect("manifest");
        std::fs::write(
            dir.path().join("src/lib.rs"),
            "// TODO: prod debt\npub fn a() {}\n",
        )
        .expect("lib");
        std::fs::write(
            dir.path().join("tests/it.rs"),
            "// TODO: fix this test\npub fn t() {}\n",
        )
        .expect("test file");
        dir
    }

    fn request(path: std::path::PathBuf, include_tests: bool) -> SatdAnalysisRequest {
        SatdAnalysisRequest {
            path,
            strict_mode: false,
            include_tests,
            extended: false,
        }
    }

    /// `--include-tests` forwarded `strict_mode` as the include-tests argument,
    /// so the flag whose only job is to add test-file debt added nothing.
    #[tokio::test]
    async fn test_include_tests_actually_includes_test_files() {
        let dir = crate_with_test_debt();
        let facade = facade();

        let without = facade
            .analyze_project(request(dir.path().to_path_buf(), false))
            .await
            .expect("analysis without tests");
        let with = facade
            .analyze_project(request(dir.path().to_path_buf(), true))
            .await
            .expect("analysis with tests");

        assert_eq!(
            without.violations.len(),
            1,
            "only src/lib.rs debt without the flag: {:?}",
            without.violations
        );
        assert!(
            with.violations.len() > without.violations.len(),
            "--include-tests must add the TODO in tests/: {:?}",
            with.violations
        );
        assert!(
            with.violations
                .iter()
                .any(|v| v.file_path.contains("it.rs")),
            "the test file's debt must be listed: {:?}",
            with.violations
        );
    }

    /// REGRESSION: `SkipCounts::tests` was structurally pinned at 0.
    ///
    /// Discovery dropped test files (`is_source_file() && !is_test_file()`)
    /// before the counting loop in `analyze_directory_with_stats` ever saw
    /// them, and that loop was the only thing that could record a skip. So the
    /// bucket whose entire job is to disclose declined test files — and whose
    /// `note()` renders "N test (use --include-tests)", a string nothing could
    /// ever produce — reported 0 for every tree, `tests/` directory or not.
    /// `files_not_read.tests: 0` beside a `tests/` directory full of unread
    /// code is the same count-with-no-denominator #1015 set out to remove.
    #[tokio::test]
    async fn test_declined_test_files_are_counted_not_silently_dropped() {
        let dir = crate_with_test_debt();
        let facade = facade();

        let without = facade
            .analyze_project(request(dir.path().to_path_buf(), false))
            .await
            .expect("analysis without tests");
        assert_eq!(
            without.skipped.tests, 1,
            "tests/it.rs was found and declined; the report must say so: {:?}",
            without.skipped
        );
        assert!(
            without.summary.contains("1 test (use --include-tests)"),
            "and the human-readable summary must disclose it too: {}",
            without.summary
        );

        let with = facade
            .analyze_project(request(dir.path().to_path_buf(), true))
            .await
            .expect("analysis with tests");
        assert_eq!(
            with.skipped.tests, 0,
            "with --include-tests nothing is declined for being a test: {:?}",
            with.skipped
        );
    }

    /// A single FILE path was walked as if it were a directory, so a file with
    /// known debt reported zero violations.
    #[tokio::test]
    async fn test_a_single_file_path_is_analyzed_not_walked() {
        let dir = crate_with_test_debt();
        let file = dir.path().join("src/lib.rs");

        let result = facade()
            .analyze_project(request(file.clone(), false))
            .await
            .expect("single-file analysis");

        assert_eq!(
            result.violations.len(),
            1,
            "the file's own TODO must be reported: {:?}",
            result.violations
        );
        assert_eq!(result.total_files, 1);
        assert!(result.violations[0].file_path.contains("lib.rs"));
    }
}
