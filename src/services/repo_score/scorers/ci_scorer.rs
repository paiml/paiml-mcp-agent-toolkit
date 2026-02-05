// CiScorer - Category E: Continuous Integration (20 points)
//
// Scores based on:
// - E1: CI Workflows Present (6 points) - GitHub Actions workflows exist
// - E2: Workflows Configured Properly (6 points) - Valid YAML with standard jobs
// - E3: Advanced CI Features (8 points) - Coverage, security, caching, matrix builds
//
// Issue #72: Enhanced feedback with actionable recommendations

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

    /// Score CI workflows presence (E1: 6 points)
    async fn score_workflows_present(&self, repo_path: &Path) -> Result<SubcategoryScore> {
        let workflows_dir = repo_path.join(".github/workflows");

        if !workflows_dir.exists() {
            return Ok(SubcategoryScore {
                id: "E1".to_string(),
                name: "CI Workflows Present".to_string(),
                score: 0.0,
                max_score: 6.0,
                findings: vec![Finding {
                    severity: Severity::Error,
                    category: "CI".to_string(),
                    message:
                        "Missing: Create .github/workflows/ directory with CI workflow (+6 pts)"
                            .to_string(),
                    location: Some(workflows_dir.display().to_string()),
                    impact_points: -6.0,
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
                score: 1.0,
                max_score: 6.0,
                findings: vec![Finding {
                    severity: Severity::Warning,
                    category: "CI".to_string(),
                    message: "Missing: Add workflow files to .github/workflows/ (+5 pts)"
                        .to_string(),
                    location: Some(workflows_dir.display().to_string()),
                    impact_points: -5.0,
                }],
            });
        }

        // Score: 2 pts for 1 workflow, +2 pts for 2+ workflows, +2 pts for 3+ workflows
        let score = match workflow_files.len() {
            1 => 2.0,
            2 => 4.0,
            _ => 6.0,
        };

        let mut findings = vec![];
        for workflow_path in &workflow_files {
            findings.push(Finding {
                severity: Severity::Success,
                category: "CI".to_string(),
                message: format!(
                    "✓ Workflow: {} (+2 pts)",
                    workflow_path
                        .file_name()
                        .expect("internal error")
                        .to_string_lossy()
                ),
                location: Some(workflow_path.display().to_string()),
                impact_points: 2.0,
            });
        }

        Ok(SubcategoryScore {
            id: "E1".to_string(),
            name: "CI Workflows Present".to_string(),
            score,
            max_score: 6.0,
            findings,
        })
    }

    /// Score workflow configuration (E2: 6 points)
    async fn score_workflows_configured(&self, repo_path: &Path) -> Result<SubcategoryScore> {
        let workflows_dir = repo_path.join(".github/workflows");

        if !workflows_dir.exists() {
            return Ok(SubcategoryScore {
                id: "E2".to_string(),
                name: "Workflows Configured Properly".to_string(),
                score: 0.0,
                max_score: 6.0,
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
                max_score: 6.0,
                findings: vec![],
            });
        }

        let mut total_score: f64 = 0.0;
        let mut findings = vec![];
        let mut has_testing = false;
        let mut has_linting = false;

        for workflow_path in &workflow_files {
            let content = match tokio::fs::read_to_string(workflow_path).await {
                Ok(c) => c,
                Err(_) => continue,
            };

            // Basic YAML validation - check for required keys
            let has_name = content.contains("name:");
            let has_on = content.contains("on:");
            let has_jobs = content.contains("jobs:");
            let workflow_name = workflow_path
                .file_name()
                .expect("internal error")
                .to_string_lossy();

            if has_name && has_on && has_jobs {
                total_score += 2.0;
                findings.push(Finding {
                    severity: Severity::Success,
                    category: "CI".to_string(),
                    message: format!("✓ Valid workflow structure: {} (+2 pts)", workflow_name),
                    location: Some(workflow_path.display().to_string()),
                    impact_points: 2.0,
                });
            } else {
                let mut missing = vec![];
                if !has_name {
                    missing.push("name");
                }
                if !has_on {
                    missing.push("on");
                }
                if !has_jobs {
                    missing.push("jobs");
                }
                findings.push(Finding {
                    severity: Severity::Warning,
                    category: "CI".to_string(),
                    message: format!(
                        "Incomplete: {} missing {} (+2 pts if fixed)",
                        workflow_name,
                        missing.join(", ")
                    ),
                    location: Some(workflow_path.display().to_string()),
                    impact_points: 0.0,
                });
            }

            // Check for common CI patterns
            let content_lower = content.to_lowercase();
            if content_lower.contains("test")
                || content_lower.contains("cargo test")
                || content_lower.contains("npm test")
            {
                has_testing = true;
            }
            if content_lower.contains("lint")
                || content_lower.contains("clippy")
                || content_lower.contains("eslint")
            {
                has_linting = true;
            }
        }

        // Bonus for testing and linting
        if has_testing {
            total_score += 1.0;
            findings.push(Finding {
                severity: Severity::Success,
                category: "CI".to_string(),
                message: "✓ Testing step detected (+1 pt)".to_string(),
                location: None,
                impact_points: 1.0,
            });
        } else {
            findings.push(Finding {
                severity: Severity::Info,
                category: "CI".to_string(),
                message: "Missing: Add testing step (cargo test, npm test) (+1 pt)".to_string(),
                location: None,
                impact_points: 0.0,
            });
        }

        if has_linting {
            total_score += 1.0;
            findings.push(Finding {
                severity: Severity::Success,
                category: "CI".to_string(),
                message: "✓ Linting step detected (+1 pt)".to_string(),
                location: None,
                impact_points: 1.0,
            });
        } else {
            findings.push(Finding {
                severity: Severity::Info,
                category: "CI".to_string(),
                message: "Missing: Add linting step (clippy, eslint) (+1 pt)".to_string(),
                location: None,
                impact_points: 0.0,
            });
        }

        let score = total_score.min(6.0);

        Ok(SubcategoryScore {
            id: "E2".to_string(),
            name: "Workflows Configured Properly".to_string(),
            score,
            max_score: 6.0,
            findings,
        })
    }

    /// Score advanced CI features (E3: 8 points)
    /// Issue #72: Provides actionable feedback for advanced CI improvements
    async fn score_advanced_features(&self, repo_path: &Path) -> Result<SubcategoryScore> {
        let workflows_dir = repo_path.join(".github/workflows");
        let mut total_score: f64 = 0.0;
        let mut findings = vec![];

        if !workflows_dir.exists() {
            return Ok(SubcategoryScore {
                id: "E3".to_string(),
                name: "Advanced CI Features".to_string(),
                score: 0.0,
                max_score: 8.0,
                findings: vec![Finding {
                    severity: Severity::Info,
                    category: "CI".to_string(),
                    message:
                        "Add workflows first to unlock advanced CI features (+8 pts available)"
                            .to_string(),
                    location: None,
                    impact_points: 0.0,
                }],
            });
        }

        // Collect all workflow content
        let mut all_content = String::new();
        for entry in WalkDir::new(&workflows_dir)
            .max_depth(1)
            .into_iter()
            .flatten()
        {
            if entry.file_type().is_file() {
                let path = entry.path();
                let extension = path.extension().and_then(|s| s.to_str());
                if extension == Some("yml") || extension == Some("yaml") {
                    if let Ok(content) = tokio::fs::read_to_string(path).await {
                        all_content.push_str(&content.to_lowercase());
                    }
                }
            }
        }

        // Check for coverage reporting (2 pts)
        let has_coverage = all_content.contains("codecov")
            || all_content.contains("coveralls")
            || all_content.contains("llvm-cov")
            || all_content.contains("coverage");
        if has_coverage {
            total_score += 2.0;
            findings.push(Finding {
                severity: Severity::Success,
                category: "CI".to_string(),
                message: "✓ Code coverage reporting (+2 pts)".to_string(),
                location: None,
                impact_points: 2.0,
            });
        } else {
            findings.push(Finding {
                severity: Severity::Info,
                category: "CI".to_string(),
                message: "Missing: Add coverage reporting (codecov, coveralls) (+2 pts)"
                    .to_string(),
                location: None,
                impact_points: 0.0,
            });
        }

        // Check for security scanning (2 pts)
        let has_security = all_content.contains("security")
            || all_content.contains("audit")
            || all_content.contains("trivy")
            || all_content.contains("snyk")
            || all_content.contains("codeql")
            || all_content.contains("dependabot");
        if has_security {
            total_score += 2.0;
            findings.push(Finding {
                severity: Severity::Success,
                category: "CI".to_string(),
                message: "✓ Security scanning enabled (+2 pts)".to_string(),
                location: None,
                impact_points: 2.0,
            });
        } else {
            findings.push(Finding {
                severity: Severity::Info,
                category: "CI".to_string(),
                message: "Missing: Add security scanning (cargo audit, CodeQL, Trivy) (+2 pts)"
                    .to_string(),
                location: None,
                impact_points: 0.0,
            });
        }

        // Check for caching (2 pts)
        let has_caching = all_content.contains("cache") || all_content.contains("actions/cache");
        if has_caching {
            total_score += 2.0;
            findings.push(Finding {
                severity: Severity::Success,
                category: "CI".to_string(),
                message: "✓ Build caching configured (+2 pts)".to_string(),
                location: None,
                impact_points: 2.0,
            });
        } else {
            findings.push(Finding {
                severity: Severity::Info,
                category: "CI".to_string(),
                message: "Missing: Add caching (actions/cache) for faster builds (+2 pts)"
                    .to_string(),
                location: None,
                impact_points: 0.0,
            });
        }

        // Check for matrix builds (2 pts)
        let has_matrix = all_content.contains("matrix:") || all_content.contains("strategy:");
        if has_matrix {
            total_score += 2.0;
            findings.push(Finding {
                severity: Severity::Success,
                category: "CI".to_string(),
                message: "✓ Matrix/strategy builds configured (+2 pts)".to_string(),
                location: None,
                impact_points: 2.0,
            });
        } else {
            findings.push(Finding {
                severity: Severity::Info,
                category: "CI".to_string(),
                message: "Missing: Add matrix builds for multi-platform testing (+2 pts)"
                    .to_string(),
                location: None,
                impact_points: 0.0,
            });
        }

        Ok(SubcategoryScore {
            id: "E3".to_string(),
            name: "Advanced CI Features".to_string(),
            score: total_score.min(8.0),
            max_score: 8.0,
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
        let e3 = self.score_advanced_features(repo_path).await?;

        let total_score = e1.score + e2.score + e3.score;

        let mut findings = e1.findings.clone();
        findings.extend(e2.findings.clone());
        findings.extend(e3.findings.clone());

        Ok(CategoryScore::new(
            total_score,
            self.max_score(),
            vec![e1, e2, e3],
            findings,
        ))
    }
}

impl Default for CiScorer {
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
        TempDir::new().expect("internal error")
    }

    fn create_workflow(repo_path: &Path, name: &str, content: &str) {
        let workflows_dir = repo_path.join(".github/workflows");
        fs::create_dir_all(&workflows_dir).expect("internal error");
        let workflow_path = workflows_dir.join(name);
        fs::write(workflow_path, content).expect("internal error");
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

        let result = scorer
            .score(repo_path, &config)
            .await
            .expect("internal error");

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

        let result = scorer
            .score(repo_path, &config)
            .await
            .expect("internal error");

        // E1: 2 (1 workflow), E2: 4 (valid + test + lint), E3: 0 (no advanced) = 6
        // With the improved workflow, should get some E3 points too
        assert!(result.score >= 5.0 && result.score <= 8.0);
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

        let result = scorer
            .score(repo_path, &config)
            .await
            .expect("internal error");

        // E1: 6 (3 workflows), E2: 6 (maxed out), E3: 0 (no advanced) = 12
        assert!(result.score >= 10.0 && result.score <= 14.0);
    }

    #[tokio::test]
    async fn test_ci_scorer_minimal_workflow() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        create_workflow(repo_path, "build.yml", MINIMAL_WORKFLOW);

        let scorer = CiScorer::new();
        let config = ScorerConfig::default();

        let result = scorer
            .score(repo_path, &config)
            .await
            .expect("internal error");

        // E1: 2 (1 workflow), E2: 2 (valid only), E3: 0 = 4
        assert!(result.score >= 3.0 && result.score <= 5.0);
    }

    #[tokio::test]
    async fn test_ci_workflows_present_subcategory() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        create_workflow(repo_path, "ci.yml", PERFECT_WORKFLOW);

        let scorer = CiScorer::new();
        let config = ScorerConfig::default();

        let result = scorer
            .score(repo_path, &config)
            .await
            .expect("internal error");

        let e1 = result
            .subcategories
            .iter()
            .find(|s| s.id == "E1")
            .expect("internal error");
        assert_eq!(e1.name, "CI Workflows Present");
        assert_eq!(e1.max_score, 6.0);
        assert!(e1.score >= 1.0 && e1.score <= 3.0); // 1 workflow = 2 pts
    }

    #[tokio::test]
    async fn test_ci_workflows_configured_subcategory() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        create_workflow(repo_path, "ci.yml", PERFECT_WORKFLOW);

        let scorer = CiScorer::new();
        let config = ScorerConfig::default();

        let result = scorer
            .score(repo_path, &config)
            .await
            .expect("internal error");

        let e2 = result
            .subcategories
            .iter()
            .find(|s| s.id == "E2")
            .expect("internal error");
        assert_eq!(e2.name, "Workflows Configured Properly");
        assert_eq!(e2.max_score, 6.0);
        assert!(e2.score >= 3.0 && e2.score <= 5.0); // valid + test + lint
    }

    #[tokio::test]
    async fn test_ci_advanced_features_subcategory() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        create_workflow(repo_path, "ci.yml", PERFECT_WORKFLOW);

        let scorer = CiScorer::new();
        let config = ScorerConfig::default();

        let result = scorer
            .score(repo_path, &config)
            .await
            .expect("internal error");

        let e3 = result
            .subcategories
            .iter()
            .find(|s| s.id == "E3")
            .expect("internal error");
        assert_eq!(e3.name, "Advanced CI Features");
        assert_eq!(e3.max_score, 8.0);
        // Should have findings telling us what's missing
        assert!(!e3.findings.is_empty());
    }

    #[tokio::test]
    async fn test_ci_empty_workflows_dir() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        // Create directory but no files
        let workflows_dir = repo_path.join(".github/workflows");
        fs::create_dir_all(&workflows_dir).expect("internal error");

        let scorer = CiScorer::new();
        let config = ScorerConfig::default();

        let result = scorer
            .score(repo_path, &config)
            .await
            .expect("internal error");

        // E1: 1 (dir exists but empty), E2: 0, E3: 0 = 1
        assert!(result.score >= 0.0 && result.score <= 2.0);
    }

    #[tokio::test]
    async fn test_ci_category_name() {
        let scorer = CiScorer::new();
        assert_eq!(scorer.category_name(), "Continuous Integration");
        assert_eq!(scorer.max_score(), 20.0);
    }
}
