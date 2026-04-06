#![cfg_attr(coverage_nightly, coverage(off))]
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TdgConfig {
    pub weights: WeightConfig,
    pub thresholds: ThresholdConfig,
    pub penalties: PenaltyConfig,
    pub language_overrides: HashMap<String, LanguageOverride>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightConfig {
    pub structural_complexity: f32,
    pub semantic_complexity: f32,
    pub duplication: f32,
    pub coupling: f32,
    pub documentation: f32,
    pub consistency: f32,
}

impl Default for WeightConfig {
    fn default() -> Self {
        Self {
            structural_complexity: 25.0,
            semantic_complexity: 20.0,
            duplication: 20.0,
            coupling: 15.0,
            documentation: 10.0,
            consistency: 10.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdConfig {
    pub max_cyclomatic_complexity: u32,
    pub max_cognitive_complexity: u32,
    pub max_nesting_depth: u32,
    pub min_token_sequence: usize,
    pub similarity_threshold: f32,
    pub max_coupling: u32,
    pub min_doc_coverage: f32,
}

impl Default for ThresholdConfig {
    fn default() -> Self {
        Self {
            max_cyclomatic_complexity: 30, // Enterprise standard (was 10 - too strict)
            max_cognitive_complexity: 25,  // Reasonable threshold (was 15 - too strict)
            max_nesting_depth: 4,          // Allow reasonable nesting (was 3)
            min_token_sequence: 50,
            similarity_threshold: 0.85,
            max_coupling: 15,       // More realistic (was 10)
            min_doc_coverage: 0.75, // Balanced target (was 0.8)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PenaltyConfig {
    pub complexity_penalty_base: PenaltyCurve,
    pub duplication_penalty_curve: PenaltyCurve,
    pub coupling_penalty_curve: PenaltyCurve,
}

impl Default for PenaltyConfig {
    fn default() -> Self {
        Self {
            complexity_penalty_base: PenaltyCurve::Logarithmic,
            duplication_penalty_curve: PenaltyCurve::Linear,
            coupling_penalty_curve: PenaltyCurve::Quadratic,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PenaltyCurve {
    Linear,
    Logarithmic,
    Quadratic,
    Exponential,
}

impl PenaltyCurve {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn apply(&self, value: f32, base: f32) -> f32 {
        match self {
            PenaltyCurve::Linear => value * base,
            PenaltyCurve::Logarithmic => {
                if value > 1.0 {
                    value.ln() * base
                } else {
                    0.0
                }
            }
            PenaltyCurve::Quadratic => value * value * base,
            PenaltyCurve::Exponential => value.exp() * base,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageOverride {
    pub max_cognitive_complexity: Option<u32>,
    pub min_doc_coverage: Option<f32>,
    pub enforce_error_check: Option<bool>,
    pub max_function_length: Option<u32>,
}

impl TdgConfig {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn from_file(path: &Path) -> Result<Self> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn save(&self, path: &Path) -> Result<()> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

include!("config_tests.rs");
