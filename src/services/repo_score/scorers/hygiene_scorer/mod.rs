// HygieneScorer - Category C: Repository Hygiene (15 points)
//
// Scores based on:
// - C1: No Cruft Files (5 points) - No transient files, caches, build artifacts
// - C2: No Team-Specific Files (5 points) - No .idea/, .vscode/, .DS_Store
// - C3: No Large Files in Git History (5 points) - No files >1MB in git history

#![cfg_attr(coverage_nightly, coverage(off))]

mod cruft;
mod large_files;
pub(crate) mod patterns;
mod team_files;

#[cfg(test)]
mod coverage_tests_integration;
#[cfg(test)]
mod coverage_tests_unit;
#[cfg(test)]
mod tests;

use super::{Scorer, ScorerConfig};
use crate::services::repo_score::error::Result;
use crate::services::repo_score::models::*;
use async_trait::async_trait;
use std::path::Path;

// Re-export matches_pattern for test modules
#[cfg(test)]
use patterns::matches_pattern;

pub struct HygieneScorer;

impl HygieneScorer {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Scorer for HygieneScorer {
    fn category_name(&self) -> &str {
        "Repository Hygiene"
    }

    fn max_score(&self) -> f64 {
        15.0
    }

    async fn score(&self, repo_path: &Path, config: &ScorerConfig) -> Result<CategoryScore> {
        let c1 = self.score_cruft(repo_path).await?;
        let c2 = self.score_team_files(repo_path).await?;
        let c3 = self.score_large_files(repo_path, config).await?;

        let total_score = c1.score + c2.score + c3.score;

        let mut findings = c1.findings.clone();
        findings.extend(c2.findings.clone());
        findings.extend(c3.findings.clone());

        Ok(CategoryScore::new(
            total_score,
            self.max_score(),
            vec![c1, c2, c3],
            findings,
        ))
    }
}

impl Default for HygieneScorer {
    fn default() -> Self {
        Self::new()
    }
}
