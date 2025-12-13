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

#[async_trait]
impl SignalCollector for RustcCollector {
    fn source(&self) -> SignalSource {
        SignalSource::Rustc
    }

    async fn collect(&self, project_path: &Path) -> Result<Vec<SignalEvidence>> {
        let output = Command::new("cargo")
            .args(["build", "--message-format=json"])
            .current_dir(project_path)
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut signals = Vec::new();

        for line in stdout.lines() {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                if json.get("reason").and_then(|r| r.as_str()) == Some("compiler-message") {
                    if let Some(message) = json.get("message") {
                        if let Some(level) = message.get("level").and_then(|l| l.as_str()) {
                            if level == "error" {
                                let code = message
                                    .get("code")
                                    .and_then(|c| c.get("code"))
                                    .and_then(|c| c.as_str())
                                    .map(String::from);

                                let rendered = message
                                    .get("rendered")
                                    .and_then(|r| r.as_str())
                                    .unwrap_or("")
                                    .to_string();

                                signals.push(SignalEvidence {
                                    source: SignalSource::Rustc,
                                    raw_message: rendered,
                                    error_code: code,
                                    weight: 1.0,
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(signals)
    }
}

/// Collector for clippy warnings
pub struct ClippyCollector;

#[async_trait]
impl SignalCollector for ClippyCollector {
    fn source(&self) -> SignalSource {
        SignalSource::Clippy
    }

    async fn collect(&self, project_path: &Path) -> Result<Vec<SignalEvidence>> {
        let output = Command::new("cargo")
            .args(["clippy", "--message-format=json", "--", "-D", "warnings"])
            .current_dir(project_path)
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut signals = Vec::new();

        for line in stdout.lines() {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                if json.get("reason").and_then(|r| r.as_str()) == Some("compiler-message") {
                    if let Some(message) = json.get("message") {
                        if let Some(level) = message.get("level").and_then(|l| l.as_str()) {
                            if level == "warning" || level == "error" {
                                let code = message
                                    .get("code")
                                    .and_then(|c| c.get("code"))
                                    .and_then(|c| c.as_str())
                                    .map(String::from);

                                let rendered = message
                                    .get("rendered")
                                    .and_then(|r| r.as_str())
                                    .unwrap_or("")
                                    .to_string();

                                // Weight based on lint category
                                let weight = if code
                                    .as_ref()
                                    .map(|c| c.starts_with("clippy::correctness"))
                                    .unwrap_or(false)
                                {
                                    1.0
                                } else if code
                                    .as_ref()
                                    .map(|c| c.starts_with("clippy::suspicious"))
                                    .unwrap_or(false)
                                {
                                    0.9
                                } else if code
                                    .as_ref()
                                    .map(|c| c.starts_with("clippy::complexity"))
                                    .unwrap_or(false)
                                {
                                    0.7
                                } else {
                                    0.5
                                };

                                signals.push(SignalEvidence {
                                    source: SignalSource::Clippy,
                                    raw_message: rendered,
                                    error_code: code,
                                    weight,
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(signals)
    }
}

/// Collector for test failures
pub struct TestCollector;

#[async_trait]
impl SignalCollector for TestCollector {
    fn source(&self) -> SignalSource {
        SignalSource::CargoTest
    }

    async fn collect(&self, project_path: &Path) -> Result<Vec<SignalEvidence>> {
        let output = Command::new("cargo")
            .args([
                "test",
                "--no-fail-fast",
                "--",
                "--format=json",
                "-Z",
                "unstable-options",
            ])
            .current_dir(project_path)
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut signals = Vec::new();

        for line in stdout.lines() {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                if json.get("type").and_then(|t| t.as_str()) == Some("test")
                    && json.get("event").and_then(|e| e.as_str()) == Some("failed")
                {
                    let name = json
                        .get("name")
                        .and_then(|n| n.as_str())
                        .unwrap_or("unknown")
                        .to_string();

                    let stdout_text = json
                        .get("stdout")
                        .and_then(|s| s.as_str())
                        .unwrap_or("")
                        .to_string();

                    signals.push(SignalEvidence {
                        source: SignalSource::CargoTest,
                        raw_message: format!("Test failed: {}\n{}", name, stdout_text),
                        error_code: None,
                        weight: 1.0,
                    });
                }
            }
        }

        Ok(signals)
    }
}

/// Aggregated signal collector that combines multiple sources
pub struct AggregatedCollector {
    collectors: Vec<Box<dyn SignalCollector>>,
}

impl AggregatedCollector {
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
    pub fn add_collector(&mut self, collector: Box<dyn SignalCollector>) {
        self.collectors.push(collector);
    }

    /// Get the number of collectors
    pub fn collector_count(&self) -> usize {
        self.collectors.len()
    }

    /// Collect signals from all sources
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
