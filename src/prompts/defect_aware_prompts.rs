#![cfg_attr(coverage_nightly, coverage(off))]
// Defect-Aware Prompt Generator for AI Integration (Phase 4)
// Generates context-aware prompts from organizational intelligence

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Defect pattern from OIP analysis (simplified for prompt generation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefectPattern {
    pub category: String,
    pub frequency: usize,
    pub confidence: f32,
    pub quality_signals: QualitySignals,
    #[serde(default)]
    pub examples: Vec<serde_json::Value>, // Ignored in prompts
}

/// Quality signals for defect patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualitySignals {
    pub avg_tdg_score: Option<f32>,
    pub max_tdg_score: Option<f32>,
    pub avg_complexity: Option<f32>,
    pub avg_test_coverage: Option<f32>,
    pub satd_instances: usize,
    pub avg_lines_changed: f32,
    pub avg_files_per_commit: f32,
}

/// Quality thresholds for code assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityThresholds {
    pub tdg_minimum: f32,
    pub test_coverage_minimum: f32,
    pub max_function_length: usize,
    pub max_cyclomatic_complexity: usize,
}

/// Organizational insights container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationalInsights {
    pub top_defect_categories: Vec<DefectPattern>,
}

/// Metadata about the analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisMetadata {
    pub analysis_date: String,
    pub repositories_analyzed: usize,
    pub commits_analyzed: usize,
}

/// OIP analysis summary (matches OIP output format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OipSummary {
    pub organizational_insights: OrganizationalInsights,
    pub code_quality_thresholds: QualityThresholds,
    pub metadata: AnalysisMetadata,
}

/// Generate context-aware prompts from defect analysis
pub struct DefectAwarePromptGenerator {
    pub defect_patterns: Vec<DefectPattern>,
    pub quality_thresholds: QualityThresholds,
    pub metadata: AnalysisMetadata,
}

impl DefectAwarePromptGenerator {
    /// Load OIP analysis summary from file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content =
            std::fs::read_to_string(path.as_ref()).context("Failed to read OIP summary file")?;

        let summary: OipSummary =
            serde_yaml_ng::from_str(&content).context("Failed to parse OIP summary YAML")?;

        Ok(Self {
            defect_patterns: summary.organizational_insights.top_defect_categories,
            quality_thresholds: summary.code_quality_thresholds,
            metadata: summary.metadata,
        })
    }

    /// Generate context-aware prompt for a specific task
    pub fn generate_prompt(&self, task: &str, context: &str) -> String {
        let mut prompt = String::new();

        // Task section
        prompt.push_str("# Task\n");
        prompt.push_str(task);
        prompt.push_str("\n\n");

        // Context section
        prompt.push_str("# Context\n");
        prompt.push_str(context);
        prompt.push_str("\n\n");

        // Organizational learnings
        prompt.push_str("# Organizational Quality Standards\n\n");
        prompt.push_str(&format!(
            "Based on analysis of {} repositories with {} commits:\n\n",
            self.metadata.repositories_analyzed, self.metadata.commits_analyzed
        ));

        // Quality requirements
        prompt.push_str("## Quality Requirements\n");
        prompt.push_str(&format!(
            "- Minimum TDG Score: {:.0}\n",
            self.quality_thresholds.tdg_minimum
        ));
        prompt.push_str(&format!(
            "- Test Coverage: {:.0}%+\n",
            self.quality_thresholds.test_coverage_minimum * 100.0
        ));
        prompt.push_str(&format!(
            "- Max Function Length: {} lines\n",
            self.quality_thresholds.max_function_length
        ));
        prompt.push_str(&format!(
            "- Max Cyclomatic Complexity: {}\n\n",
            self.quality_thresholds.max_cyclomatic_complexity
        ));

        // Add relevant defect patterns (only high frequency)
        let high_frequency_patterns: Vec<_> = self
            .defect_patterns
            .iter()
            .filter(|p| p.frequency >= 10) // Only include patterns with 10+ occurrences
            .collect();

        if !high_frequency_patterns.is_empty() {
            prompt.push_str("## Common Defect Patterns to Avoid\n\n");

            for pattern in high_frequency_patterns {
                let avg_tdg = pattern.quality_signals.avg_tdg_score.unwrap_or(0.0);
                prompt.push_str(&format!(
                    "### {} ({} occurrences, TDG: {:.1})\n\n",
                    pattern.category, pattern.frequency, avg_tdg
                ));
            }
        }

        // Quality gates
        prompt.push_str("## Quality Gates (Before Committing)\n\n");
        prompt.push_str("```bash\n");
        prompt.push_str(&format!(
            "pmat analyze tdg --threshold {:.0}\n",
            self.quality_thresholds.tdg_minimum
        ));
        prompt.push_str("cargo test --all-features\n");
        prompt.push_str("cargo llvm-cov report --summary-only\n");
        prompt.push_str("```\n\n");

        // Analysis metadata
        prompt.push_str(&format!(
            "**Analysis Date**: {}\n",
            self.metadata.analysis_date
        ));
        prompt.push_str(&format!(
            "**Repositories Analyzed**: {}\n",
            self.metadata.repositories_analyzed
        ));
        prompt.push_str(&format!(
            "**Commits Analyzed**: {}\n",
            self.metadata.commits_analyzed
        ));

        prompt
    }

    /// Generate prompt for preventing a specific defect category
    pub fn generate_prevention_prompt(&self, defect_category: &str) -> Option<String> {
        self.defect_patterns
            .iter()
            .find(|p| p.category == defect_category)
            .map(|pattern| {
                let mut prompt = String::new();

                prompt.push_str(&format!("# Preventing {}\n\n", pattern.category));

                prompt.push_str(&format!(
                    "**Historical Frequency**: {} occurrences\n",
                    pattern.frequency
                ));

                let avg_tdg = pattern.quality_signals.avg_tdg_score.unwrap_or(0.0);
                prompt.push_str(&format!(
                    "**Average Code Quality**: TDG {:.1}/100\n\n",
                    avg_tdg
                ));

                prompt
            })
    }
}

#[cfg(test)]
#[path = "defect_aware_prompts_tests.rs"]
mod defect_aware_prompts_tests;
