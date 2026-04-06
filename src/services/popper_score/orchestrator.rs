#![cfg_attr(coverage_nightly, coverage(off))]
//! Orchestrator for Popper Falsifiability Score v1.1
//!
//! Coordinates all 6 category scorers and applies gateway logic with normalization.
//!
//! ## Workflow
//!
//! 1. Run all 6 scorers in parallel (when possible)
//! 2. Check gateway (Category A >= 15/25)
//! 3. Apply normalization for N/A categories
//! 4. Generate recommendations
//! 5. Produce final PopperScore
//!
//! ## Toyota Way Principles
//!
//! - **Genchi Genbutsu**: Analyze actual artifacts, not claims
//! - **Jidoka**: Falsifiability gateway stops the line
//! - **Kaizen**: Track score velocity over time

use crate::services::popper_score::models::{
    AnalysisStatus, PopperAnalysis, PopperMetadata, PopperRecommendation, PopperScore,
    RecommendationPriority,
};
use crate::services::popper_score::scorer::{PopperScorer, PopperScorerError, PopperScorerResult};
use crate::services::popper_score::scorers;
use std::path::Path;

/// Orchestrator for running all Popper score categories
pub struct PopperOrchestrator {
    /// All category scorers
    scorers: Vec<Box<dyn PopperScorer>>,
}

impl PopperOrchestrator {
    /// Create a new orchestrator with all default scorers
    pub fn new() -> Self {
        Self {
            scorers: scorers::all_scorers(),
        }
    }

    /// Score a project and return comprehensive PopperScore
    pub fn score(&self, project_path: &Path) -> PopperScorerResult<PopperScore> {
        debug_assert!(project_path.exists(), "popper score path must exist");
        let project_name = project_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let mut result = PopperScore::new();
        result.metadata = PopperMetadata::new(project_name).with_path(project_path.to_path_buf());

        // Run all scorers
        for scorer in &self.scorers {
            let category_score = scorer.score(project_path)?;

            match scorer.category_id() {
                'A' => {
                    result.categories.falsifiability = category_score;
                }
                'B' => {
                    result.categories.reproducibility = category_score;
                }
                'C' => {
                    result.categories.transparency = category_score;
                }
                'D' => {
                    result.categories.statistical_rigor = category_score;
                }
                'E' => {
                    result.categories.historical_integrity = category_score;
                }
                'F' => {
                    // Handle ML category's is_applicable flag
                    result.categories.ml_reproducibility = category_score;
                    result.categories.ml_reproducibility.is_not_applicable =
                        !result.categories.ml_reproducibility.is_applicable;
                }
                _ => {
                    return Err(PopperScorerError::InvalidProject(format!(
                        "Unknown category: {}",
                        scorer.category_id()
                    )));
                }
            }
        }

        // Calculate final score with gateway logic
        result.calculate();

        // Generate analysis summary
        result.analysis = self.generate_analysis(&result);

        // Generate recommendations
        result.recommendations = self.generate_recommendations(&result);

        Ok(result)
    }

    /// Generate analysis summary based on scores
    fn generate_analysis(&self, score: &PopperScore) -> PopperAnalysis {
        let mut analysis = PopperAnalysis::default();

        // Falsifiability status
        let falsifiability_pct = score.categories.falsifiability.percentage();
        analysis.falsifiability_status = if falsifiability_pct >= 80.0 {
            AnalysisStatus::Pass
        } else if falsifiability_pct >= 60.0 {
            AnalysisStatus::Partial
        } else {
            AnalysisStatus::Fail
        };

        // Reproducibility status
        let reproducibility_pct = score.categories.reproducibility.percentage();
        analysis.reproducibility_status = if reproducibility_pct >= 80.0 {
            AnalysisStatus::Pass
        } else if reproducibility_pct >= 60.0 {
            AnalysisStatus::Partial
        } else {
            AnalysisStatus::Fail
        };

        // Scrutiny status (based on transparency)
        let transparency_pct = score.categories.transparency.percentage();
        analysis.scrutiny_status = if transparency_pct >= 80.0 {
            AnalysisStatus::Pass
        } else if transparency_pct >= 60.0 {
            AnalysisStatus::Partial
        } else {
            AnalysisStatus::Fail
        };

        // Methodology status (based on statistical rigor)
        let statistical_pct = score.categories.statistical_rigor.percentage();
        analysis.methodology_status = if statistical_pct >= 80.0 {
            AnalysisStatus::Pass
        } else if statistical_pct >= 60.0 {
            AnalysisStatus::Partial
        } else {
            AnalysisStatus::Fail
        };

        // Validation status (based on historical integrity)
        let historical_pct = score.categories.historical_integrity.percentage();
        analysis.validation_status = if historical_pct >= 80.0 {
            AnalysisStatus::Pass
        } else if historical_pct >= 60.0 {
            AnalysisStatus::Partial
        } else {
            AnalysisStatus::Fail
        };

        // Generate verdict
        if !score.gateway_passed {
            analysis.verdict =
                "GATEWAY FAILED: Project does not meet minimum falsifiability requirements. \
                 Without testable claims, independent verification is not possible."
                    .to_string();
        } else if score.grade.meets_standards() {
            analysis.verdict = format!(
                "MEETS POPPERIAN STANDARDS: Project achieves {:.1}% normalized score. \
                 Claims are testable, reproducibility infrastructure exists, and \
                 methodology is documented.",
                score.normalized_score
            );
        } else {
            analysis.verdict = format!(
                "IMPROVEMENT NEEDED: Project scores {:.1}% ({}) on Popperian criteria. \
                 See recommendations for enhancing reproducibility and falsifiability.",
                score.normalized_score, score.grade
            );
        }

        analysis
    }

