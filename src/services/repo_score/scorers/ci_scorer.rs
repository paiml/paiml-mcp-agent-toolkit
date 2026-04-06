#![cfg_attr(coverage_nightly, coverage(off))]
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
}

impl Default for CiScorer {
    fn default() -> Self {
        Self::new()
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

include!("ci_scorer_workflows.rs");
include!("ci_scorer_advanced.rs");
include!("ci_scorer_tests.rs");
