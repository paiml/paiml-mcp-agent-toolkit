//! Gaming Detector: Anti-Gaming Detection for Coverage and Quality Metrics
//!
//! Detects attempts to game coverage metrics through:
//! - `#[cfg(not(coverage))]` patterns
//! - `.codecov.yml` exclusion changes
//! - Test file deletions
//! - CUDA/AVX file hiding
//!
//! Based on: docs/specifications/improve-pmat-work.md

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Gaming violation detected in codebase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamingViolation {
    /// File where violation was found
    pub file: PathBuf,

    /// Line number (0 if not applicable)
    pub line: usize,

    /// Type of gaming pattern detected
    pub pattern: GamingPattern,

    /// Severity of the violation
    pub severity: Severity,

    /// Human-readable explanation
    pub explanation: String,
}

/// Types of gaming patterns we detect
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GamingPattern {
    /// `#[cfg(not(coverage))]` to exclude code from coverage
    CfgNotCoverage,

    /// `#[cfg(not(tarpaulin))]` to exclude from tarpaulin
    CfgNotTarpaulin,

    /// `#[cfg(not(llvm_cov))]` to exclude from llvm-cov
    CfgNotLlvmCov,

    /// New exclusion added to `.codecov.yml`
    NewCodecovExclusion(String),

    /// Test file deleted during work
    TestFileDeletion,

    /// Test module marked `#[ignore]` during work
    TestModuleIgnored,

    /// CUDA/AVX file removed from manifest
    CriticalFileRemoved,

    /// Coverage exclusion comment pattern
    CoverageExclusionComment,
}

/// Severity of gaming violation
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Severity {
    /// Must be fixed before completion
    Critical,

    /// Should be fixed, but can be overridden
    Warning,

    /// Informational only
    Info,
}

/// Results of gaming detection scan
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GamingDetectionResult {
    /// All violations found
    pub violations: Vec<GamingViolation>,

    /// Number of files scanned
    pub files_scanned: usize,

    /// Patterns that were checked
    pub patterns_checked: Vec<String>,
}

impl GamingDetectionResult {
    /// Check if any critical violations were found
    pub fn has_critical_violations(&self) -> bool {
        self.violations
            .iter()
            .any(|v| v.severity == Severity::Critical)
    }

    /// Get only critical violations
    pub fn critical_violations(&self) -> Vec<&GamingViolation> {
        self.violations
            .iter()
            .filter(|v| v.severity == Severity::Critical)
            .collect()
    }

    /// Count violations by severity
    pub fn count_by_severity(&self, severity: Severity) -> usize {
        self.violations.iter().filter(|v| v.severity == severity).count()
    }
}

/// Detect coverage gaming patterns in a project
pub fn detect_coverage_gaming(project_path: &Path) -> Result<GamingDetectionResult> {
    let mut result = GamingDetectionResult {
        patterns_checked: vec![
            "cfg(not(coverage))".to_string(),
            "cfg(not(tarpaulin))".to_string(),
            "cfg(not(llvm_cov))".to_string(),
            "coverage exclusion comments".to_string(),
        ],
        ..Default::default()
    };

    // Walk through source files
    for entry in walkdir::WalkDir::new(project_path)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| !is_excluded_dir(e.path()))
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && is_source_file(path) {
            result.files_scanned += 1;

            if let Ok(content) = std::fs::read_to_string(path) {
                // Check for cfg(not(coverage)) patterns
                check_cfg_patterns(path, &content, &mut result.violations);

                // Check for coverage exclusion comments
                check_exclusion_comments(path, &content, &mut result.violations);
            }
        }
    }

    // Check for codecov.yml changes
    check_codecov_changes(project_path, &mut result.violations)?;

    Ok(result)
}

/// Check for cfg(not(...)) coverage exclusion patterns
fn check_cfg_patterns(path: &Path, content: &str, violations: &mut Vec<GamingViolation>) {
    let patterns = [
        ("cfg(not(coverage))", GamingPattern::CfgNotCoverage),
        ("cfg(not(tarpaulin))", GamingPattern::CfgNotTarpaulin),
        ("cfg(not(llvm_cov))", GamingPattern::CfgNotLlvmCov),
        ("cfg(not(tarpaulin_include))", GamingPattern::CfgNotTarpaulin),
    ];

    for (line_num, line) in content.lines().enumerate() {
        for (pattern, gaming_type) in &patterns {
            // Check for the pattern in various forms
            if line.contains(pattern)
                || line.contains(&pattern.replace("(", " ("))
                || line.contains(&format!("#[{}]", pattern))
            {
                violations.push(GamingViolation {
                    file: path.to_path_buf(),
                    line: line_num + 1,
                    pattern: gaming_type.clone(),
                    severity: Severity::Critical,
                    explanation: format!(
                        "Found {} at line {} - this excludes code from coverage",
                        pattern,
                        line_num + 1
                    ),
                });
            }
        }
    }
}

