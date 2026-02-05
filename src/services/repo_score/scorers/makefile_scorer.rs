// MakefileScorer - Category D: Build & Test Automation (25 points)
//
// Scores based on:
// - D1: Makefile Present (5 points) - Makefile exists and is valid
// - D2: Required Targets Present (15 points) - test-fast, test, lint, coverage
// - D3: Target Performance (5 points) - Fast targets complete quickly

use super::{Scorer, ScorerConfig};
use crate::services::repo_score::error::Result;
use crate::services::repo_score::models::*;
use async_trait::async_trait;
use std::path::Path;

pub struct MakefileScorer;

impl MakefileScorer {
    pub fn new() -> Self {
        Self
    }

    /// Score Makefile presence (D1: 5 points)
    async fn score_makefile_present(&self, repo_path: &Path) -> Result<SubcategoryScore> {
        let makefile_path = repo_path.join("Makefile");

        if !makefile_path.exists() {
            return Ok(SubcategoryScore {
                id: "D1".to_string(),
                name: "Makefile Present".to_string(),
                score: 0.0,
                max_score: 5.0,
                findings: vec![Finding {
                    severity: Severity::Error,
                    category: "Build".to_string(),
                    message: "Makefile not found".to_string(),
                    location: Some(makefile_path.display().to_string()),
                    impact_points: -5.0,
                }],
            });
        }

        // Check if file is not empty
        let content = tokio::fs::read_to_string(&makefile_path).await?;
        if content.trim().is_empty() {
            return Ok(SubcategoryScore {
                id: "D1".to_string(),
                name: "Makefile Present".to_string(),
                score: 1.0,
                max_score: 5.0,
                findings: vec![Finding {
                    severity: Severity::Error,
                    category: "Build".to_string(),
                    message: "Makefile is empty".to_string(),
                    location: Some(makefile_path.display().to_string()),
                    impact_points: -4.0,
                }],
            });
        }

        // Basic validation - check for target syntax
        let has_targets = content.lines().any(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && !trimmed.starts_with('#') && trimmed.contains(':')
        });

        if !has_targets {
            return Ok(SubcategoryScore {
                id: "D1".to_string(),
                name: "Makefile Present".to_string(),
                score: 2.0,
                max_score: 5.0,
                findings: vec![Finding {
                    severity: Severity::Warning,
                    category: "Build".to_string(),
                    message: "Makefile has no targets".to_string(),
                    location: Some(makefile_path.display().to_string()),
                    impact_points: -3.0,
                }],
            });
        }

        Ok(SubcategoryScore {
            id: "D1".to_string(),
            name: "Makefile Present".to_string(),
            score: 5.0,
            max_score: 5.0,
            findings: vec![Finding {
                severity: Severity::Success,
                category: "Build".to_string(),
                message: "Makefile present and valid".to_string(),
                location: Some(makefile_path.display().to_string()),
                impact_points: 0.0,
            }],
        })
    }

    /// Score required targets (D2: 15 points)
    async fn score_required_targets(&self, repo_path: &Path) -> Result<SubcategoryScore> {
        let makefile_path = repo_path.join("Makefile");

        if !makefile_path.exists() {
            return Ok(SubcategoryScore {
                id: "D2".to_string(),
                name: "Required Targets Present".to_string(),
                score: 0.0,
                max_score: 15.0,
                findings: vec![],
            });
        }

        let content = tokio::fs::read_to_string(&makefile_path).await?;

        // Required targets (each worth points)
        let required_targets = vec![
            ("test-fast", 5.0),
            ("test", 4.0),
            ("lint", 3.0),
            ("coverage", 3.0),
        ];

        let mut score = 0.0;
        let mut findings = vec![];

        for (target, points) in required_targets {
            let pattern = format!("{}:", target);
            if content.lines().any(|line| {
                let trimmed = line.trim();
                trimmed.starts_with(&pattern) || trimmed.starts_with(&format!(".PHONY: {}", target))
            }) {
                score += points;
                findings.push(Finding {
                    severity: Severity::Success,
                    category: "Build".to_string(),
                    message: format!("Target '{}' found", target),
                    location: Some("Makefile".to_string()),
                    impact_points: points,
                });
            } else {
                findings.push(Finding {
                    severity: Severity::Warning,
                    category: "Build".to_string(),
                    message: format!("Target '{}' missing", target),
                    location: Some("Makefile".to_string()),
                    impact_points: 0.0,
                });
            }
        }

        Ok(SubcategoryScore {
            id: "D2".to_string(),
            name: "Required Targets Present".to_string(),
            score,
            max_score: 15.0,
            findings,
        })
    }

    /// Score target performance (D3: 5 points)
    async fn score_target_performance(
        &self,
        repo_path: &Path,
        config: &ScorerConfig,
    ) -> Result<SubcategoryScore> {
        let makefile_path = repo_path.join("Makefile");

        if !makefile_path.exists() {
            return Ok(SubcategoryScore {
                id: "D3".to_string(),
                name: "Target Performance".to_string(),
                score: 0.0,
                max_score: 5.0,
                findings: vec![],
            });
        }

        // Skip actual performance testing if requested
        if config.skip_slow_checks {
            return Ok(SubcategoryScore {
                id: "D3".to_string(),
                name: "Target Performance".to_string(),
                score: 5.0,
                max_score: 5.0,
                findings: vec![Finding {
                    severity: Severity::Info,
                    category: "Build".to_string(),
                    message: "Performance check skipped".to_string(),
                    location: None,
                    impact_points: 0.0,
                }],
            });
        }

        // Heuristic: Check if test-fast has timeout or is marked as fast
        let content = tokio::fs::read_to_string(&makefile_path).await?;
        let content_lower = content.to_lowercase();

        let has_fast_marker = content_lower.contains("timeout")
            || content_lower.contains("test-fast")
            || content_lower.contains("--quick")
            || content_lower.contains("--fast");

        if has_fast_marker {
            Ok(SubcategoryScore {
                id: "D3".to_string(),
                name: "Target Performance".to_string(),
                score: 5.0,
                max_score: 5.0,
                findings: vec![Finding {
                    severity: Severity::Success,
                    category: "Build".to_string(),
                    message: "Makefile appears optimized for fast execution".to_string(),
                    location: Some("Makefile".to_string()),
                    impact_points: 0.0,
                }],
            })
        } else {
            Ok(SubcategoryScore {
                id: "D3".to_string(),
                name: "Target Performance".to_string(),
                score: 3.0,
                max_score: 5.0,
                findings: vec![Finding {
                    severity: Severity::Info,
                    category: "Build".to_string(),
                    message: "No performance optimization markers found".to_string(),
                    location: Some("Makefile".to_string()),
                    impact_points: 0.0,
                }],
            })
        }
    }
}

