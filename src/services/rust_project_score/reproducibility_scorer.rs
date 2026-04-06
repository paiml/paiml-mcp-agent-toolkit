#![cfg_attr(coverage_nightly, coverage(off))]
//! RPS v3.0 Reproducibility Scorer
//!
//! Wraps Popper Falsifiability Score categories B-F into a single RPS scorer,
//! absorbing scientific rigor metrics into the Rust Project Score framework.
//!
//! ## PMAT-510: Popper Score Absorption
//!
//! Popper categories B-F measure reproducibility, transparency, statistical rigor,
//! historical integrity, and ML reproducibility. These map naturally to RPS as a
//! single "Reproducibility & Scientific Rigor" category.
//!
//! Category A (Falsifiability) is NOT absorbed — it remains as the gateway check
//! evaluated separately via `check_falsifiability_gateway()`.
//!
//! ## Points: 15 max
//!
//! Normalized from Popper B-F (75 points available, 70 if ML N/A) down to 15 RPS points.

use super::models::{CategoryScore, FileCache, ScoringMode};
use super::scorer::{Scorer, ScorerResult};
use crate::services::popper_score::scorer::PopperScorer;
use crate::services::popper_score::scorers::{
    HistoricalIntegrityScorer, MLReproducibilityScorer,
    ReproducibilityScorer as PopperReproducibilityScorer, StatisticalRigorScorer,
    TransparencyScorer,
};
use std::path::Path;

/// Reproducibility & Scientific Rigor scorer for RPS v3.0
///
/// Wraps Popper categories B-F (75 points) → normalized to 15 RPS points.
pub struct ReproducibilityScorer {
    popper_b: PopperReproducibilityScorer,
    popper_c: TransparencyScorer,
    popper_d: StatisticalRigorScorer,
    popper_e: HistoricalIntegrityScorer,
    popper_f: MLReproducibilityScorer,
}

impl ReproducibilityScorer {
    /// Create a new reproducibility scorer wrapping Popper B-F
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {
            popper_b: PopperReproducibilityScorer::new(),
            popper_c: TransparencyScorer::new(),
            popper_d: StatisticalRigorScorer::new(),
            popper_e: HistoricalIntegrityScorer::new(),
            popper_f: MLReproducibilityScorer::new(),
        }
    }
}

impl Default for ReproducibilityScorer {
    fn default() -> Self {
        Self::new()
    }
}

impl Scorer for ReproducibilityScorer {
    fn name(&self) -> &str {
        "Reproducibility"
    }

    fn max_points(&self) -> f64 {
        15.0
    }

    fn score_with_mode(
        &self,
        project_path: &Path,
        _mode: ScoringMode,
    ) -> ScorerResult<CategoryScore> {
        // Run all Popper B-F scorers
        let b = self.popper_b.score(project_path).ok();
        let c = self.popper_c.score(project_path).ok();
        let d = self.popper_d.score(project_path).ok();
        let e = self.popper_e.score(project_path).ok();
        let f = self.popper_f.score(project_path).ok();

        // Accumulate earned and max from applicable categories
        let mut total_earned = 0.0_f64;
        let mut total_max = 0.0_f64;

        for score in [&b, &c, &d, &e].into_iter().flatten() {
            total_earned += score.earned;
            total_max += score.max;
        }

        // Category F (ML) is conditional — only include if applicable
        if let Some(ml_score) = &f {
            if ml_score.is_applicable {
                total_earned += ml_score.earned;
                total_max += ml_score.max;
            }
        }

        // Normalize from Popper scale to RPS 15-point scale
        let percentage = if total_max > 0.0 {
            total_earned / total_max
        } else {
            0.0
        };

        let earned = (percentage * self.max_points()).min(self.max_points());

        Ok(CategoryScore::new(earned, self.max_points()))
    }

    fn score_with_cache(
        &self,
        project_path: &Path,
        mode: ScoringMode,
        _cache: Option<&FileCache>,
    ) -> ScorerResult<CategoryScore> {
        // Popper scorers don't use FileCache, delegate to score_with_mode
        self.score_with_mode(project_path, mode)
    }

