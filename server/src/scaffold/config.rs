// Configuration types for scaffolding
// Part of TICKET-PMAT-5001

use serde::{Deserialize, Serialize};

/// Scaffolding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaffoldConfig {
    pub project_name: String,
    pub template_type: TemplateType,
    pub features: Vec<Feature>,
    pub quality_gates: QualityGateConfig,
}

/// Template type for scaffolding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TemplateType {
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

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // QualityGateConfig Tests
    // ============================================================================

    #[test]
    fn test_quality_gate_config_default() {
        let config = QualityGateConfig::default();
        assert_eq!(config.max_cyclomatic, 10);
        assert_eq!(config.max_cognitive, 15);
        assert!((config.min_coverage - 0.80).abs() < f32::EPSILON);
        assert!((config.min_mutation_score - 0.85).abs() < f32::EPSILON);
        assert!(config.strict_satd);
    }

    #[test]
    fn test_quality_gate_config_extreme_tdd() {
        let config = QualityGateConfig::extreme_tdd();
        assert_eq!(config.max_cyclomatic, 10);
        assert_eq!(config.max_cognitive, 15);
        assert!((config.min_coverage - 0.85).abs() < f32::EPSILON);
        assert!((config.min_mutation_score - 0.90).abs() < f32::EPSILON);
        assert!(config.strict_satd);
    }

    #[test]
    fn test_quality_gate_config_clone() {
        let config = QualityGateConfig::default();
        let cloned = config.clone();
        assert_eq!(cloned.max_cyclomatic, config.max_cyclomatic);
        assert_eq!(cloned.max_cognitive, config.max_cognitive);
    }

    #[test]
    fn test_quality_gate_config_debug() {
        let config = QualityGateConfig::default();
        let debug = format!("{:?}", config);
        assert!(debug.contains("QualityGateConfig"));
        assert!(debug.contains("max_cyclomatic"));
    }

    #[test]
    fn test_quality_gate_config_serialization() {
        let config = QualityGateConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("max_cyclomatic"));
        assert!(json.contains("10"));

        let deserialized: QualityGateConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.max_cyclomatic, config.max_cyclomatic);
    }

    // ============================================================================
    // TemplateType Tests
    // ============================================================================

    #[test]
    fn test_template_type_agent() {
        let template = TemplateType::Agent { based_on: AgentFramework::Pforge };
        assert!(matches!(template, TemplateType::Agent { .. }));
    }

    #[test]
    fn test_template_type_wasm() {
        let template = TemplateType::Wasm { based_on: WasmFramework::WasmLabs };
        assert!(matches!(template, TemplateType::Wasm { .. }));
    }

    #[test]
    fn test_template_type_library() {
        let template = TemplateType::Library;
        assert!(matches!(template, TemplateType::Library));
    }

    #[test]
    fn test_template_type_custom() {
        let template = TemplateType::Custom { path: std::path::PathBuf::from("/test/path") };
        assert!(matches!(template, TemplateType::Custom { .. }));
    }

    #[test]
    fn test_template_type_clone() {
        let template = TemplateType::Library;
        let cloned = template.clone();
        assert!(matches!(cloned, TemplateType::Library));
    }

    #[test]
    fn test_template_type_debug() {
        let template = TemplateType::Library;
        let debug = format!("{:?}", template);
        assert!(debug.contains("Library"));
    }

    // ============================================================================
    // AgentFramework Tests
    // ============================================================================

    #[test]
    fn test_agent_framework_pforge() {
        let framework = AgentFramework::Pforge;
        assert!(matches!(framework, AgentFramework::Pforge));
    }

    #[test]
    fn test_agent_framework_clone() {
        let framework = AgentFramework::Pforge;
        let cloned = framework.clone();
        assert!(matches!(cloned, AgentFramework::Pforge));
    }

    #[test]
    fn test_agent_framework_debug() {
        let framework = AgentFramework::Pforge;
        let debug = format!("{:?}", framework);
        assert!(debug.contains("Pforge"));
    }

    // ============================================================================
    // WasmFramework Tests
    // ============================================================================

    #[test]
    fn test_wasm_framework_wasm_labs() {
        let framework = WasmFramework::WasmLabs;
        assert!(matches!(framework, WasmFramework::WasmLabs));
    }

    #[test]
    fn test_wasm_framework_pure_wasm() {
        let framework = WasmFramework::PureWasm;
        assert!(matches!(framework, WasmFramework::PureWasm));
    }

    #[test]
    fn test_wasm_framework_clone() {
        let framework = WasmFramework::WasmLabs;
        let cloned = framework.clone();
        assert!(matches!(cloned, WasmFramework::WasmLabs));
    }

    // ============================================================================
    // Feature Tests
    // ============================================================================

    #[test]
    fn test_feature_logging() {
        let feature = Feature::Logging;
        assert!(matches!(feature, Feature::Logging));
    }

    #[test]
    fn test_feature_metrics() {
        let feature = Feature::Metrics;
        assert!(matches!(feature, Feature::Metrics));
    }

    #[test]
    fn test_feature_tracing() {
        let feature = Feature::Tracing;
        assert!(matches!(feature, Feature::Tracing));
    }

    #[test]
    fn test_feature_clone() {
        let feature = Feature::Logging;
        let cloned = feature.clone();
        assert!(matches!(cloned, Feature::Logging));
    }

    // ============================================================================
    // ScaffoldConfig Tests
    // ============================================================================

    #[test]
    fn test_scaffold_config_creation() {
        let config = ScaffoldConfig {
            project_name: "test-project".to_string(),
            template_type: TemplateType::Library,
            features: vec![Feature::Logging, Feature::Metrics],
            quality_gates: QualityGateConfig::default(),
        };

        assert_eq!(config.project_name, "test-project");
        assert!(matches!(config.template_type, TemplateType::Library));
        assert_eq!(config.features.len(), 2);
    }

    #[test]
    fn test_scaffold_config_clone() {
        let config = ScaffoldConfig {
            project_name: "test-project".to_string(),
            template_type: TemplateType::Library,
            features: vec![Feature::Logging],
            quality_gates: QualityGateConfig::default(),
        };

        let cloned = config.clone();
        assert_eq!(cloned.project_name, config.project_name);
    }

    #[test]
    fn test_scaffold_config_serialization() {
        let config = ScaffoldConfig {
            project_name: "test-project".to_string(),
            template_type: TemplateType::Library,
            features: vec![Feature::Logging],
            quality_gates: QualityGateConfig::default(),
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("test-project"));
        assert!(json.contains("Library"));

        let deserialized: ScaffoldConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.project_name, "test-project");
    }

    #[test]
    fn test_scaffold_config_agent_template() {
        let config = ScaffoldConfig {
            project_name: "agent-project".to_string(),
            template_type: TemplateType::Agent { based_on: AgentFramework::Pforge },
            features: vec![Feature::Tracing],
            quality_gates: QualityGateConfig::extreme_tdd(),
        };

        assert_eq!(config.project_name, "agent-project");
        assert!(matches!(config.template_type, TemplateType::Agent { .. }));
    }

    #[test]
    fn test_scaffold_config_wasm_template() {
        let config = ScaffoldConfig {
            project_name: "wasm-project".to_string(),
            template_type: TemplateType::Wasm { based_on: WasmFramework::PureWasm },
            features: vec![],
            quality_gates: QualityGateConfig::default(),
        };

        assert_eq!(config.project_name, "wasm-project");
        assert!(matches!(config.template_type, TemplateType::Wasm { .. }));
    }
}
