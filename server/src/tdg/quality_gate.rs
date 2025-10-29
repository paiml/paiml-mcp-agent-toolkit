//! Quality Gate System for TDG Enforcement (Sprint 66 Phase 2)
//!
//! This module provides quality gates that can enforce quality standards
//! by detecting regressions, enforcing minimum grades, and validating new files.

use crate::tdg::{Grade, TdgBaseline};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Result of running a quality gate check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateResult {
    /// Whether the gate passed
    pub passed: bool,
    /// Gate that was executed
    pub gate_name: String,
    /// Violations found (empty if passed)
    pub violations: Vec<Violation>,
    /// Summary message
    pub message: String,
}

/// A single quality gate violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    /// File that violated the gate
    pub path: PathBuf,
    /// Type of violation
    pub violation_type: ViolationType,
    /// Severity of violation
    pub severity: Severity,
    /// Detailed message
    pub message: String,
    /// Old score (for regressions)
    pub old_score: Option<f32>,
    /// New score
    pub new_score: f32,
    /// Old grade (for regressions)
    pub old_grade: Option<Grade>,
    /// New grade
    pub new_grade: Grade,
}

/// Type of quality gate violation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViolationType {
    /// Quality regression detected
    Regression,
    /// File below minimum grade
    BelowMinimum,
    /// New file below threshold
    NewFileBelowThreshold,
}

/// Severity level for violations
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    /// Informational (doesn't fail gate)
    Info,
    /// Warning (logs but doesn't fail)
    Warning,
    /// Error (fails the gate)
    Error,
    /// Critical (fails gate with high priority)
    Critical,
}

/// Configuration for quality gates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateConfig {
    /// Maximum score drop allowed before flagging regression
    pub max_score_drop: f32,
    /// Whether to allow grade drops (e.g., A → B)
    pub allow_grade_drop: bool,
    /// Minimum grades by language
    pub min_grades: HashMap<String, Grade>,
    /// Default minimum grade if language not specified
    pub default_min_grade: Grade,
    /// Whether to enforce quality on new files
    pub enforce_new_files: bool,
    /// Minimum grade for new files
    pub new_file_min_grade: Grade,
}

impl Default for GateConfig {
    fn default() -> Self {
        let mut min_grades = HashMap::new();
        min_grades.insert("rust".to_string(), Grade::BPlus);
        min_grades.insert("typescript".to_string(), Grade::BPlus);
        min_grades.insert("python".to_string(), Grade::B);
        min_grades.insert("javascript".to_string(), Grade::B);

        Self {
            max_score_drop: 5.0,
            allow_grade_drop: false,
            min_grades,
            default_min_grade: Grade::B,
            enforce_new_files: true,
            new_file_min_grade: Grade::B,
        }
    }
}

/// Trait for quality gates
pub trait QualityGate {
    /// Name of this gate
    fn name(&self) -> &str;

    /// Run the gate check
    fn check(&self, baseline: &TdgBaseline, current: &TdgBaseline) -> Result<GateResult>;
}

/// Regression detection gate
pub struct RegressionGate {
    config: GateConfig,
}

impl RegressionGate {
    pub fn new(config: GateConfig) -> Self {
        Self { config }
    }

    pub fn with_defaults() -> Self {
        Self::new(GateConfig::default())
    }
}

impl QualityGate for RegressionGate {
    fn name(&self) -> &str {
        "RegressionGate"
    }

