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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;
    use std::path::PathBuf;

    // ==================== SignalSource Tests ====================

    #[test]
    fn test_rustc_collector_source() {
        let collector = RustcCollector;
        assert_eq!(collector.source(), SignalSource::Rustc);
    }

    #[test]
    fn test_clippy_collector_source() {
        let collector = ClippyCollector;
        assert_eq!(collector.source(), SignalSource::Clippy);
    }

    #[test]
    fn test_test_collector_source() {
        let collector = TestCollector;
        assert_eq!(collector.source(), SignalSource::CargoTest);
    }

    // ==================== AggregatedCollector Tests ====================

    #[test]
    fn test_aggregated_collector_new() {
        let collector = AggregatedCollector::new();
        // Should have 3 default collectors: Rustc, Clippy, Test
        assert_eq!(collector.collector_count(), 3);
    }

    #[test]
    fn test_aggregated_collector_default() {
        let collector = AggregatedCollector::default();
        assert_eq!(collector.collector_count(), 3);
    }

    #[test]
    fn test_aggregated_collector_add_collector() {
        let mut collector = AggregatedCollector::new();
        assert_eq!(collector.collector_count(), 3);

        // Add custom collector
        collector.add_collector(Box::new(RustcCollector));
        assert_eq!(collector.collector_count(), 4);
    }

    // ==================== SignalEvidence Tests ====================

    #[test]
    fn test_signal_evidence_creation() {
        let signal = SignalEvidence {
            source: SignalSource::Rustc,
            raw_message: "error[E0308]: mismatched types".to_string(),
            error_code: Some("E0308".to_string()),
            weight: 1.0,
        };

        assert_eq!(signal.source, SignalSource::Rustc);
        assert!(signal.raw_message.contains("mismatched"));
        assert_eq!(signal.error_code, Some("E0308".to_string()));
        assert!((signal.weight - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_signal_evidence_serialization() {
        let signal = SignalEvidence {
            source: SignalSource::Clippy,
            raw_message: "warning: unused variable".to_string(),
            error_code: Some("clippy::unused".to_string()),
            weight: 0.5,
        };

        let serialized = serde_json::to_string(&signal).expect("Should serialize");
        let deserialized: SignalEvidence =
            serde_json::from_str(&serialized).expect("Should deserialize");

        assert_eq!(signal.source, deserialized.source);
        assert_eq!(signal.raw_message, deserialized.raw_message);
        assert_eq!(signal.error_code, deserialized.error_code);
        assert!((signal.weight - deserialized.weight).abs() < f32::EPSILON);
    }

    // ==================== signals_to_defects Tests ====================

    #[test]
    fn test_signals_to_defects_empty() {
        let collector = AggregatedCollector::new();
        let signals: Vec<SignalEvidence> = vec![];
        let defects = collector.signals_to_defects(signals);
        assert!(defects.is_empty());
    }

    #[test]
    fn test_signals_to_defects_rustc_error() {
        let collector = AggregatedCollector::new();
        let signals = vec![SignalEvidence {
            source: SignalSource::Rustc,
            raw_message: "error[E0308]: mismatched types".to_string(),
            error_code: Some("E0308".to_string()),
            weight: 1.0,
        }];

        let defects = collector.signals_to_defects(signals);
        assert_eq!(defects.len(), 1);
        assert_eq!(defects[0].category, DefectCategory::TypeErrors);
        assert_eq!(defects[0].severity, Severity::Critical);
    }

    #[test]
    fn test_signals_to_defects_clippy_warning() {
        let collector = AggregatedCollector::new();
        let signals = vec![SignalEvidence {
            source: SignalSource::Clippy,
            raw_message: "warning: unused variable".to_string(),
            error_code: Some("clippy::unused_variable".to_string()),
            weight: 0.5,
        }];

        let defects = collector.signals_to_defects(signals);
        assert_eq!(defects.len(), 1);
        assert_eq!(defects[0].severity, Severity::Medium);
    }

    #[test]
    fn test_signals_to_defects_test_failure() {
        let collector = AggregatedCollector::new();
        let signals = vec![SignalEvidence {
            source: SignalSource::CargoTest,
            raw_message: "Test failed: test_something".to_string(),
            error_code: None,
            weight: 1.0,
        }];

        let defects = collector.signals_to_defects(signals);
        assert_eq!(defects.len(), 1);
        assert_eq!(defects[0].severity, Severity::High);
    }

    #[test]
    fn test_signals_to_defects_multiple_signals() {
        let collector = AggregatedCollector::new();
        let signals = vec![
            SignalEvidence {
                source: SignalSource::Rustc,
                raw_message: "error[E0308]: mismatched types".to_string(),
                error_code: Some("E0308".to_string()),
                weight: 1.0,
            },
            SignalEvidence {
                source: SignalSource::Clippy,
                raw_message: "warning: unused variable".to_string(),
                error_code: None,
                weight: 0.5,
            },
            SignalEvidence {
                source: SignalSource::CargoTest,
                raw_message: "Test failed: test_foo".to_string(),
                error_code: None,
                weight: 1.0,
            },
        ];

        let defects = collector.signals_to_defects(signals);
        assert_eq!(defects.len(), 3);
    }

    #[test]
    fn test_signals_to_defects_ownership_error() {
        let collector = AggregatedCollector::new();
        let signals = vec![SignalEvidence {
            source: SignalSource::Rustc,
            raw_message: "error[E0382]: borrow of moved value".to_string(),
            error_code: Some("E0382".to_string()),
            weight: 1.0,
        }];

        let defects = collector.signals_to_defects(signals);
        assert_eq!(defects.len(), 1);
        assert_eq!(defects[0].category, DefectCategory::OwnershipBorrow);
    }

    #[test]
    fn test_signals_to_defects_unknown_error_code() {
        let collector = AggregatedCollector::new();
        let signals = vec![SignalEvidence {
            source: SignalSource::Rustc,
            raw_message: "error[E9999]: unknown error".to_string(),
            error_code: Some("E9999".to_string()),
            weight: 1.0,
        }];

        let defects = collector.signals_to_defects(signals);
        assert_eq!(defects.len(), 1);
        // Unknown error code should fallback to Configuration category
        assert_eq!(defects[0].category, DefectCategory::Configuration);
    }

    #[test]
    fn test_signals_to_defects_no_error_code() {
        let collector = AggregatedCollector::new();
        let signals = vec![SignalEvidence {
            source: SignalSource::Rustc,
            raw_message: "error: something went wrong".to_string(),
            error_code: None,
            weight: 1.0,
        }];

        let defects = collector.signals_to_defects(signals);
        assert_eq!(defects.len(), 1);
        // No error code should fallback to Configuration category
        assert_eq!(defects[0].category, DefectCategory::Configuration);
    }

    // ==================== Clippy Weight Tests ====================

    #[test]
    fn test_clippy_correctness_weight() {
        let collector = AggregatedCollector::new();
        let signals = vec![SignalEvidence {
            source: SignalSource::Clippy,
            raw_message: "error: clippy correctness issue".to_string(),
            error_code: Some("clippy::correctness::something".to_string()),
            weight: 1.0,
        }];

        let defects = collector.signals_to_defects(signals);
        assert_eq!(defects.len(), 1);
        // Correctness issues should have weight 1.0
        assert!((defects[0].signals[0].weight - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_clippy_suspicious_weight() {
        // The weight is already set in the signal, not modified by signals_to_defects
        // This test verifies the signal structure is preserved
        let collector = AggregatedCollector::new();
        let signals = vec![SignalEvidence {
            source: SignalSource::Clippy,
            raw_message: "warning: suspicious pattern".to_string(),
            error_code: Some("clippy::suspicious::something".to_string()),
            weight: 0.9,
        }];

        let defects = collector.signals_to_defects(signals);
        assert_eq!(defects.len(), 1);
        assert!((defects[0].signals[0].weight - 0.9).abs() < f32::EPSILON);
    }

    // ==================== DefectCategory Error Code Mapping Tests ====================

    #[test]
    fn test_defect_category_from_type_error() {
        assert_eq!(
            DefectCategory::from_rustc_error("E0308"),
            Some(DefectCategory::TypeErrors)
        );
        assert_eq!(
            DefectCategory::from_rustc_error("E0412"),
            Some(DefectCategory::TypeErrors)
        );
    }

    #[test]
    fn test_defect_category_from_ownership_error() {
        assert_eq!(
            DefectCategory::from_rustc_error("E0382"),
            Some(DefectCategory::OwnershipBorrow)
        );
        assert_eq!(
            DefectCategory::from_rustc_error("E0502"),
            Some(DefectCategory::OwnershipBorrow)
        );
        assert_eq!(
            DefectCategory::from_rustc_error("E0505"),
            Some(DefectCategory::OwnershipBorrow)
        );
    }

    #[test]
    fn test_defect_category_from_memory_safety_error() {
        assert_eq!(
            DefectCategory::from_rustc_error("E0507"),
            Some(DefectCategory::MemorySafety)
        );
        assert_eq!(
            DefectCategory::from_rustc_error("E0133"),
            Some(DefectCategory::MemorySafety)
        );
    }

    #[test]
    fn test_defect_category_from_trait_bounds_error() {
        assert_eq!(
            DefectCategory::from_rustc_error("E0277"),
            Some(DefectCategory::TraitBounds)
        );
    }

    #[test]
    fn test_defect_category_from_stdlib_mapping_error() {
        assert_eq!(
            DefectCategory::from_rustc_error("E0425"),
            Some(DefectCategory::StdlibMapping)
        );
        assert_eq!(
            DefectCategory::from_rustc_error("E0433"),
            Some(DefectCategory::StdlibMapping)
        );
    }

    #[test]
    fn test_defect_category_from_unknown_error() {
        assert_eq!(DefectCategory::from_rustc_error("E9999"), None);
        assert_eq!(DefectCategory::from_rustc_error(""), None);
        assert_eq!(DefectCategory::from_rustc_error("invalid"), None);
    }

    // ==================== DefectCategory Confidence Tests ====================

    #[test]
    fn test_defect_category_rustc_confidence() {
        assert!((DefectCategory::TypeErrors.rustc_confidence() - 0.95).abs() < f32::EPSILON);
        assert!((DefectCategory::OwnershipBorrow.rustc_confidence() - 0.92).abs() < f32::EPSILON);
        assert!((DefectCategory::MemorySafety.rustc_confidence() - 0.90).abs() < f32::EPSILON);
        assert!((DefectCategory::TraitBounds.rustc_confidence() - 0.95).abs() < f32::EPSILON);
        assert!((DefectCategory::StdlibMapping.rustc_confidence() - 0.85).abs() < f32::EPSILON);
        assert!((DefectCategory::Concurrency.rustc_confidence() - 0.70).abs() < f32::EPSILON);
    }

    // ==================== CodeLocation Tests ====================

    #[test]
    fn test_code_location_creation() {
        let location = CodeLocation {
            file_path: PathBuf::from("/src/main.rs"),
            line: 42,
            column: Some(10),
            span_end_line: Some(45),
        };

        assert_eq!(location.file_path, PathBuf::from("/src/main.rs"));
        assert_eq!(location.line, 42);
        assert_eq!(location.column, Some(10));
        assert_eq!(location.span_end_line, Some(45));
    }

    #[test]
    fn test_code_location_serialization() {
        let location = CodeLocation {
            file_path: PathBuf::from("/src/lib.rs"),
            line: 100,
            column: None,
            span_end_line: None,
        };

        let serialized = serde_json::to_string(&location).expect("Should serialize");
        let deserialized: CodeLocation =
            serde_json::from_str(&serialized).expect("Should deserialize");

        assert_eq!(location.file_path, deserialized.file_path);
        assert_eq!(location.line, deserialized.line);
        assert_eq!(location.column, deserialized.column);
    }

    // ==================== Severity Tests ====================

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Low < Severity::Medium);
        assert!(Severity::Medium < Severity::High);
        assert!(Severity::High < Severity::Critical);
    }

    #[test]
    fn test_severity_serialization() {
        let severities = vec![
            Severity::Low,
            Severity::Medium,
            Severity::High,
            Severity::Critical,
        ];

        for severity in severities {
            let serialized = serde_json::to_string(&severity).expect("Should serialize");
            let deserialized: Severity =
                serde_json::from_str(&serialized).expect("Should deserialize");
            assert_eq!(severity, deserialized);
        }
    }

    // ==================== SignalSource Tests ====================

    #[test]
    fn test_signal_source_variants() {
        let sources = vec![
            SignalSource::Rustc,
            SignalSource::Clippy,
            SignalSource::CargoTest,
            SignalSource::CargoBuild,
            SignalSource::LlvmCov,
            SignalSource::CargoMutants,
            SignalSource::PmatTdg,
            SignalSource::PmatComplexity,
            SignalSource::PmatSatd,
            SignalSource::PmatDeadCode,
            SignalSource::PmatRustProjectScore,
            SignalSource::PmatFiveWhys,
            SignalSource::PmatChurn,
        ];

        for source in sources {
            let serialized = serde_json::to_string(&source).expect("Should serialize");
            let deserialized: SignalSource =
                serde_json::from_str(&serialized).expect("Should deserialize");
            assert_eq!(source, deserialized);
        }
    }

    // ==================== DefectReport Tests ====================

    #[test]
    fn test_defect_report_new() {
        let location = CodeLocation {
            file_path: PathBuf::from("/src/main.rs"),
            line: 10,
            column: None,
            span_end_line: None,
        };
        let report = DefectReport::new(DefectCategory::TypeErrors, Severity::High, location);

        assert!(!report.id.is_empty()); // UUID generated
        assert_eq!(report.category, DefectCategory::TypeErrors);
        assert_eq!(report.severity, Severity::High);
        assert!((report.confidence - 0.0).abs() < f32::EPSILON);
        assert!(report.signals.is_empty());
        assert!(report.suggested_fixes.is_empty());
        assert_eq!(report.decision, OracleDecision::Skip);
    }

    #[test]
    fn test_defect_report_add_signal() {
        let location = CodeLocation {
            file_path: PathBuf::from("/src/lib.rs"),
            line: 20,
            column: None,
            span_end_line: None,
        };
        let mut report = DefectReport::new(DefectCategory::TypeErrors, Severity::High, location);

        let signal = SignalEvidence {
            source: SignalSource::Rustc,
            raw_message: "error".to_string(),
            error_code: Some("E0308".to_string()),
            weight: 1.0,
        };
        report.add_signal(signal);

        assert_eq!(report.signals.len(), 1);
        // Confidence should be category confidence * max signal weight
        // TypeErrors confidence = 0.95, signal weight = 1.0
        assert!((report.confidence - 0.95).abs() < 0.01);
    }

    #[test]
    fn test_defect_report_update_decision_auto_apply() {
        let location = CodeLocation {
            file_path: PathBuf::from("/src/main.rs"),
            line: 1,
            column: None,
            span_end_line: None,
        };
        let mut report =
            DefectReport::new(DefectCategory::TypeErrors, Severity::Critical, location);

        // Add signal to set confidence
        let signal = SignalEvidence {
            source: SignalSource::Rustc,
            raw_message: "error".to_string(),
            error_code: Some("E0308".to_string()),
            weight: 1.0,
        };
        report.add_signal(signal);

        // Update decision with low thresholds
        report.update_decision(0.9, 0.7);

        // TypeErrors confidence (0.95) >= 0.9, should be AutoApply
        assert_eq!(report.decision, OracleDecision::AutoApply);
    }

    #[test]
    fn test_defect_report_update_decision_human_review() {
        let location = CodeLocation {
            file_path: PathBuf::from("/src/main.rs"),
            line: 1,
            column: None,
            span_end_line: None,
        };
        let mut report =
            DefectReport::new(DefectCategory::Configuration, Severity::Medium, location);

        // Add signal with lower weight
        let signal = SignalEvidence {
            source: SignalSource::Clippy,
            raw_message: "warning".to_string(),
            error_code: None,
            weight: 0.9,
        };
        report.add_signal(signal);

        // Update decision
        report.update_decision(0.9, 0.5);

        // Configuration confidence (0.75) * 0.9 = 0.675, between 0.5 and 0.9
        assert_eq!(report.decision, OracleDecision::HumanReview);
    }

    #[test]
    fn test_defect_report_update_decision_skip() {
        let location = CodeLocation {
            file_path: PathBuf::from("/src/main.rs"),
            line: 1,
            column: None,
            span_end_line: None,
        };
        let mut report = DefectReport::new(DefectCategory::Configuration, Severity::Low, location);

        // Add signal with low weight
        let signal = SignalEvidence {
            source: SignalSource::Clippy,
            raw_message: "warning".to_string(),
            error_code: None,
            weight: 0.3,
        };
        report.add_signal(signal);

        // Update decision with high thresholds
        report.update_decision(0.9, 0.7);

        // Configuration confidence (0.75) * 0.3 = 0.225, below 0.7
        assert_eq!(report.decision, OracleDecision::Skip);
    }

    // ==================== OracleDecision Tests ====================

    #[test]
    fn test_oracle_decision_serialization() {
        let decisions = vec![
            OracleDecision::AutoApply,
            OracleDecision::HumanReview,
            OracleDecision::Skip,
        ];

        for decision in decisions {
            let serialized = serde_json::to_string(&decision).expect("Should serialize");
            let deserialized: OracleDecision =
                serde_json::from_str(&serialized).expect("Should deserialize");
            assert_eq!(decision, deserialized);
        }
    }

    // ==================== FixType Tests ====================

    #[test]
    fn test_fix_type_clippy_auto_fix() {
        let fix = SuggestedFix {
            description: "Apply clippy fix".to_string(),
            confidence: 0.95,
            fix_type: FixType::ClippyAutoFix,
        };

        assert_eq!(fix.description, "Apply clippy fix");
        assert!((fix.confidence - 0.95).abs() < f32::EPSILON);
    }

    #[test]
    fn test_fix_type_replacement() {
        let fix = SuggestedFix {
            description: "Replace old with new".to_string(),
            confidence: 0.8,
            fix_type: FixType::Replacement {
                old: "old_code".to_string(),
                new: "new_code".to_string(),
            },
        };

        if let FixType::Replacement { old, new } = &fix.fix_type {
            assert_eq!(old, "old_code");
            assert_eq!(new, "new_code");
        } else {
            panic!("Expected Replacement fix type");
        }
    }

    #[test]
    fn test_fix_type_serialization() {
        let fix = SuggestedFix {
            description: "Test fix".to_string(),
            confidence: 0.7,
            fix_type: FixType::DiffPatch("@@ -1,3 +1,3 @@\n-old\n+new".to_string()),
        };

        let serialized = serde_json::to_string(&fix).expect("Should serialize");
        let deserialized: SuggestedFix =
            serde_json::from_str(&serialized).expect("Should deserialize");

        assert_eq!(fix.description, deserialized.description);
    }
}
