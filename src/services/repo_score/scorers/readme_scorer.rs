#![cfg_attr(coverage_nightly, coverage(off))]
// ReadmeScorer - Category A: Documentation Quality (15 points)
//
// Scores based on:
// - A1: README Accuracy (5 points) - No broken links, valid images
// - A2: README Comprehensiveness (5 points) - Required sections present
// - A3: Professional Structure (5 points) - Hero image, ToC, centered header, no bot patterns

use super::{Scorer, ScorerConfig};
use crate::services::repo_score::error::Result;
use crate::services::repo_score::models::*;
use async_trait::async_trait;
use std::path::Path;

pub struct ReadmeScorer;

impl ReadmeScorer {
    pub fn new() -> Self {
        Self
    }
}

// A1: README Accuracy scoring - broken links and image validation
include!("readme_scorer_accuracy.rs");

// A2/A3: README Comprehensiveness and Professional Structure scoring
include!("readme_scorer_structure.rs");

#[async_trait]
impl Scorer for ReadmeScorer {
    fn category_name(&self) -> &str {
        "Documentation"
    }

    fn max_score(&self) -> f64 {
        15.0
    }

    async fn score(&self, repo_path: &Path, _config: &ScorerConfig) -> Result<CategoryScore> {
        let a1 = self.score_accuracy(repo_path).await?;
        let a2 = self.score_comprehensiveness(repo_path).await?;
        let a3 = self.score_professional_structure(repo_path).await?;

        let total_score = a1.score + a2.score + a3.score;

        let mut findings = a1.findings.clone();
        findings.extend(a2.findings.clone());
        findings.extend(a3.findings.clone());

        Ok(CategoryScore::new(
            total_score,
            self.max_score(),
            vec![a1, a2, a3],
            findings,
        ))
    }
}

impl Default for ReadmeScorer {
    fn default() -> Self {
        Self::new()
    }
}

// Unit tests for all ReadmeScorer functionality
include!("readme_scorer_tests.rs");