    fn check(&self, baseline: &TdgBaseline, current: &TdgBaseline) -> Result<GateResult> {
        let comparison = baseline.compare(current);
        let mut violations = Vec::new();

        for regressed in &comparison.regressed {
            let score_drop = regressed.old_score.total - regressed.new_score.total;
            let grade_dropped = regressed.old_score.grade > regressed.new_score.grade;

            // Check if this violates our thresholds
            let is_violation = if !self.config.allow_grade_drop && grade_dropped {
                true // Grade drop not allowed
            } else {
                score_drop > self.config.max_score_drop
            };

            if is_violation {
                let severity = if grade_dropped {
                    if score_drop > 15.0 {
                        Severity::Critical
                    } else {
                        Severity::Error
                    }
                } else if score_drop > 10.0 {
                    Severity::Error
                } else {
                    Severity::Warning
                };

                violations.push(Violation {
                    path: regressed.path.clone(),
                    violation_type: ViolationType::Regression,
                    severity,
                    message: format!(
                        "Quality regression: {} ({:.1}) → {} ({:.1}) [{}]",
                        regressed.grade_change.0,
                        regressed.old_score.total,
                        regressed.grade_change.1,
                        regressed.new_score.total,
                        format_delta(regressed.delta)
                    ),
                    old_score: Some(regressed.old_score.total),
                    new_score: regressed.new_score.total,
                    old_grade: Some(regressed.grade_change.0),
                    new_grade: regressed.grade_change.1,
                });
            }
        }

        let passed = violations.is_empty();
        let message = if passed {
            format!("✅ No quality regressions detected ({} files analyzed)", current.summary.total_files)
        } else {
            format!(
                "❌ {} quality regression(s) detected",
                violations.len()
            )
        };

        Ok(GateResult {
            passed,
            gate_name: self.name().to_string(),
            violations,
            message,
        })
    }
}

/// Minimum grade enforcement gate
pub struct MinimumGradeGate {
    config: GateConfig,
}

impl MinimumGradeGate {
    pub fn new(config: GateConfig) -> Self {
        Self { config }
    }

    pub fn with_defaults() -> Self {
        Self::new(GateConfig::default())
    }

    /// Get minimum grade for a file based on its language
    fn get_min_grade_for_file(&self, path: &PathBuf) -> Grade {
        // Detect language from extension
        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy();
            let language = match ext_str.as_ref() {
                "rs" => "rust",
                "ts" | "tsx" => "typescript",
                "js" | "jsx" => "javascript",
                "py" => "python",
                "go" => "go",
                "java" => "java",
                "rb" => "ruby",
                "php" => "php",
                "swift" => "swift",
                "kt" | "kts" => "kotlin",
                _ => return self.config.default_min_grade,
            };

            self.config
                .min_grades
                .get(language)
                .copied()
                .unwrap_or(self.config.default_min_grade)
        } else {
            self.config.default_min_grade
        }
    }
}

impl QualityGate for MinimumGradeGate {
    fn name(&self) -> &str {
        "MinimumGradeGate"
    }

    fn check(&self, _baseline: &TdgBaseline, current: &TdgBaseline) -> Result<GateResult> {
        let mut violations = Vec::new();

        for (path, entry) in &current.files {
            let min_grade = self.get_min_grade_for_file(path);
            if entry.score.grade < min_grade {
                violations.push(Violation {
                    path: path.clone(),
                    violation_type: ViolationType::BelowMinimum,
                    severity: Severity::Error,
                    message: format!(
                        "Below minimum grade: {} ({:.1}) < required {}",
                        entry.score.grade, entry.score.total, min_grade
                    ),
                    old_score: None,
                    new_score: entry.score.total,
                    old_grade: None,
                    new_grade: entry.score.grade,
                });
            }
        }

        let passed = violations.is_empty();
        let message = if passed {
            format!(
                "✅ All files meet minimum grade requirements ({} files)",
                current.summary.total_files
            )
        } else {
            format!(
                "❌ {} file(s) below minimum grade threshold",
                violations.len()
            )
        };

        Ok(GateResult {
            passed,
            gate_name: self.name().to_string(),
            violations,
            message,
        })
    }
}

/// New file quality gate
pub struct NewFileGate {
    config: GateConfig,
}

impl NewFileGate {
    pub fn new(config: GateConfig) -> Self {
        Self { config }
    }

    pub fn with_defaults() -> Self {
        Self::new(GateConfig::default())
    }
}

impl QualityGate for NewFileGate {
    fn name(&self) -> &str {
        "NewFileGate"
    }

