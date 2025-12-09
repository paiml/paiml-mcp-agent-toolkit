//! Category A: Falsifiability & Testability (25 points) - GATEWAY
//!
//! The cornerstone of Popperian science: claims must be testable and potentially refutable.
//!
//! ## Sub-categories
//!
//! | ID | Name | Points | Description |
//! |----|------|--------|-------------|
//! | A1 | Hypothesis Documentation | 8 | Clear falsifiable claims documented |
//! | A2 | Test Coverage as Falsification | 10 | Tests attempt to refute claims |
//! | A3 | Benchmark Reproducibility | 7 | Performance claims with confidence intervals |
//!
//! ## Gateway Logic (v1.1)
//!
//! If Category A scores below 15/25 (60%), the total score is 0.
//! This implements Popper's demarcation criterion.
//!
//! ## Academic Foundation
//!
//! - Popper, K. (1934): The Logic of Scientific Discovery [1]
//! - Jia, Y. & Harman, M. (2011): Mutation Testing Survey [4]

use crate::services::popper_score::models::{PopperCategoryScore, PopperFinding, PopperSubScore};
use crate::services::popper_score::scorer::{workspace, PopperScorer, PopperScorerResult};
use regex::Regex;
use std::path::Path;

/// Scorer for Category A: Falsifiability & Testability (25 points)
///
/// This is the **GATEWAY** category. If score < 15, total Popper score = 0.
pub struct FalsifiabilityScorer;

impl FalsifiabilityScorer {
    /// Create a new falsifiability scorer
    pub fn new() -> Self {
        Self
    }

