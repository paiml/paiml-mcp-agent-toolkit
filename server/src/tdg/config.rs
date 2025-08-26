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
            max_cyclomatic_complexity: 10,
            max_cognitive_complexity: 15,
            max_nesting_depth: 3,
            min_token_sequence: 50,
            similarity_threshold: 0.85,
            max_coupling: 10,
            min_doc_coverage: 0.8,
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
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }
    
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_penalty_curve_application() {
        assert_eq!(PenaltyCurve::Linear.apply(2.0, 3.0), 6.0);
        assert!(PenaltyCurve::Logarithmic.apply(2.0, 3.0) > 0.0);
        assert_eq!(PenaltyCurve::Quadratic.apply(2.0, 3.0), 12.0);
        assert!(PenaltyCurve::Exponential.apply(2.0, 1.0) > 7.0);
    }
    
    #[test]
    fn test_config_serialization() -> Result<()> {
        let config = TdgConfig::default();
        let toml_str = toml::to_string(&config)?;
        assert!(toml_str.contains("structural_complexity"));
        assert!(toml_str.contains("max_cyclomatic_complexity"));
        Ok(())
    }
    
    #[test]
    fn test_config_from_file() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(
            temp_file,
            r#"
[weights]
structural_complexity = 30.0
semantic_complexity = 20.0
duplication = 15.0
coupling = 15.0
documentation = 10.0
consistency = 10.0

[thresholds]
max_cyclomatic_complexity = 15
max_cognitive_complexity = 20
max_nesting_depth = 4
min_token_sequence = 40
similarity_threshold = 0.9
max_coupling = 12
min_doc_coverage = 0.7

[penalties]
complexity_penalty_base = "Logarithmic"
duplication_penalty_curve = "Linear"
coupling_penalty_curve = "Quadratic"
"#
        )?;
        
        let config = TdgConfig::from_file(temp_file.path())?;
        assert_eq!(config.weights.structural_complexity, 30.0);
        assert_eq!(config.thresholds.max_cyclomatic_complexity, 15);
        assert!(matches!(config.penalties.complexity_penalty_base, PenaltyCurve::Logarithmic));
        
        Ok(())
    }
}