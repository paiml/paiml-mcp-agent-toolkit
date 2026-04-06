#![cfg_attr(coverage_nightly, coverage(off))]
// DemoScorer - Category G: Demo Quality (10 points)
//
// Scores based on:
// - G1: Time-to-Interaction (3 points) - Demo starts quickly, quick-start guide present
// - G2: Error Gracefulness (3 points) - Proper error handling, no raw panics/stack traces
// - G3: Visual Stability (2 points) - Rich output formatting, consistent UX patterns
// - G4: "Wow" Factor (2 points) - Uses rich terminal UI or interactive components
//
// References (Primary):
// - Storey et al. (2017) - Interactive demos reduce cognitive barriers
// - Lavie & Tractinsky (2004) - Visual aesthetics correlate with perceived usability
// - Miller (1968) - Response time thresholds for user perception
//
// References (Review Additions - Toyota Way):
// - Nasehi et al. (2012) - Code example quality in StackOverflow
// - Steinmacher et al. (2015) - Barriers for newcomers to OSS projects
// - Barik et al. (2017) - Error message recoverability
// - Posnett et al. (2011) - Ecological fallacy in software metrics
// - Treude et al. (2011) - Social impact of badges (diminishing returns)
// - Uddin & Robillard (2015) - API documentation failure modes

use super::{Scorer, ScorerConfig};
use crate::services::repo_score::error::Result;
use crate::services::repo_score::models::*;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// --- Submodule includes ---
// RepoArchetype enum, DemoScorer struct, detect_archetype
include!("demo_scorer_archetype.rs");
// File counting helpers (count_files_by_extension, count_code_files)
include!("demo_scorer_file_counting.rs");
// G1: Time-to-Interaction (3 points)
include!("demo_scorer_g1_time_to_interaction.rs");
// G2: Error Gracefulness - entry point and N/A handling
include!("demo_scorer_g2_error_gracefulness.rs");
// G2: Error pattern analysis of demo files
include!("demo_scorer_g2_analysis.rs");
// G3: Visual Stability (2 points)
include!("demo_scorer_g3_visual_stability.rs");
// G3: Library usage verification (Genchi Genbutsu)
include!("demo_scorer_g3_library_verification.rs");
// G4: "Wow" Factor (2 points)
include!("demo_scorer_g4_wow_factor.rs");
// find_demo_files helper (used by G2, G3, G4)
include!("demo_scorer_find_demo_files.rs");

#[async_trait]
impl Scorer for DemoScorer {
    fn category_name(&self) -> &str {
        "Demo Quality"
    }

    fn max_score(&self) -> f64 {
        10.0
    }

    async fn score(&self, repo_path: &Path, _config: &ScorerConfig) -> Result<CategoryScore> {
        debug_assert!(
            repo_path.exists(),
            "repo_path must exist: {}",
            repo_path.display()
        );
        // Detect repository archetype for calibrated scoring
        let archetype = self.detect_archetype(repo_path).await;

        let g1 = self.score_time_to_interaction(repo_path).await?;
        let g2 = self.score_error_gracefulness(repo_path, archetype).await?;
        let g3 = self.score_visual_stability(repo_path).await?;
        let g4 = self.score_wow_factor(repo_path).await?;

        // Calculate dynamic max score based on archetype (N/A categories removed from denominator)
        let effective_max = g1.max_score + g2.max_score + g3.max_score + g4.max_score;
        let total_score = g1.score + g2.score + g3.score + g4.score;

        let mut findings = vec![Finding {
            severity: Severity::Info,
            category: "Demo Quality".to_string(),
            message: format!("Repository detected as: {} archetype", archetype.name()),
            location: None,
            impact_points: 0.0,
        }];
        findings.extend(g1.findings.clone());
        findings.extend(g2.findings.clone());
        findings.extend(g3.findings.clone());
        findings.extend(g4.findings.clone());

        Ok(CategoryScore::new(
            total_score,
            effective_max,
            vec![g1, g2, g3, g4],
            findings,
        ))
    }
}

impl Default for DemoScorer {
    fn default() -> Self {
        Self::new()
    }
}

// Tests extracted to demo_scorer_tests.rs for file health compliance (CB-040)
#[cfg(test)]
#[path = "demo_scorer_tests.rs"]
mod tests;
