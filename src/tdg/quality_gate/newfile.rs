//! New file quality gate checks for TDG quality enforcement.

use super::types::{GateConfig, GateResult, QualityGate, Severity, Violation, ViolationType};
use crate::tdg::TdgBaseline;
use anyhow::Result;

/// New file quality gate
pub struct NewFileGate {
    config: GateConfig,
}

impl NewFileGate {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(config: GateConfig) -> Self {
        Self { config }
    }

    pub fn with_defaults() -> Self {
        Self::new(GateConfig::default())
    }
}

impl QualityGate for NewFileGate {
    fn name(&self) -> &str {
        debug_assert!(true, "contract: name");
        "NewFileGate"
    }

    fn check(&self, baseline: &TdgBaseline, current: &TdgBaseline) -> Result<GateResult> {
        debug_assert!(true, "contract: check");
        if !self.config.enforce_new_files {
            return Ok(GateResult {
                passed: true,
                gate_name: self.name().to_string(),
                violations: Vec::new(),
                message: "\u{2705} New file enforcement disabled".to_string(),
            });
        }

        let comparison = baseline.compare(current);
        let mut violations = Vec::new();

        // Check newly added files
        for added_path in &comparison.added {
            if let Some(entry) = current.files.get(added_path) {
                if entry.score.grade > self.config.new_file_min_grade {
                    violations.push(Violation {
                        path: added_path.clone(),
                        violation_type: ViolationType::NewFileBelowThreshold,
                        severity: Severity::Error,
                        message: format!(
                            "New file below minimum grade: {} ({:.1}) < required {}",
                            entry.score.grade, entry.score.total, self.config.new_file_min_grade
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
                "\u{2705} No new files added".to_string()
            } else {
                format!(
                    "\u{2705} All {} new file(s) meet quality threshold",
                    comparison.added.len()
                )
            }
        } else {
            format!(
                "\u{274c} {} new file(s) below quality threshold",
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
    fn test_new_file_gate_detects_low_quality_new_files() {
        let baseline =
            create_test_baseline(vec![(PathBuf::from("src/existing.rs"), 90.0, Grade::A)]);

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
    fn test_new_file_gate_allows_good_new_files() {
        let baseline =
            create_test_baseline(vec![(PathBuf::from("src/existing.rs"), 90.0, Grade::A)]);

        let current = create_test_baseline(vec![
            (PathBuf::from("src/existing.rs"), 90.0, Grade::A),
            (PathBuf::from("src/new_good.rs"), 85.0, Grade::BPlus),
        ]);

        let gate = NewFileGate::with_defaults();
        let result = gate.check(&baseline, &current).unwrap();

        assert!(result.passed);
    }

    #[test]
    fn test_new_file_gate_name() {
        let gate = NewFileGate::with_defaults();
        assert_eq!(gate.name(), "NewFileGate");
    }

    #[test]
    fn test_new_file_gate_new() {
        let config = GateConfig::default();
        let gate = NewFileGate::new(config);
        assert_eq!(gate.name(), "NewFileGate");
    }

    #[test]
    fn test_new_file_gate_disabled() {
        let mut config = GateConfig::default();
        config.enforce_new_files = false;

        let gate = NewFileGate::new(config);
        let baseline = TdgBaseline::new(None);
        let current = create_test_baseline(vec![(PathBuf::from("src/new_bad.rs"), 50.0, Grade::D)]);

        let result = gate.check(&baseline, &current).unwrap();
        assert!(result.passed);
        assert!(result.message.contains("disabled"));
    }

    #[test]
    fn test_new_file_gate_no_new_files() {
        let baseline =
            create_test_baseline(vec![(PathBuf::from("src/existing.rs"), 90.0, Grade::A)]);
        let current =
            create_test_baseline(vec![(PathBuf::from("src/existing.rs"), 90.0, Grade::A)]);

        let gate = NewFileGate::with_defaults();
        let result = gate.check(&baseline, &current).unwrap();

        assert!(result.passed);
        assert!(result.message.contains("No new files"));
    }

    #[test]
    fn test_new_file_gate_config_access() {
        let config = GateConfig::default();
        let gate = NewFileGate::new(config);
        assert_eq!(gate.name(), "NewFileGate");
    }

    #[test]
    fn test_new_file_gate_with_threshold_score() {
        let mut config = GateConfig::default();
        config.new_file_min_grade = Grade::C;

        let baseline = create_test_baseline(vec![]);
        let current = create_test_baseline(vec![
            (PathBuf::from("new.rs"), 73.0, Grade::C), // At threshold
        ]);

        let gate = NewFileGate::new(config);
        let result = gate.check(&baseline, &current).unwrap();
        assert!(result.passed);
    }

    #[test]
    fn test_new_file_gate_just_below_threshold() {
        let mut config = GateConfig::default();
        config.new_file_min_grade = Grade::C;

        let baseline = create_test_baseline(vec![]);
        let current = create_test_baseline(vec![
            (PathBuf::from("new.rs"), 69.0, Grade::CMinus), // Below threshold
        ]);

        let gate = NewFileGate::new(config);
        let result = gate.check(&baseline, &current).unwrap();
        assert!(!result.passed);
    }

    #[test]
    fn test_new_file_gate_multiple_new_files() {
        let config = GateConfig::default();

        let baseline = create_test_baseline(vec![(PathBuf::from("existing.rs"), 90.0, Grade::A)]);
        let current = create_test_baseline(vec![
            (PathBuf::from("existing.rs"), 90.0, Grade::A),
            (PathBuf::from("new1.rs"), 85.0, Grade::B),
            (PathBuf::from("new2.rs"), 88.0, Grade::BPlus),
        ]);

        let gate = NewFileGate::new(config);
        let result = gate.check(&baseline, &current).unwrap();
        assert!(result.passed);
    }

    #[test]
    fn test_new_file_gate_mixed_quality_new_files() {
        let baseline = create_test_baseline(vec![(PathBuf::from("src/old.rs"), 90.0, Grade::A)]);
        let current = create_test_baseline(vec![
            (PathBuf::from("src/old.rs"), 90.0, Grade::A),
            (PathBuf::from("src/new_good.rs"), 92.0, Grade::A),
            (PathBuf::from("src/new_bad.rs"), 55.0, Grade::D),
        ]);

        let gate = NewFileGate::with_defaults();
        let result = gate.check(&baseline, &current).unwrap();

        assert!(!result.passed);
        assert_eq!(result.violations.len(), 1);
        assert!(result
            .message
            .contains("1 new file(s) below quality threshold"));
        assert!(result.violations[0]
            .message
            .contains("New file below minimum grade"));
    }
}
