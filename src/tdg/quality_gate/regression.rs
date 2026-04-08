//! Regression detection gate for TDG quality enforcement.

use super::types::{GateConfig, GateResult, QualityGate, Severity, Violation, ViolationType};
use crate::tdg::TdgBaseline;
use anyhow::Result;

/// Regression detection gate
pub struct RegressionGate {
    config: GateConfig,
}

impl RegressionGate {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new(config: GateConfig) -> Self {
        Self { config }
    }

    /// With defaults.
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
            let grade_dropped = regressed.old_score.grade < regressed.new_score.grade;

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
                        "Quality regression: {} ({:.1}) \u{2192} {} ({:.1}) [{}]",
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
            format!(
                "\u{2705} No quality regressions detected ({} files analyzed)",
                current.summary.total_files
            )
        } else {
            format!(
                "\u{274c} {} quality regression(s) detected",
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn format_delta(delta: f32) -> String {
    if delta >= 0.0 {
        format!("+{:.1}", delta)
    } else {
        format!("{:.1}", delta)
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::tdg::{BaselineEntry, ComponentScores, Grade, Language, TdgScore};
    use std::path::PathBuf;

    fn create_test_baseline(scores: Vec<(PathBuf, f32, Grade)>) -> TdgBaseline {
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
                    critical_defects_count: 0,
                    has_critical_defects: false,
                    has_contract_coverage: false,
                },
                components: ComponentScores::default(),
                git_context: None,
            };
            baseline.add_entry(path, entry);
        }
        baseline
    }

    #[test]
    fn test_regression_gate_detects_score_drop() {
        let baseline = create_test_baseline(vec![(PathBuf::from("src/main.rs"), 90.0, Grade::A)]);
        let current = create_test_baseline(vec![(PathBuf::from("src/main.rs"), 75.0, Grade::B)]);

        let gate = RegressionGate::with_defaults();
        let result = gate.check(&baseline, &current).unwrap();

        assert!(!result.passed);
        assert_eq!(result.violations.len(), 1);
        assert_eq!(
            result.violations[0].violation_type,
            ViolationType::Regression
        );
    }

    #[test]
    fn test_regression_gate_passes_improvement() {
        let baseline = create_test_baseline(vec![(PathBuf::from("src/main.rs"), 75.0, Grade::B)]);
        let current = create_test_baseline(vec![(PathBuf::from("src/main.rs"), 90.0, Grade::A)]);

        let gate = RegressionGate::with_defaults();
        let result = gate.check(&baseline, &current).unwrap();

        assert!(result.passed);
        assert_eq!(result.violations.len(), 0);
    }

    #[test]
    fn test_regression_gate_allows_small_drop() {
        let baseline = create_test_baseline(vec![(PathBuf::from("src/main.rs"), 90.0, Grade::A)]);
        let current = create_test_baseline(vec![(PathBuf::from("src/main.rs"), 87.0, Grade::A)]);

        let gate = RegressionGate::with_defaults();
        let result = gate.check(&baseline, &current).unwrap();

        assert!(result.passed); // 3 point drop is within threshold
    }

    #[test]
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
    fn test_grade_drop_not_allowed() {
        let mut config = GateConfig::default();
        config.allow_grade_drop = false;

        let baseline = create_test_baseline(vec![(PathBuf::from("src/main.rs"), 91.0, Grade::A)]);
        let current =
            create_test_baseline(vec![(PathBuf::from("src/main.rs"), 89.0, Grade::BPlus)]);

        let gate = RegressionGate::new(config);
        let result = gate.check(&baseline, &current).unwrap();

        assert!(!result.passed); // Grade dropped from A to B+
    }

    #[test]
    fn test_empty_baseline_comparison() {
        let baseline = TdgBaseline::new(None);
        let current = TdgBaseline::new(None);

        let gate = RegressionGate::with_defaults();
        let result = gate.check(&baseline, &current).unwrap();

        assert!(result.passed);
        assert_eq!(result.violations.len(), 0);
    }

    #[test]
    fn test_regression_gate_name() {
        let gate = RegressionGate::with_defaults();
        assert_eq!(gate.name(), "RegressionGate");
    }

    #[test]
    fn test_regression_gate_new() {
        let config = GateConfig::default();
        let gate = RegressionGate::new(config);
        assert_eq!(gate.name(), "RegressionGate");
    }

    #[test]
    fn test_format_delta_positive() {
        assert_eq!(format_delta(5.5), "+5.5");
        assert_eq!(format_delta(0.0), "+0.0");
        assert_eq!(format_delta(10.0), "+10.0");
    }

    #[test]
    fn test_format_delta_negative() {
        assert_eq!(format_delta(-5.5), "-5.5");
        assert_eq!(format_delta(-10.0), "-10.0");
        assert_eq!(format_delta(-0.1), "-0.1");
    }

    #[test]
    fn test_regression_gate_critical_severity() {
        let baseline =
            create_test_baseline(vec![(PathBuf::from("src/critical.rs"), 95.0, Grade::APLus)]);
        let current =
            create_test_baseline(vec![(PathBuf::from("src/critical.rs"), 60.0, Grade::D)]);

        let gate = RegressionGate::with_defaults();
        let result = gate.check(&baseline, &current).unwrap();

        assert!(!result.passed);
        assert_eq!(result.violations.len(), 1);
        assert!(result.violations[0].severity >= Severity::Error);
    }

    #[test]
    fn test_regression_gate_error_severity_no_grade_drop() {
        let baseline = create_test_baseline(vec![(PathBuf::from("src/test.rs"), 90.0, Grade::A)]);
        let current = create_test_baseline(vec![
            (PathBuf::from("src/test.rs"), 75.0, Grade::A), // Same grade, big score drop
        ]);

        let mut config = GateConfig::default();
        config.allow_grade_drop = true;
        let gate = RegressionGate::new(config);
        let result = gate.check(&baseline, &current).unwrap();

        assert!(!result.passed);
        if !result.violations.is_empty() {
            assert_eq!(result.violations[0].severity, Severity::Error);
        }
    }

    #[test]
    fn test_regression_gate_warning_severity() {
        let baseline = create_test_baseline(vec![(PathBuf::from("src/test.rs"), 90.0, Grade::A)]);
        let current = create_test_baseline(vec![
            (PathBuf::from("src/test.rs"), 82.0, Grade::A), // Same grade, small-medium drop
        ]);

        let mut config = GateConfig::default();
        config.allow_grade_drop = true;
        let gate = RegressionGate::new(config);
        let result = gate.check(&baseline, &current).unwrap();

        assert!(!result.passed);
        if !result.violations.is_empty() {
            assert_eq!(result.violations[0].severity, Severity::Warning);
        }
    }

    #[test]
    fn test_regression_gate_config_access() {
        let config = GateConfig::default();
        let gate = RegressionGate::new(config);
        assert_eq!(gate.name(), "RegressionGate");
    }

    #[test]
    fn test_regression_gate_with_allowed_grade_drop() {
        let mut config = GateConfig::default();
        config.allow_grade_drop = true;
        config.max_score_drop = 20.0;

        let baseline = create_test_baseline(vec![(PathBuf::from("src/main.rs"), 91.0, Grade::A)]);
        let current =
            create_test_baseline(vec![(PathBuf::from("src/main.rs"), 85.0, Grade::BPlus)]);

        let gate = RegressionGate::new(config);
        let result = gate.check(&baseline, &current).unwrap();

        assert!(result.passed);
    }

    #[test]
    fn test_format_delta_small_values() {
        assert_eq!(format_delta(0.1), "+0.1");
        assert_eq!(format_delta(-0.1), "-0.1");
        assert_eq!(format_delta(0.01), "+0.0"); // Formatted to 1 decimal
    }

    #[test]
    fn test_format_delta_large_values() {
        assert_eq!(format_delta(100.0), "+100.0");
        assert_eq!(format_delta(-100.0), "-100.0");
    }

    #[test]
    fn test_regression_gate_exact_threshold() {
        let mut config = GateConfig::default();
        config.max_score_drop = 5.0;
        config.allow_grade_drop = true;

        let baseline =
            create_test_baseline(vec![(PathBuf::from("src/main.rs"), 80.0, Grade::BMinus)]);
        let current = create_test_baseline(vec![
            (PathBuf::from("src/main.rs"), 75.0, Grade::C), // Exactly 5.0 drop
        ]);

        let gate = RegressionGate::new(config);
        let result = gate.check(&baseline, &current).unwrap();
        assert!(result.passed);
    }

    #[test]
    fn test_regression_gate_just_over_threshold() {
        let mut config = GateConfig::default();
        config.max_score_drop = 5.0;

        let baseline =
            create_test_baseline(vec![(PathBuf::from("src/main.rs"), 80.0, Grade::BMinus)]);
        let current = create_test_baseline(vec![
            (PathBuf::from("src/main.rs"), 74.9, Grade::C), // 5.1 drop - over threshold
        ]);

        let gate = RegressionGate::new(config);
        let result = gate.check(&baseline, &current).unwrap();
        assert!(!result.passed);
    }

    #[test]
    fn test_regression_gate_multiple_files() {
        let config = GateConfig::default();

        let baseline = create_test_baseline(vec![
            (PathBuf::from("a.rs"), 90.0, Grade::A),
            (PathBuf::from("b.rs"), 85.0, Grade::B),
            (PathBuf::from("c.rs"), 80.0, Grade::BMinus),
        ]);
        let current = create_test_baseline(vec![
            (PathBuf::from("a.rs"), 89.0, Grade::A),      // Minor drop
            (PathBuf::from("b.rs"), 86.0, Grade::B),      // Improved
            (PathBuf::from("c.rs"), 82.0, Grade::BMinus), // Improved
        ]);

        let gate = RegressionGate::new(config);
        let result = gate.check(&baseline, &current).unwrap();
        assert!(result.passed);
    }

    #[test]
    fn test_regression_gate_file_removed() {
        let config = GateConfig::default();

        let baseline = create_test_baseline(vec![
            (PathBuf::from("a.rs"), 90.0, Grade::A),
            (PathBuf::from("b.rs"), 85.0, Grade::B),
        ]);
        let current = create_test_baseline(vec![
            (PathBuf::from("a.rs"), 90.0, Grade::A),
            // b.rs removed
        ]);

        let gate = RegressionGate::new(config);
        let result = gate.check(&baseline, &current).unwrap();
        assert!(result.passed);
    }

    #[test]
    fn test_format_delta_zero() {
        let result = format_delta(0.0);
        assert!(result.contains("0"));
    }
}