/// Check for coverage exclusion comment patterns
fn check_exclusion_comments(path: &Path, content: &str, violations: &mut Vec<GamingViolation>) {
    let comment_patterns = [
        "// LCOV_EXCL_START",
        "// LCOV_EXCL_STOP",
        "// LCOV_EXCL_LINE",
        "/* LCOV_EXCL_START */",
        "/* LCOV_EXCL_STOP */",
        "// coverage:ignore",
        "// istanbul ignore",
        "// c8 ignore",
        "#[no_coverage]",
        "#[coverage(off)]",
    ];

    for (line_num, line) in content.lines().enumerate() {
        for pattern in &comment_patterns {
            if line.contains(pattern) {
                violations.push(GamingViolation {
                    file: path.to_path_buf(),
                    line: line_num + 1,
                    pattern: GamingPattern::CoverageExclusionComment,
                    severity: Severity::Warning,
                    explanation: format!(
                        "Found coverage exclusion comment '{}' at line {}",
                        pattern,
                        line_num + 1
                    ),
                });
            }
        }
    }
}

/// Check for changes to .codecov.yml that add exclusions
fn check_codecov_changes(project_path: &Path, violations: &mut Vec<GamingViolation>) -> Result<()> {
    let codecov_path = project_path.join(".codecov.yml");
    let codecov_yaml = project_path.join("codecov.yml");

    let codecov_file = if codecov_path.exists() {
        Some(codecov_path)
    } else if codecov_yaml.exists() {
        Some(codecov_yaml)
    } else {
        None
    };

    if let Some(codecov_file) = codecov_file {
        // Check if file was modified in current work
        if let Ok(output) = std::process::Command::new("git")
            .args(["diff", "--name-only", "HEAD~1"])
            .current_dir(project_path)
            .output()
        {
            let changed_files = String::from_utf8_lossy(&output.stdout);
            let codecov_rel = codecov_file
                .strip_prefix(project_path)
                .unwrap_or(&codecov_file)
                .to_string_lossy();

            if changed_files.contains(&*codecov_rel) {
                // Check what was added
                if let Ok(content) = std::fs::read_to_string(&codecov_file) {
                    // Look for ignore patterns
                    if content.contains("ignore:")
                        || content.contains("exclude:")
                        || content.contains("paths:")
                    {
                        violations.push(GamingViolation {
                            file: codecov_file,
                            line: 0,
                            pattern: GamingPattern::NewCodecovExclusion(
                                "Modified codecov config".to_string(),
                            ),
                            severity: Severity::Critical,
                            explanation:
                                "codecov.yml was modified during this work item - verify no gaming"
                                    .to_string(),
                        });
                    }
                }
            }
        }
    }

    Ok(())
}

/// Detect test file deletions by comparing against baseline manifest
pub fn detect_test_deletions(
    project_path: &Path,
    baseline_files: &HashSet<PathBuf>,
) -> Vec<GamingViolation> {
    let mut violations = Vec::new();

    for baseline_file in baseline_files {
        let full_path = project_path.join(baseline_file);

        // Check if it was a test file
        let is_test = baseline_file.to_string_lossy().contains("test")
            || baseline_file.to_string_lossy().contains("_test")
            || baseline_file
                .to_string_lossy()
                .contains("/tests/");

        if is_test && !full_path.exists() {
            violations.push(GamingViolation {
                file: baseline_file.clone(),
                line: 0,
                pattern: GamingPattern::TestFileDeletion,
                severity: Severity::Critical,
                explanation: format!(
                    "Test file {} was deleted during this work item",
                    baseline_file.display()
                ),
            });
        }
    }

    violations
}

/// Detect critical file removals (CUDA, AVX, etc.)
pub fn detect_critical_file_removals(
    project_path: &Path,
    baseline_files: &HashSet<PathBuf>,
) -> Vec<GamingViolation> {
    let mut violations = Vec::new();

    for baseline_file in baseline_files {
        let full_path = project_path.join(baseline_file);
        let ext = baseline_file
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        // Critical file types that cannot be removed
        let is_critical = matches!(ext, "cu" | "cuh")  // CUDA
            || baseline_file.to_string_lossy().contains("simd")
            || baseline_file.to_string_lossy().contains("avx")
            || baseline_file.to_string_lossy().contains("neon");

        if is_critical && !full_path.exists() {
            violations.push(GamingViolation {
                file: baseline_file.clone(),
                line: 0,
                pattern: GamingPattern::CriticalFileRemoved,
                severity: Severity::Critical,
                explanation: format!(
                    "Critical file {} (CUDA/SIMD) was removed during this work item",
                    baseline_file.display()
                ),
            });
        }
    }

    violations
}

/// Check if path is an excluded directory
fn is_excluded_dir(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    path_str.contains("/target/")
        || path_str.contains("/.git/")
        || path_str.contains("/node_modules/")
        || path_str.contains("/.pmat-")
        || path_str.contains("/vendor/")
}

/// Check if file is a source file we should scan
fn is_source_file(path: &Path) -> bool {
    let ext = path.extension().and_then(|e| e.to_str());
    matches!(
        ext,
        Some("rs" | "cu" | "cuh" | "c" | "cpp" | "h" | "hpp" | "py" | "ts" | "tsx" | "js" | "jsx" | "go")
    )
}