#[async_trait]
impl Scorer for MakefileScorer {
    fn category_name(&self) -> &str {
        "Build & Test Automation"
    }

    fn max_score(&self) -> f64 {
        25.0
    }

    async fn score(&self, repo_path: &Path, config: &ScorerConfig) -> Result<CategoryScore> {
        let d1 = self.score_makefile_present(repo_path).await?;
        let d2 = self.score_required_targets(repo_path).await?;
        let d3 = self.score_target_performance(repo_path, config).await?;

        let total_score = d1.score + d2.score + d3.score;

        let mut findings = d1.findings.clone();
        findings.extend(d2.findings.clone());
        findings.extend(d3.findings.clone());

        Ok(CategoryScore::new(
            total_score,
            self.max_score(),
            vec![d1, d2, d3],
            findings,
        ))
    }
}

impl Default for MakefileScorer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_temp_repo() -> TempDir {
        TempDir::new().unwrap()
    }

    fn create_makefile(repo_path: &Path, content: &str) {
        let makefile_path = repo_path.join("Makefile");
        fs::write(makefile_path, content).unwrap();
    }

    const PERFECT_MAKEFILE: &str = r#"
.PHONY: test-fast test lint coverage

test-fast:
	cargo test --lib --tests -- --test-threads=1

test:
	cargo test

lint:
	cargo clippy -- -D warnings

coverage:
	cargo llvm-cov --html
"#;

    const MINIMAL_MAKEFILE: &str = r#"
test:
	cargo test
"#;

    #[tokio::test]
    async fn test_makefile_scorer_no_makefile() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        let scorer = MakefileScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        assert_eq!(result.score, 0.0);
        assert_eq!(result.status, ScoreStatus::Fail);
    }

    #[tokio::test]
    async fn test_makefile_scorer_perfect_makefile() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        create_makefile(repo_path, PERFECT_MAKEFILE);

        let scorer = MakefileScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        // D1: 5, D2: 15 (all targets), D3: 5 (has test-fast marker) = 25
        assert_eq!(result.score, 25.0);
        assert_eq!(result.status, ScoreStatus::Pass);
    }

    #[tokio::test]
    async fn test_makefile_scorer_minimal_makefile() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        create_makefile(repo_path, MINIMAL_MAKEFILE);

        let scorer = MakefileScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        // D1: 5 (present), D2: 4 (only test), D3: 3 (no markers) = 12
        // 12/25 = 48% = Fail (<70%)
        assert!(result.score >= 11.0 && result.score <= 13.0);
        assert_eq!(result.status, ScoreStatus::Fail);
    }

    #[tokio::test]
    async fn test_makefile_scorer_empty_makefile() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        create_makefile(repo_path, "");

        let scorer = MakefileScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        // D1: 1 (empty), D2: 0, D3: 3 (no markers, skip_slow_checks=false) = 4
        assert!(result.score >= 3.0 && result.score <= 5.0);
    }

    #[tokio::test]
    async fn test_makefile_present_subcategory() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        create_makefile(repo_path, PERFECT_MAKEFILE);

        let scorer = MakefileScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        let d1 = result.subcategories.iter().find(|s| s.id == "D1").unwrap();
        assert_eq!(d1.name, "Makefile Present");
        assert_eq!(d1.score, 5.0);
    }

    #[tokio::test]
    async fn test_makefile_required_targets_subcategory() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        create_makefile(repo_path, PERFECT_MAKEFILE);

        let scorer = MakefileScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        let d2 = result.subcategories.iter().find(|s| s.id == "D2").unwrap();
        assert_eq!(d2.name, "Required Targets Present");
        assert_eq!(d2.score, 15.0);
    }

    #[tokio::test]
    async fn test_makefile_performance_subcategory() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        create_makefile(repo_path, PERFECT_MAKEFILE);

        let scorer = MakefileScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        let d3 = result.subcategories.iter().find(|s| s.id == "D3").unwrap();
        assert_eq!(d3.name, "Target Performance");
        assert_eq!(d3.score, 5.0);
    }

    #[tokio::test]
    async fn test_makefile_partial_targets() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        let partial_makefile = r#"
test-fast:
	cargo test --lib

lint:
	cargo clippy
"#;
        create_makefile(repo_path, partial_makefile);

        let scorer = MakefileScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        // D1: 5, D2: 5+3=8 (test-fast + lint), D3: 5 = 18
        assert!(result.score >= 17.0 && result.score <= 19.0);
    }

    #[tokio::test]
    async fn test_makefile_category_name() {
        let scorer = MakefileScorer::new();
        assert_eq!(scorer.category_name(), "Build & Test Automation");
        assert_eq!(scorer.max_score(), 25.0);
    }
}
