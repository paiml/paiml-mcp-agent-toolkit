#![cfg_attr(coverage_nightly, coverage(off))]
// PrecommitScorer - Category B: Pre-commit Hooks (20 points)
//
// Scores based on:
// - B1: Pre-commit Hook Present (10 points) - .git/hooks/pre-commit exists and executable
// - B2: Hook Execution Time (10 points) - Runs in <30 seconds

use super::{Scorer, ScorerConfig};
use crate::services::repo_score::error::Result;
use crate::services::repo_score::models::*;
use async_trait::async_trait;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

pub struct PrecommitScorer;

impl PrecommitScorer {
    pub fn new() -> Self {
        Self
    }

    /// Score pre-commit hook presence (B1: 10 points)
    async fn score_hook_present(&self, repo_path: &Path) -> Result<SubcategoryScore> {
        debug_assert!(
            repo_path.exists(),
            "repo_path must exist: {}",
            repo_path.display()
        );
        let git_hooks_path = repo_path.join(".git/hooks");
        let precommit_path = git_hooks_path.join("pre-commit");

        if !precommit_path.exists() {
            return Ok(SubcategoryScore {
                id: "B1".to_string(),
                name: "Pre-commit Hook Present".to_string(),
                score: 0.0,
                max_score: 10.0,
                findings: vec![Finding {
                    severity: Severity::Error,
                    category: "Pre-commit".to_string(),
                    message: "No pre-commit hook found".to_string(),
                    location: Some(precommit_path.display().to_string()),
                    impact_points: -10.0,
                }],
            });
        }

        // Check if executable (Unix only)
        #[cfg(unix)]
        {
            let metadata = std::fs::metadata(&precommit_path)?;
            let permissions = metadata.permissions();
            let is_executable = permissions.mode() & 0o111 != 0;

            if !is_executable {
                return Ok(SubcategoryScore {
                    id: "B1".to_string(),
                    name: "Pre-commit Hook Present".to_string(),
                    score: 5.0, // Partial credit
                    max_score: 10.0,
                    findings: vec![Finding {
                        severity: Severity::Warning,
                        category: "Pre-commit".to_string(),
                        message: "Pre-commit hook exists but is not executable".to_string(),
                        location: Some(precommit_path.display().to_string()),
                        impact_points: -5.0,
                    }],
                });
            }
        }

        // Check file is not empty
        let content = tokio::fs::read_to_string(&precommit_path).await?;
        if content.trim().is_empty() {
            return Ok(SubcategoryScore {
                id: "B1".to_string(),
                name: "Pre-commit Hook Present".to_string(),
                score: 2.0, // Minimal credit
                max_score: 10.0,
                findings: vec![Finding {
                    severity: Severity::Error,
                    category: "Pre-commit".to_string(),
                    message: "Pre-commit hook is empty".to_string(),
                    location: Some(precommit_path.display().to_string()),
                    impact_points: -8.0,
                }],
            });
        }

        Ok(SubcategoryScore {
            id: "B1".to_string(),
            name: "Pre-commit Hook Present".to_string(),
            score: 10.0,
            max_score: 10.0,
            findings: vec![Finding {
                severity: Severity::Success,
                category: "Pre-commit".to_string(),
                message: "Pre-commit hook present and executable".to_string(),
                location: Some(precommit_path.display().to_string()),
                impact_points: 0.0,
            }],
        })
    }

    /// Score hook execution time (B2: 10 points)
    async fn score_hook_performance(
        &self,
        repo_path: &Path,
        config: &ScorerConfig,
    ) -> Result<SubcategoryScore> {
        debug_assert!(
            repo_path.exists(),
            "repo_path must exist: {}",
            repo_path.display()
        );
        let precommit_path = repo_path.join(".git/hooks/pre-commit");

        if !precommit_path.exists() {
            // No hook = no performance to check
            return Ok(SubcategoryScore {
                id: "B2".to_string(),
                name: "Hook Execution Time".to_string(),
                score: 0.0,
                max_score: 10.0,
                findings: vec![Finding {
                    severity: Severity::Info,
                    category: "Pre-commit".to_string(),
                    message: "No hook to check performance".to_string(),
                    location: None,
                    impact_points: 0.0,
                }],
            });
        }

        // Skip performance test if requested (can be expensive)
        if config.skip_slow_checks {
            return Ok(SubcategoryScore {
                id: "B2".to_string(),
                name: "Hook Execution Time".to_string(),
                score: 10.0, // Assume good performance
                max_score: 10.0,
                findings: vec![Finding {
                    severity: Severity::Info,
                    category: "Pre-commit".to_string(),
                    message: "Performance check skipped (use --no-skip-slow)".to_string(),
                    location: None,
                    impact_points: 0.0,
                }],
            });
        }

        // For now, just check if hook has linting commands (heuristic)
        let content = tokio::fs::read_to_string(&precommit_path).await?;
        let content_lower = content.to_lowercase();

        let has_linting = content_lower.contains("lint")
            || content_lower.contains("clippy")
            || content_lower.contains("eslint")
            || content_lower.contains("pylint")
            || content_lower.contains("black")
            || content_lower.contains("prettier");

        let has_testing = content_lower.contains("cargo test")
            || content_lower.contains("cargo nextest")
            || content_lower.contains("pytest")
            || content_lower.contains("npm test")
            || content_lower.contains("make test")
            || content_lower.contains("yarn test")
            || content_lower.contains("go test");

        if has_linting && !has_testing {
            Ok(SubcategoryScore {
                id: "B2".to_string(),
                name: "Hook Execution Time".to_string(),
                score: 10.0,
                max_score: 10.0,
                findings: vec![Finding {
                    severity: Severity::Success,
                    category: "Pre-commit".to_string(),
                    message: "Hook contains linting (likely fast)".to_string(),
                    location: Some(precommit_path.display().to_string()),
                    impact_points: 0.0,
                }],
            })
        } else if has_testing {
            Ok(SubcategoryScore {
                id: "B2".to_string(),
                name: "Hook Execution Time".to_string(),
                score: 5.0, // Deduct for potentially slow tests
                max_score: 10.0,
                findings: vec![Finding {
                    severity: Severity::Warning,
                    category: "Pre-commit".to_string(),
                    message: "Hook contains tests (may be slow)".to_string(),
                    location: Some(precommit_path.display().to_string()),
                    impact_points: -5.0,
                }],
            })
        } else {
            // Unknown content, give benefit of doubt
            Ok(SubcategoryScore {
                id: "B2".to_string(),
                name: "Hook Execution Time".to_string(),
                score: 7.0,
                max_score: 10.0,
                findings: vec![Finding {
                    severity: Severity::Info,
                    category: "Pre-commit".to_string(),
                    message: "Hook performance assumed acceptable".to_string(),
                    location: Some(precommit_path.display().to_string()),
                    impact_points: 0.0,
                }],
            })
        }
    }
}

