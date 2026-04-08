#![cfg_attr(coverage_nightly, coverage(off))]
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Serialize)]
/// Configuration for deep context.
pub struct DeepContextConfig {
    #[serde(default)]
    pub entry_points: Vec<String>,

    #[serde(default = "default_dead_code_threshold")]
    pub dead_code_threshold: f64,

    #[serde(default)]
    pub complexity_thresholds: ComplexityThresholds,

    #[serde(default)]
    pub include_tests: bool,

    #[serde(default)]
    pub include_benches: bool,

    #[serde(default)]
    pub cross_language_detection: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
/// Threshold values for complexitys.
pub struct ComplexityThresholds {
    #[serde(default = "default_cyclomatic_warning")]
    pub cyclomatic_warning: u32,

    #[serde(default = "default_cyclomatic_error")]
    pub cyclomatic_error: u32,

    #[serde(default = "default_cognitive_warning")]
    pub cognitive_warning: u32,

    #[serde(default = "default_cognitive_error")]
    pub cognitive_error: u32,
}

impl Default for ComplexityThresholds {
    fn default() -> Self {
        Self {
            cyclomatic_warning: 10,
            cyclomatic_error: 20,
            cognitive_warning: 15,
            cognitive_error: 30,
        }
    }
}

impl Default for DeepContextConfig {
    fn default() -> Self {
        Self {
            entry_points: Vec::new(),
            dead_code_threshold: 0.15,
            complexity_thresholds: ComplexityThresholds::default(),
            include_tests: false,
            include_benches: false,
            cross_language_detection: true,
        }
    }
}

// Default value functions for serde
fn default_dead_code_threshold() -> f64 {
    0.15
}

fn default_cyclomatic_warning() -> u32 {
    10
}

fn default_cyclomatic_error() -> u32 {
    20
}

fn default_cognitive_warning() -> u32 {
    15
}

fn default_cognitive_error() -> u32 {
    30
}

// Validation, detection, merge, and file I/O methods for DeepContextConfig
include!("deep_context_config_methods.rs");

// Unit tests and property tests
include!("deep_context_config_tests.rs");