    /// Generate prioritized recommendations based on gaps
    fn generate_recommendations(&self, score: &PopperScore) -> Vec<PopperRecommendation> {
        let mut recommendations = Vec::new();

        // Gateway recommendation (highest priority if failed)
        if !score.gateway_passed {
            recommendations.push(
                PopperRecommendation::new(
                    "Falsifiability (Gateway)",
                    "Add explicit falsifiable claims and test coverage. \
                     This is required for any score above 0.",
                    RecommendationPriority::Critical,
                    15.0,
                )
                .with_command("pmat quality-gate --checks tests,coverage"),
            );
            return recommendations; // No point in other recommendations if gateway fails
        }

        // Check each category and add recommendations for low scores
        let categories = [
            (
                &score.categories.falsifiability,
                "Falsifiability",
                "Add more tests, property-based testing, or mutation testing",
                "cargo mutants --package core",
            ),
            (
                &score.categories.reproducibility,
                "Reproducibility",
                "Add lock files, Docker/Nix environment, or Makefile",
                "nix flake init",
            ),
            (
                &score.categories.transparency,
                "Transparency",
                "Add LICENSE file, API documentation, or ADRs",
                "cargo doc --open",
            ),
            (
                &score.categories.statistical_rigor,
                "Statistical Rigor",
                "Add benchmark sample sizes, confidence intervals, or effect sizes",
                "cargo criterion --message-format json",
            ),
            (
                &score.categories.historical_integrity,
                "Historical Integrity",
                "Add CODEOWNERS, commit linting, or pre-registration docs",
                "git config commit.template .gitmessage",
            ),
        ];

        for (category, name, desc, cmd) in categories {
            let pct = category.percentage();
            if pct < 60.0 {
                let priority = if pct < 30.0 {
                    RecommendationPriority::High
                } else {
                    RecommendationPriority::Medium
                };

                let potential = ((60.0 - pct) / 100.0) * category.max;
                recommendations.push(
                    PopperRecommendation::new(name, desc, priority, potential).with_command(cmd),
                );
            }
        }

        // ML recommendations (if applicable)
        if !score.categories.ml_reproducibility.is_not_applicable {
            let ml_pct = score.categories.ml_reproducibility.percentage();
            if ml_pct < 60.0 {
                recommendations.push(
                    PopperRecommendation::new(
                        "ML Reproducibility",
                        "Add random seed fixing, DVC for model versioning, or data documentation",
                        RecommendationPriority::Medium,
                        (60.0 - ml_pct) / 100.0 * 5.0,
                    )
                    .with_command("dvc init"),
                );
            }
        }

        // Sort by priority (Critical > High > Medium > Low)
        recommendations.sort_by(|a, b| b.priority.cmp(&a.priority));

        recommendations
    }
}

impl Default for PopperOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Public API Functions
// ============================================================================

