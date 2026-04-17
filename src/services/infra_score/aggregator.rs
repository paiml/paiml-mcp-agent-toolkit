#![cfg_attr(coverage_nightly, coverage(off))]
//! Infra Score Aggregator
//!
//! Runs all 5 scorers and produces the final InfraScore.

use crate::services::infra_score::models::*;
use crate::services::infra_score::scorers::*;
use std::path::Path;
use std::time::Instant;

/// Infra score aggregator.
pub struct InfraScoreAggregator;

impl InfraScoreAggregator {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self
    }

    /// Aggregate all infra scores for a repository
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub async fn aggregate(&self, repo_path: &Path) -> anyhow::Result<InfraScore> {
        let start = Instant::now();

        // Run all 5 base scorers + 1 bonus scorer
        let wa_scorer = WorkflowArchitectureScorer::new();
        let br_scorer = BuildReliabilityScorer::new();
        let qp_scorer = QualityPipelineScorer::new();
        let dr_scorer = DeploymentReleaseScorer::new();
        let sc_scorer = SupplyChainScorer::new();
        let pv_scorer = ProvableContractsScorer::new();

        let workflow_architecture = wa_scorer.score(repo_path).await?;
        let build_reliability = br_scorer.score(repo_path).await?;
        let quality_pipeline = qp_scorer.score(repo_path).await?;
        let deployment_release = dr_scorer.score(repo_path).await?;
        let supply_chain = sc_scorer.score(repo_path).await?;
        let provable_contracts = pv_scorer.score(repo_path).await?;

        let categories = InfraCategoryScores {
            workflow_architecture,
            build_reliability,
            quality_pipeline,
            deployment_release,
            supply_chain,
            provable_contracts,
        };

        let total_score = categories.total();
        let grade = InfraGrade::from_score(total_score);
        let auto_fail = grade.is_auto_fail();

        // Generate recommendations for failed checks
        let recommendations = self.generate_recommendations(&categories);

        // Build metadata
        let mut metadata = InfraScoreMetadata::new(repo_path.to_path_buf());
        metadata.execution_time_ms = start.elapsed().as_millis() as u64;

        // Try to get git context
        if let Ok(branch) = self.get_git_branch(repo_path) {
            metadata.git_branch = Some(branch);
        }
        if let Ok(commit) = self.get_git_commit(repo_path) {
            metadata.git_commit = Some(commit);
        }

        Ok(InfraScore {
            total_score,
            grade,
            auto_fail,
            categories,
            recommendations,
            metadata,
        })
    }

    /// Generate recommendations from failed checks across all categories
    fn generate_recommendations(
        &self,
        categories: &InfraCategoryScores,
    ) -> Vec<InfraRecommendation> {
        let mut recs = Vec::new();

        self.recommend_from_category(&categories.workflow_architecture, &mut recs);
        self.recommend_from_category(&categories.build_reliability, &mut recs);
        self.recommend_from_category(&categories.quality_pipeline, &mut recs);
        self.recommend_from_category(&categories.deployment_release, &mut recs);
        self.recommend_from_category(&categories.supply_chain, &mut recs);

        // Sort by priority descending (Critical first)
        recs.sort_by(|a, b| b.priority.cmp(&a.priority));
        recs
    }

    fn recommend_from_category(
        &self,
        category: &InfraCategoryScore,
        recs: &mut Vec<InfraRecommendation>,
    ) {
        for check in &category.checks {
            if !check.passed {
                let priority = if check.max_score >= 5.0 {
                    InfraPriority::Critical
                } else if check.max_score >= 3.0 {
                    InfraPriority::High
                } else {
                    InfraPriority::Medium
                };

                recs.push(InfraRecommendation {
                    priority,
                    check_id: check.id.clone(),
                    title: format!("Fix {}: {}", check.id, check.name),
                    description: check
                        .evidence
                        .first()
                        .cloned()
                        .unwrap_or_else(|| format!("{} check failed", check.id)),
                    impact_points: check.max_score - check.score,
                    estimated_effort: estimate_effort(check.max_score),
                });
            }
        }
    }

    fn get_git_branch(&self, repo_path: &Path) -> anyhow::Result<String> {
        let git_head = repo_path.join(".git/HEAD");
        if git_head.exists() {
            let content = std::fs::read_to_string(git_head)?;
            if let Some(branch) = content.strip_prefix("ref: refs/heads/") {
                return Ok(branch.trim().to_string());
            }
        }
        Ok("unknown".to_string())
    }

    fn get_git_commit(&self, repo_path: &Path) -> anyhow::Result<String> {
        let git_head = repo_path.join(".git/HEAD");
        if git_head.exists() {
            let content = std::fs::read_to_string(git_head)?;
            if content.starts_with("ref:") {
                if let Some(ref_path) = content.strip_prefix("ref: ") {
                    let ref_file = repo_path.join(".git").join(ref_path.trim());
                    if ref_file.exists() {
                        let commit = std::fs::read_to_string(ref_file)?;
                        return Ok(commit.trim().to_string());
                    }
                }
            } else {
                return Ok(content.trim().to_string());
            }
        }
        Ok("unknown".to_string())
    }
}

impl Default for InfraScoreAggregator {
    fn default() -> Self {
        Self::new()
    }
}

