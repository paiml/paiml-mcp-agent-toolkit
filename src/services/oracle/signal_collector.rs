#![cfg_attr(coverage_nightly, coverage(off))]
//! Signal collectors for gathering quality signals from various sources
//!
//! Implements Genchi Genbutsu (Go and See) - evidence from actual tools.

use super::types::*;
use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;
use std::process::Command;

/// Trait for collecting signals from a quality source
#[async_trait]
pub trait SignalCollector: Send + Sync {
    /// Source type for this collector
    fn source(&self) -> SignalSource;

    /// Collect signals from the project
    async fn collect(&self, project_path: &Path) -> Result<Vec<SignalEvidence>>;
}

/// Collector for rustc compiler errors
pub struct RustcCollector;

/// Collector for clippy warnings
pub struct ClippyCollector;

/// Collector for test failures
pub struct TestCollector;

// Collector trait implementations (RustcCollector, ClippyCollector, TestCollector)
include!("signal_collector_impls.rs");

/// Aggregated signal collector that combines multiple sources
pub struct AggregatedCollector {
    collectors: Vec<Box<dyn SignalCollector>>,
}

impl AggregatedCollector {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {
            collectors: vec![
                Box::new(RustcCollector),
                Box::new(ClippyCollector),
                Box::new(TestCollector),
            ],
        }
    }

    /// Add a custom collector
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn add_collector(&mut self, collector: Box<dyn SignalCollector>) {
        self.collectors.push(collector);
    }

    /// Get the number of collectors
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn collector_count(&self) -> usize {
        self.collectors.len()
    }

    /// Collect signals from all sources
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub async fn collect_all(&self, project_path: &Path) -> Result<Vec<SignalEvidence>> {
        let mut all_signals = Vec::new();

        for collector in &self.collectors {
            match collector.collect(project_path).await {
                Ok(signals) => all_signals.extend(signals),
                Err(e) => {
                    // Log but continue with other collectors (Jidoka - don't stop entirely)
                    eprintln!("Warning: {:?} collector failed: {}", collector.source(), e);
                }
            }
        }

        Ok(all_signals)
    }

    /// Convert signals to defect reports
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn signals_to_defects(&self, signals: Vec<SignalEvidence>) -> Vec<DefectReport> {
        let mut defects: Vec<DefectReport> = Vec::new();

        for signal in signals {
            // Try to get category from error code
            let category = signal
                .error_code
                .as_ref()
                .and_then(|code| DefectCategory::from_rustc_error(code))
                .unwrap_or(DefectCategory::Configuration);

            // Determine severity based on source
            let severity = match signal.source {
                SignalSource::Rustc => Severity::Critical,
                SignalSource::CargoTest => Severity::High,
                SignalSource::Clippy => Severity::Medium,
                _ => Severity::Low,
            };

            // Create or update defect report
            let mut defect = DefectReport::new(
                category,
                severity,
                CodeLocation {
                    file_path: std::path::PathBuf::from("unknown"),
                    line: 0,
                    column: None,
                    span_end_line: None,
                },
            );
            defect.add_signal(signal);
            defects.push(defect);
        }

        defects
    }
}

impl Default for AggregatedCollector {
    fn default() -> Self {
        Self::new()
    }
}

// Tests: collector sources, aggregation, signal evidence, defect conversion
include!("signal_collector_tests.rs");

// Tests: DefectCategory, Severity, CodeLocation, DefectReport, OracleDecision, FixType
include!("signal_collector_tests_types.rs");