    fn check(&self, baseline: &TdgBaseline, current: &TdgBaseline) -> Result<GateResult> {
        if !self.config.enforce_new_files {
            return Ok(GateResult {
                passed: true,
                gate_name: self.name().to_string(),
                violations: Vec::new(),
                message: "✅ New file enforcement disabled".to_string(),
            });
        }

        let comparison = baseline.compare(current);
        let mut violations = Vec::new();

        // Check newly added files
        for added_path in &comparison.added {
            if let Some(entry) = current.files.get(added_path) {
                if entry.score.grade < self.config.new_file_min_grade {
                    violations.push(Violation {
                        path: added_path.clone(),
                        violation_type: ViolationType::NewFileBelowThreshold,
                        severity: Severity::Error,
                        message: format!(
                            "New file below minimum grade: {} ({:.1}) < required {}",
                            entry.score.grade,
                            entry.score.total,
                            self.config.new_file_min_grade
                        ),
                        old_score: None,
                        new_score: entry.score.total,
                        old_grade: None,
                        new_grade: entry.score.grade,
                    });
                }
            }
        }

        let passed = violations.is_empty();
        let message = if passed {
            if comparison.added.is_empty() {
                "✅ No new files added".to_string()
            } else {
                format!(
                    "✅ All {} new file(s) meet quality threshold",
                    comparison.added.len()
                )
            }
        } else {
            format!(
                "❌ {} new file(s) below quality threshold",
                violations.len()
            )
        };

        Ok(GateResult {
            passed,
            gate_name: self.name().to_string(),
            violations,
            message,
        })
    }
}

