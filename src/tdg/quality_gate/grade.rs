//! Minimum grade enforcement gate for TDG quality enforcement.

use super::types::{GateConfig, GateResult, QualityGate, Severity, Violation, ViolationType};
use crate::tdg::{Grade, TdgBaseline};
use anyhow::Result;
use std::path::PathBuf;

/// Minimum grade enforcement gate
pub struct MinimumGradeGate {
    config: GateConfig,
}

impl MinimumGradeGate {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(config: GateConfig) -> Self {
        Self { config }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_defaults() -> Self {
        Self::new(GateConfig::default())
    }

    /// Get minimum grade for a file based on its language
    fn get_min_grade_for_file(&self, path: &PathBuf) -> Grade {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
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
        debug_assert!(true, "contract: name");
        "MinimumGradeGate"
    }

    fn check(&self, _baseline: &TdgBaseline, current: &TdgBaseline) -> Result<GateResult> {
        debug_assert!(true, "contract: check");
        let mut violations = Vec::new();

        for (path, entry) in &current.files {
            let min_grade = self.get_min_grade_for_file(path);
            if entry.score.grade > min_grade {
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
                "\u{2705} All files meet minimum grade requirements ({} files)",
                current.summary.total_files
            )
        } else {
            format!(
                "\u{274c} {} file(s) below minimum grade threshold",
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
    use crate::tdg::{BaselineEntry, ComponentScores, Language, TdgScore};

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
    fn test_language_specific_min_grades() {
        let baseline = TdgBaseline::new(None);
        let current = create_test_baseline(vec![
            (PathBuf::from("src/main.rs"), 88.0, Grade::BPlus), // Rust requires B+
            (PathBuf::from("src/script.py"), 82.0, Grade::B),   // Python requires B
            (PathBuf::from("src/app.js"), 75.0, Grade::C),      // JS requires B (FAIL)
        ]);

        let gate = MinimumGradeGate::with_defaults();
        let result = gate.check(&baseline, &current).unwrap();

        assert!(!result.passed);
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].path, PathBuf::from("src/app.js"));
    }

    #[test]
    fn test_minimum_grade_gate_name() {
        let gate = MinimumGradeGate::with_defaults();
        assert_eq!(gate.name(), "MinimumGradeGate");
    }

    #[test]
    fn test_minimum_grade_gate_new() {
        let config = GateConfig::default();
        let gate = MinimumGradeGate::new(config);
        assert_eq!(gate.name(), "MinimumGradeGate");
    }

    #[test]
    fn test_get_min_grade_for_rust_file() {
        let gate = MinimumGradeGate::with_defaults();
        let path = PathBuf::from("src/main.rs");
        assert_eq!(gate.get_min_grade_for_file(&path), Grade::BPlus);
    }

    #[test]
    fn test_get_min_grade_for_typescript_file() {
        let gate = MinimumGradeGate::with_defaults();
        let path = PathBuf::from("src/app.ts");
        assert_eq!(gate.get_min_grade_for_file(&path), Grade::BPlus);

        let tsx_path = PathBuf::from("components/App.tsx");
        assert_eq!(gate.get_min_grade_for_file(&tsx_path), Grade::BPlus);
    }

    #[test]
    fn test_get_min_grade_for_javascript_file() {
        let gate = MinimumGradeGate::with_defaults();
        let path = PathBuf::from("src/app.js");
        assert_eq!(gate.get_min_grade_for_file(&path), Grade::B);

        let jsx_path = PathBuf::from("components/App.jsx");
        assert_eq!(gate.get_min_grade_for_file(&jsx_path), Grade::B);
    }

    #[test]
    fn test_get_min_grade_for_python_file() {
        let gate = MinimumGradeGate::with_defaults();
        let path = PathBuf::from("src/main.py");
        assert_eq!(gate.get_min_grade_for_file(&path), Grade::B);
    }

    #[test]
    fn test_get_min_grade_for_unknown_extension() {
        let gate = MinimumGradeGate::with_defaults();
        let path = PathBuf::from("readme.md");
        assert_eq!(gate.get_min_grade_for_file(&path), Grade::B); // default

        let unknown = PathBuf::from("data.xyz");
        assert_eq!(gate.get_min_grade_for_file(&unknown), Grade::B);
    }

    #[test]
    fn test_get_min_grade_for_no_extension() {
        let gate = MinimumGradeGate::with_defaults();
        let path = PathBuf::from("Makefile");
        assert_eq!(gate.get_min_grade_for_file(&path), Grade::B); // default
    }

    #[test]
    fn test_get_min_grade_for_go_file() {
        let gate = MinimumGradeGate::with_defaults();
        let path = PathBuf::from("main.go");
        assert_eq!(gate.get_min_grade_for_file(&path), Grade::B); // not in min_grades
    }

    #[test]
    fn test_get_min_grade_for_java_file() {
        let gate = MinimumGradeGate::with_defaults();
        let path = PathBuf::from("Main.java");
        assert_eq!(gate.get_min_grade_for_file(&path), Grade::B);
    }

    #[test]
    fn test_get_min_grade_for_ruby_file() {
        let gate = MinimumGradeGate::with_defaults();
        let path = PathBuf::from("app.rb");
        assert_eq!(gate.get_min_grade_for_file(&path), Grade::B);
    }

    #[test]
    fn test_get_min_grade_for_php_file() {
        let gate = MinimumGradeGate::with_defaults();
        let path = PathBuf::from("index.php");
        assert_eq!(gate.get_min_grade_for_file(&path), Grade::B);
    }

    #[test]
    fn test_get_min_grade_for_swift_file() {
        let gate = MinimumGradeGate::with_defaults();
        let path = PathBuf::from("App.swift");
        assert_eq!(gate.get_min_grade_for_file(&path), Grade::B);
    }

    #[test]
    fn test_get_min_grade_for_kotlin_file() {
        let gate = MinimumGradeGate::with_defaults();
        let path = PathBuf::from("Main.kt");
        assert_eq!(gate.get_min_grade_for_file(&path), Grade::B);

        let kts_path = PathBuf::from("build.gradle.kts");
        assert_eq!(gate.get_min_grade_for_file(&kts_path), Grade::B);
    }

    #[test]
    fn test_minimum_grade_gate_config_access() {
        let config = GateConfig::default();
        let gate = MinimumGradeGate::new(config);
        assert_eq!(gate.name(), "MinimumGradeGate");
    }

    #[test]
    fn test_minimum_grade_gate_all_grades() {
        let config = GateConfig::default();
        let gate = MinimumGradeGate::new(config);

        let grades_and_expected = vec![
            (Grade::APLus, true),
            (Grade::A, true),
            (Grade::AMinus, true),
            (Grade::BPlus, true),
            (Grade::B, true),       // At default threshold
            (Grade::BMinus, false), // Below default threshold (B)
            (Grade::CPlus, false),
            (Grade::C, false),
            (Grade::CMinus, false),
            (Grade::D, false),
            (Grade::F, false),
        ];

        for (grade, should_pass) in grades_and_expected {
            let score = match grade {
                Grade::APLus => 97.0,
                Grade::A => 93.0,
                Grade::AMinus => 90.0,
                Grade::BPlus => 87.0,
                Grade::B => 83.0,
                Grade::BMinus => 80.0,
                Grade::CPlus => 77.0,
                Grade::C => 73.0,
                Grade::CMinus => 70.0,
                Grade::D => 60.0,
                Grade::F => 50.0,
            };

            let baseline = create_test_baseline(vec![]);
            let current = create_test_baseline(vec![(PathBuf::from("test.txt"), score, grade)]);

            let result = gate.check(&baseline, &current).unwrap();
            if should_pass {
                assert!(result.passed, "Grade {:?} should pass", grade);
            } else {
                assert!(!result.passed, "Grade {:?} should fail", grade);
            }
        }
    }

    #[test]
    fn test_minimum_grade_gate_multiple_files() {
        let config = GateConfig::default();

        let baseline = create_test_baseline(vec![]);
        let current = create_test_baseline(vec![
            (PathBuf::from("a.rs"), 95.0, Grade::A),
            (PathBuf::from("b.rs"), 90.0, Grade::AMinus),
            (PathBuf::from("c.rs"), 85.0, Grade::BPlus),
        ]);

        let gate = MinimumGradeGate::new(config);
        let result = gate.check(&baseline, &current).unwrap();
        assert!(result.passed);
    }
}
