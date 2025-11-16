// CiScorer - Category E: Continuous Integration (20 points)
//
// Scores based on:
// - E1: CI Workflows Present (10 points) - GitHub Actions workflows exist
// - E2: Workflows Configured Properly (10 points) - Valid YAML with standard jobs

use super::{Scorer, ScorerConfig};
use crate::services::repo_score::error::Result;
use crate::services::repo_score::models::*;
use async_trait::async_trait;
use std::path::Path;
use walkdir::WalkDir;

pub struct CiScorer;

impl CiScorer {
    pub fn new() -> Self {
        Self
    }

    /// Score CI workflows presence (E1: 10 points)
    async fn score_workflows_present(&self, repo_path: &Path) -> Result<SubcategoryScore> {
        let workflows_dir = repo_path.join(".github/workflows");

        if !workflows_dir.exists() {
            return Ok(SubcategoryScore {
                id: "E1".to_string(),
                name: "CI Workflows Present".to_string(),
                score: 0.0,
                max_score: 10.0,
                findings: vec![Finding {
                    severity: Severity::Error,
                    category: "CI".to_string(),
                    message: "No .github/workflows directory found".to_string(),
                    location: Some(workflows_dir.display().to_string()),
                    impact_points: -10.0,
                }],
            });
        }

        // Find all YAML workflow files
        let mut workflow_files = vec![];
        for entry in WalkDir::new(&workflows_dir)
            .max_depth(1)
            .into_iter()
            .flatten()
        {
            if entry.file_type().is_file() {
                let path = entry.path();
                let extension = path.extension().and_then(|s| s.to_str());
                if extension == Some("yml") || extension == Some("yaml") {
                    workflow_files.push(path.to_path_buf());
                }
            }
        }

        if workflow_files.is_empty() {
            return Ok(SubcategoryScore {
                id: "E1".to_string(),
                name: "CI Workflows Present".to_string(),
                score: 2.0,
                max_score: 10.0,
                findings: vec![Finding {
                    severity: Severity::Error,
                    category: "CI".to_string(),
                    message: ".github/workflows exists but no workflow files found".to_string(),
                    location: Some(workflows_dir.display().to_string()),
                    impact_points: -8.0,
                }],
            });
        }

        // Score based on number of workflows (more = better, up to 10 points)
        let score = (workflow_files.len() as f64 * 3.0).min(10.0);

        let mut findings = vec![];
        for workflow_path in &workflow_files {
            findings.push(Finding {
                severity: Severity::Success,
                category: "CI".to_string(),
                message: format!(
                    "Workflow found: {}",
                    workflow_path.file_name().unwrap().to_string_lossy()
                ),
                location: Some(workflow_path.display().to_string()),
                impact_points: 3.0,
            });
        }

        Ok(SubcategoryScore {
            id: "E1".to_string(),
            name: "CI Workflows Present".to_string(),
            score,
            max_score: 10.0,
            findings,
        })
    }

    /// Score workflow configuration (E2: 10 points)
    async fn score_workflows_configured(&self, repo_path: &Path) -> Result<SubcategoryScore> {
        let workflows_dir = repo_path.join(".github/workflows");

        if !workflows_dir.exists() {
            return Ok(SubcategoryScore {
                id: "E2".to_string(),
                name: "Workflows Configured Properly".to_string(),
                score: 0.0,
                max_score: 10.0,
                findings: vec![],
            });
        }

        // Find workflow files
        let mut workflow_files = vec![];
        for entry in WalkDir::new(&workflows_dir)
            .max_depth(1)
            .into_iter()
            .flatten()
        {
            if entry.file_type().is_file() {
                let path = entry.path();
                let extension = path.extension().and_then(|s| s.to_str());
                if extension == Some("yml") || extension == Some("yaml") {
                    workflow_files.push(path.to_path_buf());
                }
            }
        }

        if workflow_files.is_empty() {
            return Ok(SubcategoryScore {
                id: "E2".to_string(),
                name: "Workflows Configured Properly".to_string(),
                score: 0.0,
                max_score: 10.0,
                findings: vec![],
            });
        }

        let mut total_score: f64 = 0.0;
        let mut findings = vec![];

        for workflow_path in &workflow_files {
            let content = match tokio::fs::read_to_string(workflow_path).await {
                Ok(c) => c,
                Err(_) => continue,
            };

            // Basic YAML validation - check for required keys
            let has_name = content.contains("name:");
            let has_on = content.contains("on:");
            let has_jobs = content.contains("jobs:");

            if has_name && has_on && has_jobs {
                total_score += 3.0;
                findings.push(Finding {
                    severity: Severity::Success,
                    category: "CI".to_string(),
                    message: format!(
                        "Workflow properly configured: {}",
                        workflow_path.file_name().unwrap().to_string_lossy()
                    ),
                    location: Some(workflow_path.display().to_string()),
                    impact_points: 3.0,
                });
            } else {
                findings.push(Finding {
                    severity: Severity::Warning,
                    category: "CI".to_string(),
                    message: format!(
                        "Workflow incomplete: {}",
                        workflow_path.file_name().unwrap().to_string_lossy()
                    ),
                    location: Some(workflow_path.display().to_string()),
                    impact_points: 0.0,
                });
            }

            // Bonus: Check for common CI patterns
            let content_lower = content.to_lowercase();
            let has_testing = content_lower.contains("test")
                || content_lower.contains("cargo test")
                || content_lower.contains("npm test");
            let has_linting = content_lower.contains("lint")
                || content_lower.contains("clippy")
                || content_lower.contains("eslint");

            if has_testing {
                total_score += 1.0;
            }
            if has_linting {
                total_score += 1.0;
            }
        }

        let score = total_score.min(10.0);

        Ok(SubcategoryScore {
            id: "E2".to_string(),
            name: "Workflows Configured Properly".to_string(),
            score,
            max_score: 10.0,
            findings,
        })
    }
}

