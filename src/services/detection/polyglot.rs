#![cfg_attr(coverage_nightly, coverage(off))]
// Toyota Way: Unified Polyglot Analysis Strategy

use super::{
    DetectionConfig, DetectionInput, DetectionOutput, Detector, DetectorCapabilities,
    DetectorSpecificConfig,
};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Polyglot analysis strategy using the existing polyglot analyzer
pub struct PolyglotDetector;

impl Default for PolyglotDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl PolyglotDetector {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self
    }
}

/// Polyglot analysis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolyglotConfig {
    pub include_dependencies: bool,
    pub analyze_frameworks: bool,
    pub detect_patterns: bool,
    pub max_depth: usize,
}

impl Default for PolyglotConfig {
    fn default() -> Self {
        Self {
            include_dependencies: true,
            analyze_frameworks: true,
            detect_patterns: true,
            max_depth: 10,
        }
    }
}

/// Polyglot analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolyglotAnalysis {
    pub languages: Vec<LanguageStats>,
    pub cross_language_dependencies: Vec<CrossLanguageDependency>,
    pub architecture_pattern: Option<ArchitecturePattern>,
    pub integration_points: Vec<IntegrationPoint>,
    pub recommendation_score: f64,
}

/// Statistics for a specific language in the project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageStats {
    pub language: String,
    pub file_count: usize,
    pub line_count: usize,
    pub complexity_score: f64,
    pub test_coverage: f64,
    pub primary_frameworks: Vec<String>,
}

/// Cross-language dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossLanguageDependency {
    pub from_language: String,
    pub to_language: String,
    pub dependency_type: DependencyType,
    pub coupling_strength: f64,
    pub files_involved: Vec<String>,
}

/// Types of dependencies between languages
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DependencyType {
    FFI,
    ProcessCommunication,
    SharedDataStructure,
    ConfigurationFile,
    BuildSystem,
    Testing,
}

/// Detected architecture patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArchitecturePattern {
    Monolithic,
    Microservices,
    LayeredWithFFI,
    Modular,
    EventDriven,
}

/// Integration point in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationPoint {
    pub point_type: IntegrationPointType,
    pub location: String,
    pub technologies: Vec<String>,
    pub complexity_score: f64,
}

/// Types of integration points
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntegrationPointType {
    RestAPI,
    GraphQLAPI,
    Database,
    MessageQueue,
    FileSystem,
    ExternalService,
}

// Detector trait implementation and PolyglotDetector analysis methods
include!("polyglot_detection.rs");
include!("polyglot_utilities.rs");

// Property tests for polyglot detection
include!("polyglot_tests.rs");
