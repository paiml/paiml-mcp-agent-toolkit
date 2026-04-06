//! F-grade specific handling for TDG quality enforcement.

use super::types::{GateResult, QualityGate, Severity, Violation, ViolationType};
use crate::tdg::{Grade, TdgBaseline};
use anyhow::Result;

/// F-Grade detection gate - fails when any F-grade files exist
///
/// This gate addresses the "hidden F-grade" problem where averaging
/// can mask critical quality issues. Any file with F grade (<50 points)
/// indicates severe technical debt that should be addressed before
/// the project can achieve high quality ratings.
pub struct FGradeGate {
    /// Maximum allowed F-grade files (default: 0)
    max_f_grades: usize,
}

impl FGradeGate {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(max_f_grades: usize) -> Self {
        Self { max_f_grades }
    }

    /// Create with strict defaults (zero F-grades allowed)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_defaults() -> Self {
        Self::new(0)
    }

    /// Create with relaxed policy (allows some F-grades during migration)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_migration_policy(max_f_grades: usize) -> Self {
        Self::new(max_f_grades)
    }
}

impl QualityGate for FGradeGate {
    fn name(&self) -> &str {
        "FGradeGate"
    }

    fn check(&self, _baseline: &TdgBaseline, current: &TdgBaseline) -> Result<GateResult> {
        let mut violations = Vec::new();
        let mut f_grade_count = 0;

        for (path, entry) in &current.files {
            if entry.score.grade == Grade::F {
                f_grade_count += 1;
                violations.push(Violation {
                    path: path.clone(),
                    violation_type: ViolationType::BelowMinimum,
                    severity: Severity::Critical,
                    message: format!(
                        "F-grade file: {} ({:.1} points) - requires immediate attention",
                        entry.score.grade, entry.score.total
                    ),
                    old_score: None,
                    new_score: entry.score.total,
                    old_grade: None,
                    new_grade: entry.score.grade,
                });
            }
        }

        let passed = f_grade_count <= self.max_f_grades;
        let message = if passed {
            if f_grade_count == 0 {
                format!(
                    "\u{2705} No F-grade files detected ({} files analyzed)",
                    current.summary.total_files
                )
            } else {
                format!(
                    "\u{26a0}\u{fe0f} {} F-grade file(s) detected (within migration limit of {})",
                    f_grade_count, self.max_f_grades
                )
            }
        } else {
            format!(
                "\u{274c} {} F-grade file(s) detected (max allowed: {}). \
                F-grades cap project score at B. Fix these files to improve project grade.",
                f_grade_count, self.max_f_grades
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
    use crate::tdg::{BaselineEntry, ComponentScores, Language, TdgScore};
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
    fn test_f_grade_gate_detects_f_grades() {
        let baseline = TdgBaseline::new(None);
        let current = create_test_baseline(vec![
            (PathBuf::from("src/good.rs"), 90.0, Grade::A),
            (PathBuf::from("src/bad.rs"), 40.0, Grade::F),
        ]);

        let gate = FGradeGate::with_defaults();
        let result = gate.check(&baseline, &current).unwrap();

        assert!(!result.passed);
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].severity, Severity::Critical);
        assert!(result.message.contains("1 F-grade file"));
    }

    #[test]
    fn test_f_grade_gate_passes_no_f_grades() {
        let baseline = TdgBaseline::new(None);
        let current = create_test_baseline(vec![
            (PathBuf::from("src/good1.rs"), 90.0, Grade::A),
            (PathBuf::from("src/good2.rs"), 75.0, Grade::B),
            (PathBuf::from("src/ok.rs"), 60.0, Grade::C),
        ]);

        let gate = FGradeGate::with_defaults();
        let result = gate.check(&baseline, &current).unwrap();

        assert!(result.passed);
        assert_eq!(result.violations.len(), 0);
        assert!(result.message.contains("No F-grade files"));
    }

    #[test]
    fn test_f_grade_gate_multiple_f_grades() {
        let baseline = TdgBaseline::new(None);
        let current = create_test_baseline(vec![
            (PathBuf::from("src/bad1.rs"), 30.0, Grade::F),
            (PathBuf::from("src/bad2.rs"), 45.0, Grade::F),
            (PathBuf::from("src/good.rs"), 90.0, Grade::A),
        ]);

        let gate = FGradeGate::with_defaults();
        let result = gate.check(&baseline, &current).unwrap();

        assert!(!result.passed);
        assert_eq!(result.violations.len(), 2);
        assert!(result.message.contains("2 F-grade file"));
    }

    #[test]
    fn test_f_grade_gate_migration_policy() {
        let baseline = TdgBaseline::new(None);
        let current = create_test_baseline(vec![
            (PathBuf::from("src/legacy1.rs"), 40.0, Grade::F),
            (PathBuf::from("src/legacy2.rs"), 45.0, Grade::F),
            (PathBuf::from("src/good.rs"), 90.0, Grade::A),
        ]);

        let gate = FGradeGate::with_migration_policy(2);
        let result = gate.check(&baseline, &current).unwrap();

        assert!(result.passed);
        assert_eq!(result.violations.len(), 2);
        assert!(result.message.contains("within migration limit"));
    }

    #[test]
    fn test_f_grade_gate_migration_policy_exceeded() {
        let baseline = TdgBaseline::new(None);
        let current = create_test_baseline(vec![
            (PathBuf::from("src/bad1.rs"), 30.0, Grade::F),
            (PathBuf::from("src/bad2.rs"), 35.0, Grade::F),
            (PathBuf::from("src/bad3.rs"), 40.0, Grade::F),
        ]);

        let gate = FGradeGate::with_migration_policy(2);
        let result = gate.check(&baseline, &current).unwrap();

        assert!(!result.passed);
        assert_eq!(result.violations.len(), 3);
        assert!(result.message.contains("max allowed: 2"));
    }

    #[test]
    fn test_f_grade_gate_name() {
        let gate = FGradeGate::with_defaults();
        assert_eq!(gate.name(), "FGradeGate");
    }

    #[test]
    fn test_f_grade_gate_new() {
        let gate = FGradeGate::new(5);
        assert_eq!(gate.name(), "FGradeGate");
    }

    #[test]
    fn test_f_grade_gate_empty_baseline() {
        let baseline = TdgBaseline::new(None);
        let current = TdgBaseline::new(None);

        let gate = FGradeGate::with_defaults();
        let result = gate.check(&baseline, &current).unwrap();

        assert!(result.passed);
        assert_eq!(result.violations.len(), 0);
    }

    #[test]
    fn test_f_grade_gate_d_grade_not_f() {
        let baseline = TdgBaseline::new(None);
        let current = create_test_baseline(vec![
            (PathBuf::from("src/poor.rs"), 55.0, Grade::D), // D is not F
            (PathBuf::from("src/good.rs"), 90.0, Grade::A),
        ]);

        let gate = FGradeGate::with_defaults();
        let result = gate.check(&baseline, &current).unwrap();

        assert!(result.passed); // D grade passes FGradeGate
        assert_eq!(result.violations.len(), 0);
    }
}