#[async_trait]
impl Scorer for CiScorer {
    fn category_name(&self) -> &str {
        "Continuous Integration"
    }

    fn max_score(&self) -> f64 {
        20.0
    }

    async fn score(&self, repo_path: &Path, _config: &ScorerConfig) -> Result<CategoryScore> {
        let e1 = self.score_workflows_present(repo_path).await?;
        let e2 = self.score_workflows_configured(repo_path).await?;

        let total_score = e1.score + e2.score;

        let mut findings = e1.findings.clone();
        findings.extend(e2.findings.clone());

        Ok(CategoryScore::new(
            total_score,
            self.max_score(),
            vec![e1, e2],
            findings,
        ))
    }
}

impl Default for CiScorer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_temp_repo() -> TempDir {
        TempDir::new().unwrap()
    }

    fn create_workflow(repo_path: &Path, name: &str, content: &str) {
        let workflows_dir = repo_path.join(".github/workflows");
        fs::create_dir_all(&workflows_dir).unwrap();
        let workflow_path = workflows_dir.join(name);
        fs::write(workflow_path, content).unwrap();
    }

    const PERFECT_WORKFLOW: &str = r#"
name: CI

on:
  push:
    branches: [ main, master ]
  pull_request:
    branches: [ main, master ]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run tests
        run: cargo test
      - name: Run clippy
        run: cargo clippy -- -D warnings
"#;

    const MINIMAL_WORKFLOW: &str = r#"
name: Build

on: push

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo "Hello"
"#;

    #[tokio::test]
    async fn test_ci_scorer_no_workflows() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        let scorer = CiScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        assert_eq!(result.score, 0.0);
        assert_eq!(result.status, ScoreStatus::Fail);
    }

    #[tokio::test]
    async fn test_ci_scorer_perfect_workflow() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        create_workflow(repo_path, "ci.yml", PERFECT_WORKFLOW);

        let scorer = CiScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        // E1: 3 (1 workflow), E2: 5 (valid + test + lint) = 8
        // 8/20 = 40% = Fail (<70%)
        assert!(result.score >= 7.0 && result.score <= 9.0);
        assert_eq!(result.status, ScoreStatus::Fail);
    }

    #[tokio::test]
    async fn test_ci_scorer_multiple_workflows() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        create_workflow(repo_path, "ci.yml", PERFECT_WORKFLOW);
        create_workflow(repo_path, "release.yml", MINIMAL_WORKFLOW);
        create_workflow(repo_path, "lint.yml", MINIMAL_WORKFLOW);

        let scorer = CiScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        // E1: 9 (3 workflows × 3), E2: 10 (maxed out) = 19
        assert!(result.score >= 18.0 && result.score <= 20.0);
        assert_eq!(result.status, ScoreStatus::Pass);
    }

    #[tokio::test]
    async fn test_ci_scorer_minimal_workflow() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        create_workflow(repo_path, "build.yml", MINIMAL_WORKFLOW);

        let scorer = CiScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        // E1: 3 (1 workflow), E2: 3 (valid only) = 6
        assert!(result.score >= 5.0 && result.score <= 7.0);
    }

    #[tokio::test]
    async fn test_ci_workflows_present_subcategory() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        create_workflow(repo_path, "ci.yml", PERFECT_WORKFLOW);

        let scorer = CiScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        let e1 = result.subcategories.iter().find(|s| s.id == "E1").unwrap();
        assert_eq!(e1.name, "CI Workflows Present");
        assert!(e1.score >= 2.0 && e1.score <= 4.0);
    }

    #[tokio::test]
    async fn test_ci_workflows_configured_subcategory() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        create_workflow(repo_path, "ci.yml", PERFECT_WORKFLOW);

        let scorer = CiScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        let e2 = result.subcategories.iter().find(|s| s.id == "E2").unwrap();
        assert_eq!(e2.name, "Workflows Configured Properly");
        assert!(e2.score >= 4.0 && e2.score <= 6.0);
    }

    #[tokio::test]
    async fn test_ci_empty_workflows_dir() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        // Create directory but no files
        let workflows_dir = repo_path.join(".github/workflows");
        fs::create_dir_all(&workflows_dir).unwrap();

        let scorer = CiScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        // E1: 2 (dir exists but empty), E2: 0 = 2
        assert!(result.score >= 1.0 && result.score <= 3.0);
    }

    #[tokio::test]
    async fn test_ci_category_name() {
        let scorer = CiScorer::new();
        assert_eq!(scorer.category_name(), "Continuous Integration");
        assert_eq!(scorer.max_score(), 20.0);
    }
}