/// Format a score delta for display
fn format_delta(delta: f32) -> String {
    if delta >= 0.0 {
        format!("+{:.1}", delta)
    } else {
        format!("{:.1}", delta)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tdg::{BaselineEntry, ComponentScores, TdgScore};

    fn create_test_baseline(scores: Vec<(PathBuf, f32, Grade)>) -> TdgBaseline {
        use crate::tdg::Language;
        let mut baseline = TdgBaseline::new(None);
        for (path, score, grade) in scores {
            let entry = BaselineEntry {
                content_hash: blake3::hash(b"test"),
                score: TdgScore {
                    total: score,
                    grade,
                    structural_complexity: score,
                    semantic_complexity: score,
                    duplication_ratio: 0.0,
                    coupling_score: score,
                    doc_coverage: score,
                    consistency_score: score,
                    entropy_score: score,
                    confidence: 1.0,
                    language: Language::Rust,
                    file_path: Some(path.clone()),
                    penalties_applied: Vec::new(),
                },
                components: ComponentScores::default(),
                git_context: None,
            };
            baseline.add_entry(path, entry);
        }
        baseline
    }

    #[test]
    #[ignore] // RED test - run with --include-ignored
    fn test_regression_gate_detects_score_drop() {
        let baseline = create_test_baseline(vec![(
            PathBuf::from("src/main.rs"),
            90.0,
            Grade::A,
        )]);

        let current = create_test_baseline(vec![(
            PathBuf::from("src/main.rs"),
            75.0,
            Grade::B,
        )]);

        let gate = RegressionGate::with_defaults();
        let result = gate.check(&baseline, &current).unwrap();

        assert!(!result.passed);
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].violation_type, ViolationType::Regression);
    }

    #[test]
    #[ignore]
    fn test_regression_gate_passes_improvement() {
        let baseline = create_test_baseline(vec![(
            PathBuf::from("src/main.rs"),
            75.0,
            Grade::B,
        )]);

        let current = create_test_baseline(vec![(
            PathBuf::from("src/main.rs"),
            90.0,
            Grade::A,
        )]);

        let gate = RegressionGate::with_defaults();
        let result = gate.check(&baseline, &current).unwrap();

        assert!(result.passed);
        assert_eq!(result.violations.len(), 0);
    }

    #[test]
    #[ignore]
    fn test_regression_gate_allows_small_drop() {
        let baseline = create_test_baseline(vec![(
            PathBuf::from("src/main.rs"),
            90.0,
            Grade::A,
        )]);

        let current = create_test_baseline(vec![(
            PathBuf::from("src/main.rs"),
            87.0,
            Grade::A,
        )]);

        let gate = RegressionGate::with_defaults();
        let result = gate.check(&baseline, &current).unwrap();

        assert!(result.passed); // 3 point drop is within threshold
    }

    #[test]
    #[ignore]
    fn test_minimum_grade_gate_enforces_threshold() {
        let baseline = TdgBaseline::new(None);
        let current = create_test_baseline(vec![
            (PathBuf::from("src/good.rs"), 90.0, Grade::A),
            (PathBuf::from("src/bad.rs"), 70.0, Grade::C),
        ]);

        let gate = MinimumGradeGate::with_defaults();
        let result = gate.check(&baseline, &current).unwrap();

        assert!(!result.passed);
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].path, PathBuf::from("src/bad.rs"));
    }

    #[test]
    #[ignore]
    fn test_minimum_grade_gate_passes_all_above_threshold() {
        let baseline = TdgBaseline::new(None);
        let current = create_test_baseline(vec![
            (PathBuf::from("src/good1.rs"), 90.0, Grade::A),
            (PathBuf::from("src/good2.rs"), 85.0, Grade::BPlus),
        ]);

        let gate = MinimumGradeGate::with_defaults();
        let result = gate.check(&baseline, &current).unwrap();

        assert!(result.passed);
    }

    #[test]
    #[ignore]
    fn test_new_file_gate_detects_low_quality_new_files() {
        let baseline = create_test_baseline(vec![(
            PathBuf::from("src/existing.rs"),
            90.0,
            Grade::A,
        )]);

        let current = create_test_baseline(vec![
            (PathBuf::from("src/existing.rs"), 90.0, Grade::A),
            (PathBuf::from("src/new_bad.rs"), 65.0, Grade::D),
        ]);

        let gate = NewFileGate::with_defaults();
        let result = gate.check(&baseline, &current).unwrap();

        assert!(!result.passed);
        assert_eq!(result.violations.len(), 1);
        assert_eq!(
            result.violations[0].violation_type,
            ViolationType::NewFileBelowThreshold
        );
    }

    #[test]
    #[ignore]
    fn test_new_file_gate_allows_good_new_files() {
        let baseline = create_test_baseline(vec![(
            PathBuf::from("src/existing.rs"),
            90.0,
            Grade::A,
        )]);

        let current = create_test_baseline(vec![
            (PathBuf::from("src/existing.rs"), 90.0, Grade::A),
            (PathBuf::from("src/new_good.rs"), 85.0, Grade::BPlus),
        ]);

        let gate = NewFileGate::with_defaults();
        let result = gate.check(&baseline, &current).unwrap();

        assert!(result.passed);
    }

    #[test]
    #[ignore]
    fn test_multiple_violations() {
        let baseline = create_test_baseline(vec![
            (PathBuf::from("src/file1.rs"), 90.0, Grade::A),
            (PathBuf::from("src/file2.rs"), 85.0, Grade::BPlus),
        ]);

        let current = create_test_baseline(vec![
            (PathBuf::from("src/file1.rs"), 70.0, Grade::C),
            (PathBuf::from("src/file2.rs"), 68.0, Grade::C),
        ]);

        let gate = RegressionGate::with_defaults();
        let result = gate.check(&baseline, &current).unwrap();

        assert!(!result.passed);
        assert_eq!(result.violations.len(), 2);
    }

    #[test]
    #[ignore]
    fn test_grade_drop_not_allowed() {
        let mut config = GateConfig::default();
        config.allow_grade_drop = false;

        let baseline = create_test_baseline(vec![(
            PathBuf::from("src/main.rs"),
            91.0,
            Grade::A,
        )]);

        let current = create_test_baseline(vec![(
            PathBuf::from("src/main.rs"),
            89.0,
            Grade::BPlus,
        )]);

        let gate = RegressionGate::new(config);
        let result = gate.check(&baseline, &current).unwrap();

        assert!(!result.passed); // Grade dropped from A to B+
    }

    #[test]
    #[ignore]
    fn test_language_specific_min_grades() {
        let baseline = TdgBaseline::new(None);
        let current = create_test_baseline(vec![
            (PathBuf::from("src/main.rs"), 88.0, Grade::BPlus),    // Rust requires B+
            (PathBuf::from("src/script.py"), 82.0, Grade::B),      // Python requires B
            (PathBuf::from("src/app.js"), 75.0, Grade::C),         // JS requires B (FAIL)
        ]);

        let gate = MinimumGradeGate::with_defaults();
        let result = gate.check(&baseline, &current).unwrap();

        assert!(!result.passed);
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].path, PathBuf::from("src/app.js"));
    }

    #[test]
    #[ignore]
    fn test_empty_baseline_comparison() {
        let baseline = TdgBaseline::new(None);
        let current = TdgBaseline::new(None);

        let gate = RegressionGate::with_defaults();
        let result = gate.check(&baseline, &current).unwrap();

        assert!(result.passed);
        assert_eq!(result.violations.len(), 0);
    }
}