    fn recommendations(&self, project_path: &Path) -> Vec<String> {
        let mut recs = Vec::new();

        // Check each Popper category for gaps
        if let Ok(b) = self.popper_b.score(project_path) {
            if b.percentage() < 60.0 {
                recs.push(
                    "Add lock files, Docker/Nix environment for reproducibility (Popper B)"
                        .to_string(),
                );
            }
        }

        if let Ok(c) = self.popper_c.score(project_path) {
            if c.percentage() < 60.0 {
                recs.push(
                    "Add LICENSE, ADRs, and API documentation for transparency (Popper C)"
                        .to_string(),
                );
            }
        }

        if let Ok(d) = self.popper_d.score(project_path) {
            if d.percentage() < 60.0 {
                recs.push(
                    "Add benchmark confidence intervals and statistical rigor (Popper D)"
                        .to_string(),
                );
            }
        }

        recs
    }
}

/// Check Popper Category A (Falsifiability) gateway for RPS v3.0
///
/// If a project scores below 60% on falsifiability, RPS v3.0 caps
/// the overall grade at F. This implements Jidoka (stop the line).
///
/// Returns `Some(percentage)` if gateway passes, `None` if it fails.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn check_falsifiability_gateway(project_path: &Path) -> Option<f64> {
    use crate::services::popper_score::scorers::FalsifiabilityScorer;

    let scorer = FalsifiabilityScorer::new();
    if let Ok(result) = scorer.score(project_path) {
        let pct = result.percentage();
        if pct >= 60.0 {
            Some(pct)
        } else {
            None
        }
    } else {
        // If we can't score falsifiability, don't block
        Some(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_reproducibility_scorer_basics() {
        let scorer = ReproducibilityScorer::new();
        assert_eq!(scorer.name(), "Reproducibility");
        assert_eq!(scorer.max_points(), 15.0);
    }

    #[test]
    fn test_empty_project_scores_low() {
        let temp_dir = tempdir().unwrap();
        let scorer = ReproducibilityScorer::new();
        let result = scorer.score(temp_dir.path()).unwrap();
        assert!(result.earned < 5.0);
        assert_eq!(result.max, 15.0);
    }

    #[test]
    fn test_well_equipped_project() {
        let temp_dir = tempdir().unwrap();

        // Create signals for Popper B-F
        fs::write(temp_dir.path().join("Cargo.lock"), "# Lock").unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::write(temp_dir.path().join("Makefile"), "build:\n\tcargo build").unwrap();
        fs::write(
            temp_dir.path().join("LICENSE"),
            "MIT License\n\nCopyright (c) 2025",
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("README.md"),
            "# Project\n\n## Installation\n\n```bash\nmake build\n```\n\n## Usage\n\nRun the tool.\n\nThis is a comprehensive README with enough content to pass the threshold for documentation scoring purposes.",
        )
        .unwrap();
        fs::create_dir_all(temp_dir.path().join("docs")).unwrap();
        fs::create_dir_all(temp_dir.path().join(".git")).unwrap();

        let scorer = ReproducibilityScorer::new();
        let result = scorer.score(temp_dir.path()).unwrap();

        // Should earn meaningful points
        assert!(
            result.earned > 3.0,
            "Expected >3 earned, got {}",
            result.earned
        );
        assert_eq!(result.max, 15.0);
    }

    #[test]
    fn test_falsifiability_gateway_empty_project() {
        let temp_dir = tempdir().unwrap();
        let result = check_falsifiability_gateway(temp_dir.path());
        // Empty project should fail the gateway
        assert!(result.is_none());
    }

    #[test]
    fn test_recommendations_for_bare_project() {
        let temp_dir = tempdir().unwrap();
        let scorer = ReproducibilityScorer::new();
        let recs = scorer.recommendations(temp_dir.path());
        // Should have at least some recommendations for a bare project
        assert!(!recs.is_empty());
    }
}
