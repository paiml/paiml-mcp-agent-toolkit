//! Shared Known Defects Detection Module
//!
//! Provides defect detection capabilities for:
//! - rust-project-score (KnownDefectsScorer)
//! - TDG analyzer (auto-fail on critical defects)
//! - analyze defects command (project-wide scanning)
//!
//! Based on specification: docs/specifications/known-defects-languages-spec.md

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Defect severity levels (based on production impact)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Critical, // Auto-fail in TDG, exit code 1 in analyze
    High,
    Medium,
    Low,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Critical => "CRITICAL",
            Severity::High => "HIGH",
            Severity::Medium => "MEDIUM",
            Severity::Low => "LOW",
        }
    }
}

/// A detected defect instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefectInstance {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub code_snippet: String,
}

/// A known defect pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefectPattern {
    pub id: String,
    pub name: String,
    pub severity: Severity,
    pub fix_recommendation: String,
    pub bad_example: String,
    pub good_example: String,
    pub evidence_description: String,
    pub evidence_url: Option<String>,
    pub instances: Vec<DefectInstance>,
}

/// Defect detector for Rust code
pub struct RustDefectDetector {
    unwrap_regex: Regex,
}

impl RustDefectDetector {
    pub fn new() -> Self {
        Self {
            unwrap_regex: Regex::new(r"\.unwrap\(\)").expect("internal error"),
        }
    }

    /// Check if a file should be excluded from defect detection
    fn should_exclude_file(&self, file_path: &Path) -> bool {
        let path_str = file_path.to_string_lossy();
        let file_name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        // Exclude test directories
        if path_str.contains("/tests/")
            || path_str.starts_with("tests/")
            || path_str.contains("/benches/")
            || path_str.starts_with("benches/")
        {
            return true;
        }

        // Exclude examples directory (demos and samples often use .expect("internal error") for brevity)
        if path_str.contains("/examples/")
            || path_str.starts_with("examples/")
            || path_str.starts_with("./examples/")
        {
            return true;
        }

        // Exclude fuzz targets (fuzz tests typically use .expect("internal error") for simplicity)
        if path_str.contains("/fuzz/")
            || path_str.starts_with("fuzz/")
            || path_str.starts_with("./fuzz/")
        {
            return true;
        }

        // Exclude test file patterns
        if file_name.ends_with("_tests.rs")
            || file_name.ends_with("_test.rs")
            || file_name.starts_with("test_")
        {
            return true;
        }

        false
    }

    /// Check if content contains test-related markers
    fn has_test_markers(&self, content: &str) -> bool {
        // Check for test cfg attributes
        let has_cfg_test = content.contains("#[cfg(test)]")
            || content.contains("#[cfg(all(test,")
            || content.contains("#[cfg(any(test,");

        // Check for test function attributes
        let has_test_attr = content.contains("#[test]")
            || content.contains("#[tokio::test]")
            || content.contains("#[async_test]");

        has_cfg_test || has_test_attr
    }

    /// Detect all defects in Rust source code
    /// Returns vector of detected defect patterns with instances
    pub fn detect(&self, content: &str, file_path: &Path) -> Vec<DefectPattern> {
        let mut defects = Vec::new();

        // Exclude test files entirely
        if self.should_exclude_file(file_path) {
            return defects;
        }

        // Exclude files with test markers
        if self.has_test_markers(content) {
            return defects;
        }

        // Detect .unwrap() calls
        let unwrap_instances = self.detect_unwraps(content, file_path);
        if !unwrap_instances.is_empty() {
            defects.push(DefectPattern {
                id: "RUST-UNWRAP-001".to_string(),
                name: ".unwrap() calls".to_string(),
                severity: Severity::Critical,
                fix_recommendation:
                    "Use .expect() with descriptive messages or proper error handling with ?"
                        .to_string(),
                bad_example: "let x = result.unwrap();".to_string(),
                good_example: "let x = result.expect(\"Bot feature file must be valid\");"
                    .to_string(),
                evidence_description: "Cloudflare outage 2025-11-18 (3+ hour network outage)"
                    .to_string(),
                evidence_url: Some("https://blog.cloudflare.com/2025-01-18-outage".to_string()),
                instances: unwrap_instances,
            });
        }

        defects
    }

