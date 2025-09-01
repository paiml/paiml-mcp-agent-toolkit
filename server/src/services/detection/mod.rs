// Toyota Way: Unified Detection Framework for Structural Complexity Reduction
//
// This module consolidates detection services under a single, unified
// framework to reduce structural complexity and achieve A+ grade.
//
// Consolidates:
// - duplicate_detector.rs (high-performance LSH duplicate detection)
// - satd_detector.rs (Self-Admitted Technical Debt detection)  
// - polyglot_analyzer.rs (cross-language analysis)

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;

pub mod duplicates;
pub mod integration_tests;
pub mod polyglot;
pub mod satd;

/// Core detection trait for all detection strategies
#[async_trait]
pub trait Detector: Send + Sync {
    /// Input type for this detector
    type Input;
    /// Output type for this detector  
    type Output;
    /// Configuration type for this detector
    type Config;
    
    /// Perform detection analysis
    async fn detect(
        &self,
        input: Self::Input,
        config: Self::Config,
    ) -> Result<Self::Output>;
    
    /// Get the detector name
    fn name(&self) -> &'static str;
    
    /// Get detector capabilities/features
    fn capabilities(&self) -> DetectorCapabilities;
}

/// Detector capabilities descriptor
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DetectorCapabilities {
    pub supports_batch: bool,
    pub supports_streaming: bool,
    pub language_agnostic: bool,
    pub requires_ast: bool,
}

/// Registry for managing detection strategies
pub struct DetectionRegistry {
    detectors: std::collections::HashMap<String, Arc<dyn Detector<Input = DetectionInput, Output = DetectionOutput, Config = DetectionConfig>>>,
}

/// Unified detection input wrapper
#[derive(Debug, Clone)]
pub enum DetectionInput {
    SingleFile(std::path::PathBuf),
    MultipleFiles(Vec<std::path::PathBuf>),
    ProjectDirectory(std::path::PathBuf),
    Content(String),
}

/// Unified detection output wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DetectionOutput {
    Duplicates(duplicates::DuplicateDetectionResult),
    SATD(satd::SATDAnalysisResult),
    Polyglot(polyglot::PolyglotAnalysis),
}

/// Unified detection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionConfig {
    pub max_files: Option<usize>,
    pub parallel_processing: bool,
    pub output_format: OutputFormat,
    pub detector_specific: DetectorSpecificConfig,
}

/// Output format options
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OutputFormat {
    Json,
    Yaml,
    Summary,
}

/// Detector-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DetectorSpecificConfig {
    Duplicates(duplicates::DuplicateConfig),
    SATD(satd::SATDConfig),
    Polyglot(polyglot::PolyglotConfig),
}

impl DetectionRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            detectors: std::collections::HashMap::new(),
        };
        
        // Register all available detection strategies
        registry.register_defaults();
        registry
    }
    
    fn register_defaults(&mut self) {
        // Register duplicate detector
        self.register("duplicates", Arc::new(duplicates::DuplicateDetector::new()));
        
        // Register SATD detector
        self.register("satd", Arc::new(satd::SATDDetector::new()));
        
        // Register polyglot analyzer
        self.register("polyglot", Arc::new(polyglot::PolyglotDetector::new()));
    }
    
    pub fn register(&mut self, name: &str, detector: Arc<dyn Detector<Input = DetectionInput, Output = DetectionOutput, Config = DetectionConfig>>) {
        self.detectors.insert(name.to_string(), detector);
    }
    
    pub fn get_detector(&self, name: &str) -> Option<Arc<dyn Detector<Input = DetectionInput, Output = DetectionOutput, Config = DetectionConfig>>> {
        self.detectors.get(name).cloned()
    }
    
    pub fn list_detectors(&self) -> Vec<&str> {
        self.detectors.keys().map(|s| s.as_str()).collect()
    }
    
    /// Run detection using the specified detector
    pub async fn detect(
        &self,
        detector_name: &str,
        input: DetectionInput,
        config: DetectionConfig,
    ) -> Result<DetectionOutput> {
        if let Some(detector) = self.get_detector(detector_name) {
            detector.detect(input, config).await
        } else {
            Err(anyhow::anyhow!("Unknown detector: {}", detector_name))
        }
    }
}