/// Estimate effort based on check point value
fn estimate_effort(max_score: f64) -> String {
    if max_score >= 5.0 {
        "30 minutes".to_string()
    } else if max_score >= 3.0 {
        "15 minutes".to_string()
    } else {
        "5 minutes".to_string()
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_file(repo_path: &Path, relative_path: &str, content: &str) {
        let file_path = repo_path.join(relative_path);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(file_path, content).unwrap();
    }

    #[tokio::test]
    async fn test_empty_repo_auto_fail() {
        let tmp = TempDir::new().unwrap();
        let aggregator = InfraScoreAggregator::new();
        let result = aggregator.aggregate(tmp.path()).await.unwrap();

        assert!(result.auto_fail);
        assert!(result.total_score < 90.0);
        assert!(!result.recommendations.is_empty());
    }

    #[tokio::test]
    async fn test_empty_repo_grade_f() {
        let tmp = TempDir::new().unwrap();
        let aggregator = InfraScoreAggregator::new();
        let result = aggregator.aggregate(tmp.path()).await.unwrap();

        // With no workflows at all, should be very low
        assert!(matches!(result.grade, InfraGrade::F | InfraGrade::D));
    }

    #[tokio::test]
    async fn test_perfect_repo() {
        let tmp = TempDir::new().unwrap();
        let repo = tmp.path();

        // Create perfect CI workflow
        create_file(
            repo,
            ".github/workflows/ci.yml",
            r#"name: CI
on:
  push:
    branches: [main]
  pull_request:
    branches: [main]
  workflow_dispatch:

concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true

env:
  CARGO_INCREMENTAL: 0

jobs:
  test:
    uses: paiml/.github/.github/workflows/unified-gate.yml@main
  lint:
    runs-on: [self-hosted, linux]
    timeout-minutes: 30
    steps:
      - uses: actions/checkout@v4
      - uses: mozilla-actions/sccache-action@v0.0.4
      - run: cargo test --locked --all-targets
      - run: cargo clippy -- -D warnings
      - run: cargo llvm-cov --html
      - run: cargo audit
      - run: cargo fmt -- --check
  gate:
    needs: [test, lint]
    if: always()
    runs-on: ubuntu-latest
    steps:
      - run: echo "gate passed"
"#,
        );

        // Create nightly workflow
        create_file(
            repo,
            ".github/workflows/nightly.yml",
            r#"name: Nightly
on:
  schedule:
    - cron: '0 4 * * *'
  workflow_dispatch:
jobs:
  build:
    strategy:
      matrix:
        target: [x86_64-unknown-linux-gnu, x86_64-apple-darwin, x86_64-pc-windows-msvc]
    runs-on: ubuntu-latest
    steps:
      - uses: softprops/action-gh-release@v2
      - uses: sigstore/cosign-installer@v3
      - run: gpg --verify
"#,
        );

        // Create supporting files
        create_file(repo, ".github/CODEOWNERS", "* @org/team");
        create_file(repo, ".github/dependabot.yml", "version: 2");
        create_file(
            repo,
            "Cargo.toml",
            "[package]\nname = \"test-crate\"\nversion = \"1.2.3\"",
        );

        let aggregator = InfraScoreAggregator::new();
        let result = aggregator.aggregate(repo.as_ref()).await.unwrap();

        assert!(
            !result.auto_fail,
            "Score {:.1} should not auto-fail",
            result.total_score
        );
        assert!(
            result.total_score >= 90.0,
            "Expected >=90, got {:.1}",
            result.total_score
        );
    }

    #[tokio::test]
    async fn test_recommendations_sorted_by_priority() {
        let tmp = TempDir::new().unwrap();
        let aggregator = InfraScoreAggregator::new();
        let result = aggregator.aggregate(tmp.path()).await.unwrap();

        if result.recommendations.len() > 1 {
            for window in result.recommendations.windows(2) {
                assert!(window[0].priority >= window[1].priority);
            }
        }
    }

    #[tokio::test]
    async fn test_metadata_populated() {
        let tmp = TempDir::new().unwrap();
        let aggregator = InfraScoreAggregator::new();
        let result = aggregator.aggregate(tmp.path()).await.unwrap();

        assert_eq!(result.metadata.repository_path, tmp.path());
        assert_eq!(result.metadata.spec_version, "0.1.0");
    }

    #[tokio::test]
    async fn test_category_totals_sum_correctly() {
        let tmp = TempDir::new().unwrap();
        let aggregator = InfraScoreAggregator::new();
        let result = aggregator.aggregate(tmp.path()).await.unwrap();

        let expected = result.categories.workflow_architecture.score
            + result.categories.build_reliability.score
            + result.categories.quality_pipeline.score
            + result.categories.deployment_release.score
            + result.categories.supply_chain.score;

        assert!(
            (result.total_score - expected).abs() < f64::EPSILON,
            "total_score {} != sum of categories {}",
            result.total_score,
            expected
        );
    }

    #[test]
    fn test_estimate_effort() {
        assert_eq!(estimate_effort(5.0), "30 minutes");
        assert_eq!(estimate_effort(3.0), "15 minutes");
        assert_eq!(estimate_effort(2.0), "5 minutes");
    }

    #[test]
    fn test_default_trait() {
        let _aggregator = InfraScoreAggregator;
    }
}