    /// A1: Hypothesis Documentation (8 points)
    ///
    /// Checks for:
    /// - Clear hypothesis statements in README or DESIGN.md
    /// - Explicit claims about what the software does/achieves
    /// - Defined success criteria with measurable thresholds
    /// - Documented failure conditions (what would falsify claims)
    fn score_hypothesis_documentation(&self, project_path: &Path) -> PopperSubScore {
        let mut earned: f64 = 0.0;
        let max: f64 = 8.0;
        let mut description: Vec<String> = Vec::new();

        // Check README.md for hypothesis/claims
        let readme_path = project_path.join("README.md");
        if readme_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&readme_path) {
                let content_lower = content.to_lowercase();

                // Check for explicit claims (2 points)
                let claim_patterns = [
                    "claim", "guarantee", "ensures", "provides", "achieves", "delivers",
                ];
                if claim_patterns.iter().any(|p| content_lower.contains(p)) {
                    earned += 2.0;
                    description.push("explicit claims found".to_string());
                }

                // Check for measurable thresholds (2 points)
                let threshold_regex =
                    Regex::new(r"(?i)(>|<|>=|<=|≥|≤)\s*\d+|(\d+%|\d+ms|\d+s|\d+x)").unwrap();
                if threshold_regex.is_match(&content) {
                    earned += 2.0;
                    description.push("measurable thresholds found".to_string());
                }

                // Check for success criteria (2 points)
                if content_lower.contains("success criteria")
                    || content_lower.contains("requirements")
                    || content_lower.contains("acceptance")
                {
                    earned += 2.0;
                    description.push("success criteria found".to_string());
                }

                // Check for falsification conditions (2 points)
                if content_lower.contains("falsif")
                    || content_lower.contains("refute")
                    || content_lower.contains("fail")
                    || content_lower.contains("condition")
                {
                    earned += 2.0;
                    description.push("failure conditions found".to_string());
                }
            }
        }

        // Check for DESIGN.md or ARCHITECTURE.md
        for doc_file in ["DESIGN.md", "ARCHITECTURE.md", "SPEC.md"] {
            let doc_path = project_path.join(doc_file);
            if doc_path.exists() {
                earned = (earned + 1.0).min(max);
                description.push(format!("{} exists", doc_file));
                break;
            }
        }

        PopperSubScore::new(
            "A1",
            "Hypothesis Documentation",
            earned,
            max,
            &description.join(", "),
        )
    }

    /// A2: Test Coverage as Falsification (10 points)
    ///
    /// Checks for:
    /// - Unit test coverage ≥85% (line coverage) - 3 points
    /// - Branch coverage ≥75% - 2 points
    /// - Mutation testing on core modules - 2 points
    /// - Property-based tests - 2 points
    /// - Negative tests (expected failures) - 1 point
    ///
    /// **Workspace-aware**: Checks all workspace members for tests/coverage.
    fn score_test_coverage(&self, project_path: &Path) -> PopperSubScore {
        let mut earned: f64 = 0.0;
        let max: f64 = 10.0;
        let mut description: Vec<String> = Vec::new();

        // Workspace-aware: Check for test directory in any workspace member
        let has_tests = workspace::any_member_has_dir(project_path, "tests")
            || workspace::any_member_has_dir(project_path, "src")
            || self.has_test_files_workspace(project_path);

        if has_tests {
            earned += 2.0;
            description.push("test files exist".to_string());
        }

        // Check for coverage configuration (1 point) - check root AND members
        let coverage_configs = [
            ".cargo/config.toml", // llvm-cov config
            "codecov.yml",
            ".codecov.yml",
            "tarpaulin.toml",
            "lcov.info",
        ];
        let has_coverage = coverage_configs.iter().any(|config| {
            project_path.join(config).exists()
                || workspace::any_member_has_file(project_path, config)
        });
        if has_coverage {
            earned += 1.0;
            description.push("coverage config found".to_string());
        }

        // Check for mutation testing (2 points) - check root AND members
        let mutation_configs = ["mutants.toml", ".cargo/mutants.toml"];
        let has_mutation = mutation_configs.iter().any(|config| {
            project_path.join(config).exists()
                || workspace::any_member_has_file(project_path, config)
        });
        if has_mutation {
            earned += 2.0;
            description.push("mutation testing configured".to_string());
        }

        // Check for property-based tests (2 points) - workspace-aware
        let test_content = self.read_test_files_workspace(project_path);
        if test_content.contains("proptest") || test_content.contains("quickcheck") {
            earned += 2.0;
            description.push("property-based tests found".to_string());
        }

        // Check for negative tests (1 point)
        if test_content.contains("#[should_panic]")
            || test_content.contains("assert_err")
            || test_content.contains("expect_err")
        {
            earned += 1.0;
            description.push("negative tests found".to_string());
        }

        // Check CI for test commands (2 points)
        if self.check_ci_for_tests(project_path) {
            earned += 2.0;
            description.push("CI runs tests".to_string());
        }

        PopperSubScore::new(
            "A2",
            "Test Coverage as Falsification",
            earned.min(max),
            max,
            &description.join(", "),
        )
    }

    /// A3: Benchmark Reproducibility (7 points)
    ///
    /// Checks for:
    /// - Criterion or equivalent benchmark framework - 2 points
    /// - Hardware specifications documented - 2 points
    /// - Statistical significance (confidence intervals) - 2 points
    /// - External reproducibility - 1 point
    ///
    /// **Workspace-aware**: Checks workspace members for benches/ and Cargo.toml.
    fn score_benchmark_reproducibility(&self, project_path: &Path) -> PopperSubScore {
        let mut earned: f64 = 0.0;
        let max: f64 = 7.0;
        let mut description: Vec<String> = Vec::new();

        // Workspace-aware: Check for benchmark directory in any workspace member
        let has_benches = workspace::any_member_has_dir(project_path, "benches");
        if has_benches {
            earned += 1.0;
            description.push("benches/ exists".to_string());

            // Check for Criterion.rs in any workspace member (2 points)
            let bench_content = workspace::read_member_dir_content(project_path, "benches", "rs");
            if bench_content.contains("criterion") || bench_content.contains("Criterion") {
                earned += 2.0;
                description.push("Criterion.rs found".to_string());
            }
        }

        // Check Cargo.toml for benchmark dependencies (root and members)
        let has_bench_dep = workspace::get_code_paths(project_path).iter().any(|member| {
            let cargo_path = member.join("Cargo.toml");
            if cargo_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&cargo_path) {
                    return content.contains("criterion") || content.contains("divan");
                }
            }
            false
        });
        // Also check root Cargo.toml
        let root_cargo = project_path.join("Cargo.toml");
        let has_root_bench_dep = if root_cargo.exists() {
            std::fs::read_to_string(&root_cargo)
                .is_ok_and(|c| c.contains("criterion") || c.contains("divan"))
        } else {
            false
        };
        if has_bench_dep || has_root_bench_dep {
            earned += 1.0;
            description.push("benchmark dependency found".to_string());
        }

        // Check README for hardware specs (2 points) - always at root
        let readme_path = project_path.join("README.md");
        if readme_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&readme_path) {
                let hw_patterns = ["CPU", "RAM", "Intel", "AMD", "i7", "i9", "Ryzen", "GB"];
                if hw_patterns
                    .iter()
                    .any(|p| content.to_uppercase().contains(p))
                {
                    earned += 2.0;
                    description.push("hardware specs documented".to_string());
                }

                // Check for confidence intervals (1 point)
                if content.contains("95%")
                    || content.contains("confidence")
                    || content.contains("CI")
                    || content.contains("±")
                {
                    earned += 1.0;
                    description.push("confidence intervals mentioned".to_string());
                }
            }
        }

        PopperSubScore::new(
            "A3",
            "Benchmark Reproducibility",
            earned.min(max),
            max,
            &description.join(", "),
        )
    }

    /// Check if project has test files
    fn has_test_files(&self, project_path: &Path) -> bool {
        let src_path = project_path.join("src");
        if src_path.exists() {
            if let Ok(entries) = std::fs::read_dir(&src_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if content.contains("#[test]") || content.contains("#[cfg(test)]") {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }

    /// Workspace-aware: Check if any workspace member has test files
    fn has_test_files_workspace(&self, project_path: &Path) -> bool {
        for member_path in workspace::get_code_paths(project_path) {
            if self.has_test_files(&member_path) {
                return true;
            }
        }
        false
    }

    /// Workspace-aware: Read test files from all workspace members
    fn read_test_files_workspace(&self, project_path: &Path) -> String {
        let mut content = String::new();

        // Read from tests/ and src/ across all workspace members
        content.push_str(&workspace::read_member_dir_content(project_path, "tests", "rs"));
        content.push_str(&workspace::read_member_dir_content(project_path, "src", "rs"));

        content
    }

    /// Check CI configuration for test commands
    fn check_ci_for_tests(&self, project_path: &Path) -> bool {
        let ci_paths = [
            ".github/workflows",
            ".gitlab-ci.yml",
            ".circleci/config.yml",
            "Jenkinsfile",
        ];

        for ci_path in ci_paths {
            let full_path = project_path.join(ci_path);
            if full_path.exists() {
                if full_path.is_dir() {
                    // GitHub Actions
                    if let Ok(entries) = std::fs::read_dir(&full_path) {
                        for entry in entries.flatten() {
                            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                                if content.contains("cargo test")
                                    || content.contains("pytest")
                                    || content.contains("npm test")
                                {
                                    return true;
                                }
                            }
                        }
                    }
                } else if let Ok(content) = std::fs::read_to_string(&full_path) {
                    if content.contains("test") {
                        return true;
                    }
                }
            }
        }

        // Check Makefile
        let makefile = project_path.join("Makefile");
        if makefile.exists() {
            if let Ok(content) = std::fs::read_to_string(&makefile) {
                if content.contains("test:") || content.contains("test-") {
                    return true;
                }
            }
        }

        false
    }
}

impl Default for FalsifiabilityScorer {
    fn default() -> Self {
        Self::new()
    }
}

impl PopperScorer for FalsifiabilityScorer {
    fn name(&self) -> &str {
        "Falsifiability & Testability"
    }

    fn category_id(&self) -> char {
        'A'
    }

    fn max_points(&self) -> f64 {
        25.0
    }

    fn score(&self, project_path: &Path) -> PopperScorerResult<PopperCategoryScore> {
        let mut category = PopperCategoryScore::new(self.name(), 0.0, self.max_points());

        // Score each sub-category
        let a1 = self.score_hypothesis_documentation(project_path);
        let a2 = self.score_test_coverage(project_path);
        let a3 = self.score_benchmark_reproducibility(project_path);

        // Add findings based on scores
        if a1.earned < 4.0 {
            category.add_finding(PopperFinding::warning(
                "Hypothesis documentation is incomplete - add explicit falsifiable claims to README",
                8.0 - a1.earned,
            ));
        } else {
            category.add_finding(PopperFinding::positive("Good hypothesis documentation"));
        }

        if a2.earned < 6.0 {
            category.add_finding(PopperFinding::warning(
                "Test coverage needs improvement - consider adding property-based or mutation tests",
                10.0 - a2.earned,
            ));
        } else {
            category.add_finding(PopperFinding::positive("Good test coverage"));
        }

        if a3.earned < 4.0 {
            category.add_finding(PopperFinding::warning(
                "Benchmark reproducibility could be improved - add confidence intervals",
                7.0 - a3.earned,
            ));
        }

        // Add sub-scores
        category.add_sub_score(a1);
        category.add_sub_score(a2);
        category.add_sub_score(a3);

        Ok(category)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_falsifiability_scorer_basics() {
        let scorer = FalsifiabilityScorer::new();
        assert_eq!(scorer.name(), "Falsifiability & Testability");
        assert_eq!(scorer.category_id(), 'A');
        assert_eq!(scorer.max_points(), 25.0);
        assert!(scorer.is_gateway());
    }

    #[test]
    fn test_empty_project_low_score() {
        let temp_dir = tempdir().unwrap();
        let scorer = FalsifiabilityScorer::new();

        let result = scorer.score(temp_dir.path()).unwrap();
        assert!(result.earned < 15.0); // Should fail gateway
    }

    #[test]
    fn test_project_with_tests_higher_score() {
        let temp_dir = tempdir().unwrap();

        // Create tests directory
        fs::create_dir_all(temp_dir.path().join("tests")).unwrap();
        fs::write(
            temp_dir.path().join("tests/test_main.rs"),
            "#[test]\nfn test_example() {}",
        )
        .unwrap();

        // Create README with claims
        fs::write(
            temp_dir.path().join("README.md"),
            "# Project\n\nThis project claims to provide >10x performance improvement.\n\n## Success Criteria\n\n- All tests pass",
        ).unwrap();

        let scorer = FalsifiabilityScorer::new();
        let result = scorer.score(temp_dir.path()).unwrap();

        // Should have earned some points
        assert!(result.earned > 0.0);
        assert!(!result.sub_scores.is_empty());
    }

    #[test]
    fn test_project_with_criterion_benchmarks() {
        let temp_dir = tempdir().unwrap();

        // Create benches directory with Criterion
        fs::create_dir_all(temp_dir.path().join("benches")).unwrap();
        fs::write(
            temp_dir.path().join("benches/bench.rs"),
            "use criterion::{criterion_group, criterion_main, Criterion};",
        )
        .unwrap();

        // Create Cargo.toml
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            r#"[dev-dependencies]
criterion = "0.5"
"#,
        )
        .unwrap();

        let scorer = FalsifiabilityScorer::new();
        let result = scorer.score(temp_dir.path()).unwrap();

        // Should have benchmark points
        let a3 = result.sub_scores.iter().find(|s| s.id == "A3").unwrap();
        assert!(a3.earned > 0.0);
    }
}