impl Default for DetectionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for DetectionConfig {
    fn default() -> Self {
        Self {
            max_files: None,
            parallel_processing: true,
            output_format: OutputFormat::Json,
            detector_specific: DetectorSpecificConfig::Duplicates(duplicates::DuplicateConfig::default()),
        }
    }
}

/// High-level unified detection processor
pub struct UnifiedDetectionProcessor {
    registry: DetectionRegistry,
}

impl UnifiedDetectionProcessor {
    pub fn new() -> Self {
        Self {
            registry: DetectionRegistry::new(),
        }
    }
    
    /// Detect duplicates in files
    pub async fn detect_duplicates(&self, files: Vec<std::path::PathBuf>) -> Result<duplicates::DuplicateDetectionResult> {
        let input = DetectionInput::MultipleFiles(files);
        let config = DetectionConfig {
            detector_specific: DetectorSpecificConfig::Duplicates(duplicates::DuplicateConfig::default()),
            ..Default::default()
        };
        
        match self.registry.detect("duplicates", input, config).await? {
            DetectionOutput::Duplicates(result) => Ok(result),
            _ => Err(anyhow::anyhow!("Invalid output type for duplicates detector")),
        }
    }
    
    /// Detect SATD in project
    pub async fn detect_satd(&self, project_path: &Path) -> Result<satd::SATDAnalysisResult> {
        let input = DetectionInput::ProjectDirectory(project_path.to_path_buf());
        let config = DetectionConfig {
            detector_specific: DetectorSpecificConfig::SATD(satd::SATDConfig::default()),
            ..Default::default()
        };
        
        match self.registry.detect("satd", input, config).await? {
            DetectionOutput::SATD(result) => Ok(result),
            _ => Err(anyhow::anyhow!("Invalid output type for SATD detector")),
        }
    }
    
    /// Analyze polyglot architecture
    pub async fn analyze_polyglot(&self, project_path: &Path) -> Result<polyglot::PolyglotAnalysis> {
        let input = DetectionInput::ProjectDirectory(project_path.to_path_buf());
        let config = DetectionConfig {
            detector_specific: DetectorSpecificConfig::Polyglot(polyglot::PolyglotConfig::default()),
            ..Default::default()
        };
        
        match self.registry.detect("polyglot", input, config).await? {
            DetectionOutput::Polyglot(result) => Ok(result),
            _ => Err(anyhow::anyhow!("Invalid output type for polyglot detector")),
        }
    }
    
    pub fn available_detectors(&self) -> Vec<&str> {
        self.registry.list_detectors()
    }
}

impl Default for UnifiedDetectionProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;
    
    #[tokio::test]
    async fn test_detection_registry_creation() {
        let registry = DetectionRegistry::new();
        let detectors = registry.list_detectors();
        
        // Should have all three detectors
        assert!(detectors.contains(&"duplicates"));
        assert!(detectors.contains(&"satd"));
        assert!(detectors.contains(&"polyglot"));
        assert_eq!(detectors.len(), 3);
    }
    
    #[tokio::test]
    async fn test_unified_processor() {
        let processor = UnifiedDetectionProcessor::new();
        let available = processor.available_detectors();
        
        assert!(available.contains(&"duplicates"));
        assert!(available.contains(&"satd"));
        assert!(available.contains(&"polyglot"));
        assert_eq!(available.len(), 3);
    }
    
    #[test]
    fn test_detection_config_default() {
        let config = DetectionConfig::default();
        
        assert!(config.parallel_processing);
        assert!(config.max_files.is_none());
        assert_eq!(config.output_format, OutputFormat::Json);
        assert!(matches!(config.detector_specific, DetectorSpecificConfig::Duplicates(_)));
    }
    
    #[test]
    fn test_detector_capabilities() {
        let caps = DetectorCapabilities {
            supports_batch: true,
            supports_streaming: false,
            language_agnostic: true,
            requires_ast: false,
        };
        
        assert!(caps.supports_batch);
        assert!(!caps.supports_streaming);
        assert!(caps.language_agnostic);
        assert!(!caps.requires_ast);
    }
}