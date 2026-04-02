#![cfg_attr(coverage_nightly, coverage(off))]
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
    async fn detect(&self, input: Self::Input, config: Self::Config) -> Result<Self::Output>;

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
    detectors: std::collections::HashMap<
        String,
        Arc<
            dyn Detector<
                Input = DetectionInput,
                Output = DetectionOutput,
                Config = DetectionConfig,
            >,
        >,
    >,
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

pub use crate::contracts::OutputFormat;

/// Detector-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DetectorSpecificConfig {
    Duplicates(duplicates::DuplicateConfig),
    SATD(satd::SATDConfig),
    Polyglot(polyglot::PolyglotConfig),
}

// --- Implementation split files ---
include!("registry_impl.rs");
include!("unified_processor.rs");
include!("tests_detection.rs");