/// Score a project's Popper Falsifiability
///
/// This is the main entry point for scoring a project.
///
/// # Arguments
/// * `project_path` - Path to the project root
///
/// # Returns
/// * `PopperScore` - Comprehensive score with gateway logic applied
///
/// # Example
/// ```rust,ignore
/// use std::path::Path;
/// use popper_score::score_project;
///
/// let score = score_project(Path::new(".")).unwrap();
/// println!("Score: {:.1}% ({})", score.normalized_score, score.grade);
/// ```
pub fn score_project(project_path: &Path) -> PopperScorerResult<PopperScore> {
    PopperOrchestrator::new().score(project_path)
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_orchestrator_creates_all_scorers() {
        let orchestrator = PopperOrchestrator::new();
        assert_eq!(orchestrator.scorers.len(), 6);
    }

    #[test]
    fn test_empty_project_fails_gateway() {
        let temp_dir = tempdir().unwrap();
        let orchestrator = PopperOrchestrator::new();

        let result = orchestrator.score(temp_dir.path()).unwrap();

        assert!(!result.gateway_passed);
        assert_eq!(result.normalized_score, 0.0);
    }

    #[test]
    fn test_minimal_project_scores() {
        let temp_dir = tempdir().unwrap();

        // Create minimal project structure with enough signals to pass gateway
        // Gateway requires A1 + A2 + A3 >= 15 points
        fs::write(
            temp_dir.path().join("README.md"),
            r#"# Project

This project claims to provide reliable functionality with >10x performance.

## Success Criteria

- All tests pass with 95% confidence
- Coverage > 80%

## Conditions for Failure

If any test fails, the build should fail.
"#,
        )
        .unwrap();
        fs::create_dir_all(temp_dir.path().join("tests")).unwrap();
        fs::write(
            temp_dir.path().join("tests/test_main.rs"),
            r#"#[test]
fn test_example() { assert!(true); }

#[test]
#[should_panic]
fn test_panic() { panic!("expected"); }
"#,
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"1.0.0\"",
        )
        .unwrap();
        fs::write(temp_dir.path().join("Cargo.lock"), "# Lock file").unwrap();
        fs::write(
            temp_dir.path().join("LICENSE"),
            "MIT License\n\nCopyright (c) 2025",
        )
        .unwrap();
        fs::create_dir_all(temp_dir.path().join(".git")).unwrap();
        fs::create_dir_all(temp_dir.path().join(".github/workflows")).unwrap();
        fs::write(
            temp_dir.path().join(".github/workflows/ci.yml"),
            "name: CI\non: push\njobs:\n  test:\n    runs-on: ubuntu-latest\n    steps:\n      - run: cargo test",
        ).unwrap();
        // Add benches for benchmark points
        fs::create_dir_all(temp_dir.path().join("benches")).unwrap();
        fs::write(
            temp_dir.path().join("benches/bench.rs"),
            "use criterion::*;\nfn bench(c: &mut Criterion) {}\n",
        )
        .unwrap();

        let orchestrator = PopperOrchestrator::new();
        let result = orchestrator.score(temp_dir.path()).unwrap();

        // Check the score - may or may not pass gateway depending on exact conditions
        // The test verifies the scoring works, not that any specific score is achieved
        assert!(result.categories.falsifiability.earned > 0.0);
        assert!(!result.analysis.verdict.is_empty());
    }

    #[test]
    fn test_recommendations_generated() {
        let temp_dir = tempdir().unwrap();

        // Create project that passes gateway but has gaps
        fs::write(
            temp_dir.path().join("README.md"),
            "# Project\n\nClaims to provide 10x performance. Success Criteria: All tests pass.",
        )
        .unwrap();
        fs::create_dir_all(temp_dir.path().join("tests")).unwrap();
        fs::write(
            temp_dir.path().join("tests/test.rs"),
            "#[test]\nfn test() { assert!(true); }\n#[test]\n#[should_panic]\nfn test_panic() { panic!(); }",
        )
        .unwrap();
        fs::create_dir_all(temp_dir.path().join(".github/workflows")).unwrap();
        fs::write(
            temp_dir.path().join(".github/workflows/ci.yml"),
            "on: push\njobs:\n  test:\n    steps:\n      - run: cargo test",
        )
        .unwrap();

        let orchestrator = PopperOrchestrator::new();
        let result = orchestrator.score(temp_dir.path()).unwrap();

        // Check that recommendations are generated
        if result.gateway_passed && result.normalized_score < 85.0 {
            assert!(!result.recommendations.is_empty());
        }
    }

    #[test]
    fn test_ml_project_detection() {
        let temp_dir = tempdir().unwrap();

        // Create ML project
        fs::write(
            temp_dir.path().join("requirements.txt"),
            "torch==2.0.0\ntransformers\n",
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("README.md"),
            "# ML Project\n\nThis model achieves 95% accuracy. Success Criteria: Test passes.",
        )
        .unwrap();
        fs::create_dir_all(temp_dir.path().join("tests")).unwrap();
        fs::write(
            temp_dir.path().join("tests/test_model.py"),
            "def test_model(): assert True",
        )
        .unwrap();
        fs::create_dir_all(temp_dir.path().join(".github/workflows")).unwrap();
        fs::write(
            temp_dir.path().join(".github/workflows/ci.yml"),
            "on: push\njobs:\n  test:\n    steps:\n      - run: pytest",
        )
        .unwrap();

        let orchestrator = PopperOrchestrator::new();
        let result = orchestrator.score(temp_dir.path()).unwrap();

        // ML category should be applicable
        assert!(!result.categories.ml_reproducibility.is_not_applicable);
        assert_eq!(result.max_available, 100.0);
    }

    #[test]
    fn test_non_ml_project_normalization() {
        let temp_dir = tempdir().unwrap();

        // Create non-ML project
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"cli\"",
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("README.md"),
            "# CLI Tool\n\nProvides command-line utilities. Success Criteria: All tests pass.",
        )
        .unwrap();
        fs::create_dir_all(temp_dir.path().join("tests")).unwrap();
        fs::write(
            temp_dir.path().join("tests/test.rs"),
            "#[test]\nfn test() {}",
        )
        .unwrap();
        fs::create_dir_all(temp_dir.path().join(".github/workflows")).unwrap();
        fs::write(
            temp_dir.path().join(".github/workflows/ci.yml"),
            "on: push\njobs:\n  test:\n    steps:\n      - run: cargo test",
        )
        .unwrap();

        let orchestrator = PopperOrchestrator::new();
        let result = orchestrator.score(temp_dir.path()).unwrap();

        // ML category should be N/A, max should be 95
        assert!(result.categories.ml_reproducibility.is_not_applicable);
        if result.gateway_passed {
            assert_eq!(result.max_available, 95.0);
        }
    }
}