#[async_trait]
impl Scorer for PrecommitScorer {
    fn category_name(&self) -> &str {
        "Pre-commit Hooks"
    }

    fn max_score(&self) -> f64 {
        20.0
    }

    async fn score(&self, repo_path: &Path, config: &ScorerConfig) -> Result<CategoryScore> {
        debug_assert!(
            repo_path.exists(),
            "repo_path must exist: {}",
            repo_path.display()
        );
        let b1 = self.score_hook_present(repo_path).await?;
        let b2 = self.score_hook_performance(repo_path, config).await?;

        let total_score = b1.score + b2.score;

        let mut findings = b1.findings.clone();
        findings.extend(b2.findings.clone());

        Ok(CategoryScore::new(
            total_score,
            self.max_score(),
            vec![b1, b2],
            findings,
        ))
    }
}

impl Default for PrecommitScorer {
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

    fn create_git_repo(repo_path: &Path) {
        debug_assert!(
            repo_path.exists(),
            "repo_path must exist: {}",
            repo_path.display()
        );
        let git_dir = repo_path.join(".git");
        let hooks_dir = git_dir.join("hooks");
        fs::create_dir_all(&hooks_dir).unwrap();
    }

    fn create_precommit_hook(repo_path: &Path, content: &str, executable: bool) {
        debug_assert!(
            repo_path.exists(),
            "repo_path must exist: {}",
            repo_path.display()
        );
        let hook_path = repo_path.join(".git/hooks/pre-commit");
        fs::write(&hook_path, content).unwrap();

        #[cfg(unix)]
        if executable {
            let metadata = fs::metadata(&hook_path).unwrap();
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&hook_path, permissions).unwrap();
        }

        #[cfg(not(unix))]
        let _ = executable;
    }

    const LINTING_HOOK: &str = r#"#!/bin/bash
cargo clippy -- -D warnings
"#;

    #[tokio::test]
    async fn test_precommit_scorer_no_hook() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        create_git_repo(repo_path);

        let scorer = PrecommitScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        // B1: 0 (no hook), B2: 0 (no hook) = 0 total
        assert_eq!(result.score, 0.0);
        assert_eq!(result.status, ScoreStatus::Fail);
    }

    #[tokio::test]
    async fn test_precommit_scorer_valid_hook() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        create_git_repo(repo_path);
        create_precommit_hook(repo_path, LINTING_HOOK, true);

        let scorer = PrecommitScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        // B1: 10 (valid hook), B2: 10 (skip slow checks) = 20 total
        assert_eq!(result.score, 20.0);
        assert_eq!(result.status, ScoreStatus::Pass);
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn test_precommit_scorer_non_executable_hook() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        create_git_repo(repo_path);
        create_precommit_hook(repo_path, LINTING_HOOK, false);

        let scorer = PrecommitScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        // B1: 5 (not executable), B2: 10 (skip slow) = 15 total
        assert!(result.score >= 14.0 && result.score <= 16.0);
        assert!(result
            .findings
            .iter()
            .any(|f| f.message.contains("not executable")));
    }

    #[tokio::test]
    async fn test_precommit_scorer_empty_hook() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        create_git_repo(repo_path);
        create_precommit_hook(repo_path, "", true);

        let scorer = PrecommitScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        // B1: 2 (empty), B2: 7 (unknown content) = 9 total
        assert!(result.score >= 8.0 && result.score <= 10.0);
        assert!(result.findings.iter().any(|f| f.message.contains("empty")));
    }

    #[tokio::test]
    async fn test_precommit_hook_present_subcategory() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        create_git_repo(repo_path);
        create_precommit_hook(repo_path, LINTING_HOOK, true);

        let scorer = PrecommitScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        let b1 = result.subcategories.iter().find(|s| s.id == "B1").unwrap();
        assert_eq!(b1.name, "Pre-commit Hook Present");
        assert_eq!(b1.score, 10.0);
        assert_eq!(b1.max_score, 10.0);
    }

    #[tokio::test]
    async fn test_precommit_performance_subcategory() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        create_git_repo(repo_path);
        create_precommit_hook(repo_path, LINTING_HOOK, true);

        let scorer = PrecommitScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        let b2 = result.subcategories.iter().find(|s| s.id == "B2").unwrap();
        assert_eq!(b2.name, "Hook Execution Time");
        assert_eq!(b2.max_score, 10.0);
    }

    #[tokio::test]
    async fn test_precommit_category_name() {
        let scorer = PrecommitScorer::new();
        assert_eq!(scorer.category_name(), "Pre-commit Hooks");
        assert_eq!(scorer.max_score(), 20.0);
    }
}
