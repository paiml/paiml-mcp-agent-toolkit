// Scorer trait and implementations

pub mod ci_scorer;
pub mod hygiene_scorer;
pub mod makefile_scorer;
pub mod pmat_scorer;
pub mod precommit_scorer;
pub mod readme_scorer;

pub use ci_scorer::CiScorer;
pub use hygiene_scorer::HygieneScorer;
pub use makefile_scorer::MakefileScorer;
pub use pmat_scorer::PmatScorer;
pub use precommit_scorer::PrecommitScorer;
pub use readme_scorer::ReadmeScorer;

use crate::services::repo_score::error::Result;
use crate::services::repo_score::models::*;
use async_trait::async_trait;
use std::path::Path;

/// Trait for all scoring modules
#[async_trait]
pub trait Scorer: Send + Sync {
    /// Name of the category (e.g., "Documentation Quality")
    fn category_name(&self) -> &str;

    /// Maximum points available for this category
    fn max_score(&self) -> f64;

    /// Execute scoring for this category
    async fn score(&self, repo_path: &Path, config: &ScorerConfig) -> Result<CategoryScore>;
}

/// Configuration for scorers
#[derive(Debug, Clone)]
pub struct ScorerConfig {
    pub verbose: bool,
    pub timeout_seconds: u64,
    pub skip_slow_checks: bool,
    /// Deep scan: Check entire git history (slower but more thorough)
    /// Default: false (scan HEAD only)
    pub deep: bool,
}

impl Default for ScorerConfig {
    fn default() -> Self {
        Self {
            verbose: false,
            timeout_seconds: 300,
            skip_slow_checks: false,
            deep: false,
        }
    }
}
