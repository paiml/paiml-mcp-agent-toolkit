#![cfg_attr(coverage_nightly, coverage(off))]
//! Core entropy calculation logic.
//!
//! Implements Shannon entropy calculations at file, module, and project levels
//! for measuring pattern diversity across codebases.

use anyhow::Result;
use std::collections::HashMap;

use crate::entropy::pattern_extractor::{AstPattern, PatternCollection};
use crate::entropy::EntropyConfig;

use super::types::EntropyMetrics;

/// Calculates entropy metrics
pub struct EntropyCalculator {
    #[allow(dead_code)]
    config: EntropyConfig,
}

impl EntropyCalculator {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(config: EntropyConfig) -> Self {
        Self { config }
    }

    /// Calculate entropy metrics from patterns
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn calculate(&self, patterns: &PatternCollection) -> Result<EntropyMetrics> {
        let total_patterns = patterns.patterns.len();
        let total_instances: usize = patterns.patterns.values().map(|p| p.frequency).sum();

        let total_loc: usize = patterns
            .patterns
            .values()
            .map(|p| p.estimated_loc * p.frequency)
            .sum();

        // Calculate pattern diversity (Shannon entropy of pattern distribution)
        let pattern_diversity = self.calculate_pattern_diversity(patterns);

        // Calculate entropy at different levels
        let file_level_entropy = self.calculate_file_level_entropy(patterns);
        let module_level_entropy = self.calculate_module_level_entropy(patterns);
        let project_level_entropy = self.calculate_project_level_entropy(patterns);

        // Count patterns by type
        let mut patterns_by_type = HashMap::new();
        for pattern in patterns.patterns.values() {
            *patterns_by_type.entry(pattern.pattern_type).or_insert(0) += pattern.frequency;
        }

        Ok(EntropyMetrics {
            file_level_entropy,
            module_level_entropy,
            project_level_entropy,
            pattern_diversity,
            total_patterns,
            total_instances,
            total_loc,
            patterns_by_type,
        })
    }

    /// Calculate Shannon entropy of pattern distribution
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn calculate_pattern_diversity(&self, patterns: &PatternCollection) -> f64 {
        if patterns.patterns.is_empty() {
            return 0.0;
        }

        let total_instances: usize = patterns.patterns.values().map(|p| p.frequency).sum();

        if total_instances == 0 {
            return 0.0;
        }

        let mut entropy = 0.0;
        for pattern in patterns.patterns.values() {
            let probability = pattern.frequency as f64 / total_instances as f64;
            if probability > 0.0 {
                entropy -= probability * probability.log2();
            }
        }

        // Normalize to 0-1 scale (assuming max entropy of 8 bits for code patterns)
        (entropy / 8.0).min(1.0)
    }

    /// Calculate average entropy at file level
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn calculate_file_level_entropy(&self, patterns: &PatternCollection) -> f64 {
        // Calculate how diverse patterns are within each file
        let mut file_entropies = Vec::new();

        for file_patterns in patterns.file_patterns.values() {
            if file_patterns.is_empty() {
                continue;
            }

            // Count pattern frequencies in this file
            let mut pattern_counts = HashMap::new();
            for pattern_hash in file_patterns {
                *pattern_counts.entry(pattern_hash).or_insert(0) += 1;
            }

            // Calculate entropy for this file
            let total = file_patterns.len() as f64;
            let mut entropy = 0.0;

            for count in pattern_counts.values() {
                let p = f64::from(*count) / total;
                if p > 0.0 {
                    entropy -= p * p.log2();
                }
            }

            file_entropies.push(entropy);
        }

        if file_entropies.is_empty() {
            return 0.0;
        }

        // Return average file entropy
        let sum: f64 = file_entropies.iter().sum();
        (sum / file_entropies.len() as f64 / 8.0).min(1.0)
    }

    /// Calculate entropy at module level
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn calculate_module_level_entropy(&self, patterns: &PatternCollection) -> f64 {
        // Group files by module (simplified: by directory)
        let mut modules: HashMap<String, Vec<&AstPattern>> = HashMap::new();

        for pattern in patterns.patterns.values() {
            for location in &pattern.locations {
                let module = location
                    .file
                    .parent()
                    .and_then(|p| p.to_str())
                    .unwrap_or("root")
                    .to_string();

                modules.entry(module).or_default().push(pattern);
            }
        }

        // Calculate entropy for each module
        let mut module_entropies = Vec::new();

        for module_patterns in modules.values() {
            if module_patterns.is_empty() {
                continue;
            }

            let mut pattern_counts = HashMap::new();
            for pattern in module_patterns {
                *pattern_counts.entry(pattern.pattern_type).or_insert(0) += 1;
            }

            let total = module_patterns.len() as f64;
            let mut entropy = 0.0;

            for count in pattern_counts.values() {
                let p = f64::from(*count) / total;
                if p > 0.0 {
                    entropy -= p * p.log2();
                }
            }

            module_entropies.push(entropy);
        }

        if module_entropies.is_empty() {
            return 0.0;
        }

        let sum: f64 = module_entropies.iter().sum();
        (sum / module_entropies.len() as f64 / 3.0).min(1.0) // Lower max for module level
    }

    /// Calculate entropy at project level
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn calculate_project_level_entropy(&self, patterns: &PatternCollection) -> f64 {
        // Overall project pattern diversity
        self.calculate_pattern_diversity(patterns)
    }
}