    fn detect_unwraps(&self, content: &str, file_path: &Path) -> Vec<DefectInstance> {
        let mut instances = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            // Skip doc comments - they contain examples, not production code
            if trimmed.starts_with("///") || trimmed.starts_with("//!") {
                continue;
            }
            for mat in self.unwrap_regex.find_iter(line) {
                instances.push(DefectInstance {
                    file: file_path.to_string_lossy().to_string(),
                    line: line_num + 1,
                    column: mat.start() + 1,
                    code_snippet: line.trim().to_string(),
                });
            }
        }

        instances
    }

    /// Count unwrap() calls (used by rust-project-score)
    pub fn count_unwraps(&self, content: &str) -> usize {
        self.unwrap_regex.find_iter(content).count()
    }
}

impl Default for RustDefectDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_detect_unwrap() {
        let detector = RustDefectDetector::new();
        let code = r#"
            fn main() {
                let x = Some(42).unwrap();
            }
        "#;

        let path = PathBuf::from("src/main.rs");
        let defects = detector.detect(code, &path);

        assert_eq!(defects.len(), 1);
        assert_eq!(defects[0].id, "RUST-UNWRAP-001");
        assert_eq!(defects[0].severity, Severity::Critical);
        assert_eq!(defects[0].instances.len(), 1);
    }

    #[test]
    fn test_excludes_doc_comments() {
        let detector = RustDefectDetector::new();
        let code = r#"
            /// # Examples
            ///
            /// ```
            /// let result = something.unwrap();
            /// ```
            pub fn something() -> Option<i32> {
                Some(42)
            }

            //! Module doc with example
            //! let x = foo.unwrap();
        "#;

        let path = PathBuf::from("src/lib.rs");
        let defects = detector.detect(code, &path);

        assert_eq!(defects.len(), 0, "Doc comments should be excluded (issue #131)");
    }

    #[test]
    fn test_excludes_test_code() {
        let detector = RustDefectDetector::new();
        let code = r#"
            #[cfg(test)]
            mod tests {
                fn test_foo() {
                    let x = Some(42).unwrap();
                }
            }
        "#;

        let path = PathBuf::from("src/lib.rs");
        let defects = detector.detect(code, &path);

        assert_eq!(defects.len(), 0, "Test code should be excluded");
    }

    #[test]
    fn test_excludes_test_directory() {
        let detector = RustDefectDetector::new();
        let code = r#"
            fn test_helper() {
                let x = Some(42).expect("internal error");
            }
        "#;

        let path = PathBuf::from("tests/integration_test.rs");
        let defects = detector.detect(code, &path);

        assert_eq!(defects.len(), 0, "Tests directory should be excluded");
    }

    #[test]
    fn test_excludes_examples_directory() {
        let detector = RustDefectDetector::new();
        let code = r#"
            fn main() {
                let x = Some(42).expect("internal error");
            }
        "#;

        // Test various examples path patterns
        for path in &[
            "examples/demo.rs",
            "./examples/demo.rs",
            "server/examples/demo.rs",
        ] {
            let path = PathBuf::from(path);
            let defects = detector.detect(code, &path);
            assert_eq!(
                defects.len(),
                0,
                "Examples directory should be excluded: {}",
                path.display()
            );
        }
    }

    #[test]
    fn test_excludes_fuzz_directory() {
        let detector = RustDefectDetector::new();
        let code = r#"
            fn fuzz_target() {
                let x = Some(42).expect("internal error");
            }
        "#;

        // Test various fuzz path patterns
        for path in &[
            "fuzz/fuzz_targets/target.rs",
            "./fuzz/fuzz_targets/target.rs",
            "server/fuzz/target.rs",
        ] {
            let path = PathBuf::from(path);
            let defects = detector.detect(code, &path);
            assert_eq!(
                defects.len(),
                0,
                "Fuzz directory should be excluded: {}",
                path.display()
            );
        }
    }
}