/// Run meta-falsification check (verify the detector itself is working)
pub fn run_meta_falsification(_project_path: &Path) -> Result<bool> {
    // Create a temporary test pattern to verify detection
    let test_content = r#"
        // This is a meta-test pattern
        #[cfg(not(coverage))]
        fn hidden_function() {}
    "#;

    // The meta-check: if we CAN'T detect this pattern, the detector is broken
    let mut violations = Vec::new();
    check_cfg_patterns(Path::new("meta-test.rs"), test_content, &mut violations);

    // We SHOULD find exactly one violation
    if violations.len() == 1 && violations[0].pattern == GamingPattern::CfgNotCoverage {
        Ok(true) // Meta-check passed: detector is working
    } else {
        Ok(false) // Meta-check FAILED: detector is broken!
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_cfg_not_coverage() {
        let content = r#"
            #[cfg(not(coverage))]
            fn hidden() {}
        "#;

        let mut violations = Vec::new();
        check_cfg_patterns(Path::new("test.rs"), content, &mut violations);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern, GamingPattern::CfgNotCoverage);
        assert_eq!(violations[0].severity, Severity::Critical);
    }

    #[test]
    fn test_detect_cfg_not_tarpaulin() {
        let content = r#"
            #[cfg(not(tarpaulin))]
            mod hidden_tests {}
        "#;

        let mut violations = Vec::new();
        check_cfg_patterns(Path::new("test.rs"), content, &mut violations);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern, GamingPattern::CfgNotTarpaulin);
    }

    #[test]
    fn test_detect_lcov_exclusion() {
        let content = r#"
            // LCOV_EXCL_START
            fn uncovered_code() {
                // This won't be measured
            }
            // LCOV_EXCL_STOP
        "#;

        let mut violations = Vec::new();
        check_exclusion_comments(Path::new("test.rs"), content, &mut violations);

        assert_eq!(violations.len(), 2); // START and STOP
        assert!(violations
            .iter()
            .all(|v| v.pattern == GamingPattern::CoverageExclusionComment));
    }

    #[test]
    fn test_no_false_positives_normal_code() {
        let content = r#"
            fn normal_function() {
                println!("Hello, world!");
            }

            #[cfg(test)]
            mod tests {
                #[test]
                fn it_works() {
                    assert!(true);
                }
            }
        "#;

        let mut violations = Vec::new();
        check_cfg_patterns(Path::new("test.rs"), content, &mut violations);
        check_exclusion_comments(Path::new("test.rs"), content, &mut violations);

        assert!(violations.is_empty());
    }

    #[test]
    fn test_meta_falsification() {
        // The meta-check should pass (detector should find the test pattern)
        let result = run_meta_falsification(Path::new(".")).unwrap();
        assert!(result, "Meta-falsification failed: detector is broken!");
    }

    #[test]
    fn test_gaming_result_critical_check() {
        let result = GamingDetectionResult {
            violations: vec![
                GamingViolation {
                    file: PathBuf::from("test.rs"),
                    line: 1,
                    pattern: GamingPattern::CfgNotCoverage,
                    severity: Severity::Critical,
                    explanation: "Test".to_string(),
                },
                GamingViolation {
                    file: PathBuf::from("test2.rs"),
                    line: 1,
                    pattern: GamingPattern::CoverageExclusionComment,
                    severity: Severity::Warning,
                    explanation: "Test".to_string(),
                },
            ],
            files_scanned: 2,
            patterns_checked: vec![],
        };

        assert!(result.has_critical_violations());
        assert_eq!(result.critical_violations().len(), 1);
        assert_eq!(result.count_by_severity(Severity::Critical), 1);
        assert_eq!(result.count_by_severity(Severity::Warning), 1);
    }

    #[test]
    fn test_is_source_file() {
        assert!(is_source_file(Path::new("main.rs")));
        assert!(is_source_file(Path::new("kernel.cu")));
        assert!(is_source_file(Path::new("lib.py")));
        assert!(!is_source_file(Path::new("readme.md")));
        assert!(!is_source_file(Path::new("config.toml")));
    }

    #[test]
    fn test_test_deletion_detection() {
        let mut baseline = HashSet::new();
        baseline.insert(PathBuf::from("tests/unit_test.rs"));
        baseline.insert(PathBuf::from("src/lib.rs"));

        // Simulate: tests/unit_test.rs doesn't exist (deleted)
        // Note: This test assumes the file doesn't exist in the actual filesystem
        let violations = detect_test_deletions(Path::new("/nonexistent"), &baseline);

        // Both files should be flagged as missing (since /nonexistent doesn't exist)
        // But only the test file should be a TestFileDeletion
        let test_deletions: Vec<_> = violations
            .iter()
            .filter(|v| v.pattern == GamingPattern::TestFileDeletion)
            .collect();
        assert_eq!(test_deletions.len(), 1);
    }
}
