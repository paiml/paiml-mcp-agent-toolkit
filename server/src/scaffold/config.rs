// Configuration types for scaffolding
// Part of TICKET-PMAT-5001

use serde::{Deserialize, Serialize};

/// Scaffolding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaffoldConfig {
    pub project_name: String,
    pub template: Template,
    pub features: Vec<Feature>,
    pub quality_gates: QualityGateConfig,
}

/// Template type for scaffolding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Template {
    Agent { based_on: AgentFramework },
    Wasm { based_on: WasmFramework },
    Library,
    Custom { path: std::path::PathBuf },
}

/// Agent framework for template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentFramework {
    Pforge,
}

/// WASM framework for template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WasmFramework {
    WasmLabs,
    PureWasm,
}

/// Optional features to include
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Feature {
    Logging,
    Metrics,
    Tracing,
}

/// Quality gate configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGateConfig {
    pub max_cyclomatic: u8,
    pub max_cognitive: u8,
    pub min_coverage: f32,
    pub min_mutation_score: f32,
    pub strict_satd: bool,
}

impl Default for QualityGateConfig {
    fn default() -> Self {
        Self {
            max_cyclomatic: 10,
            max_cognitive: 15,
            min_coverage: 0.80,
            min_mutation_score: 0.85,
            strict_satd: true,
        }
    }
}

impl QualityGateConfig {
    /// Extreme TDD configuration (highest standards)
    pub fn extreme_tdd() -> Self {
        Self {
            max_cyclomatic: 10,
            max_cognitive: 15,
            min_coverage: 0.85,
            min_mutation_score: 0.90,
            strict_satd: true,
        }
    }
}
