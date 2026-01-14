//! Core types for PMAT Oracle
//!
//! Unified Defect Schema (UDS) and supporting types.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

/// Defect category based on OIP CITL mappings (18 categories)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DefectCategory {
    // Memory & Concurrency
    MemorySafety,
    Concurrency,
    OwnershipBorrow,

    // Type System
    TypeErrors,
    TypeAnnotationGap,
    TraitBounds,
    OperatorPrecedence,

    // Performance & Security
    PerformanceIssues,
    Security,
    Configuration,

    // API & Integration
    ApiMisuse,
    IntegrationFailure,
    StdlibMapping,

    // Code Quality
    DocumentationGap,
    TestingGap,

    // Rust-specific
    ASTTransform,
    ComprehensionBug,
    IteratorChain,
}

impl DefectCategory {
    /// Map rustc error code to defect category
    pub fn from_rustc_error(code: &str) -> Option<Self> {
        match code {
            "E0308" | "E0412" => Some(Self::TypeErrors),
            "E0382" | "E0502" | "E0503" | "E0505" | "E0499" | "E0597" | "E0716" | "E0515" => {
                Some(Self::OwnershipBorrow)
            }
            "E0507" | "E0133" => Some(Self::MemorySafety),
            "E0277" => Some(Self::TraitBounds),
            "E0425" | "E0433" => Some(Self::StdlibMapping),
            "E0599" => Some(Self::ASTTransform),
            "E0615" => Some(Self::OperatorPrecedence),
            "E0658" => Some(Self::Configuration),
            _ => None,
        }
    }

    /// Get confidence score for this category when detected via rustc
    pub fn rustc_confidence(&self) -> f32 {
        match self {
            Self::TypeErrors => 0.95,
            Self::OwnershipBorrow => 0.92,
            Self::MemorySafety => 0.90,
            Self::TraitBounds => 0.95,
            Self::StdlibMapping => 0.85,
            Self::ASTTransform => 0.85,
            Self::OperatorPrecedence => 0.80,
            Self::Configuration => 0.75,
            _ => 0.70,
        }
    }
}

/// Severity level for defects
/// Note: Order is lowest to highest for correct Ord derivation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Severity {
    /// Minor issue, cosmetic or style
    Low,
    /// Moderate impact, workaround exists
    Medium,
    /// Major functionality impact
    High,
    /// Blocks compilation or causes runtime crash
    Critical,
}

/// Source of a quality signal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SignalSource {
    Rustc,
    Clippy,
    CargoTest,
    CargoBuild,
    LlvmCov,
    CargoMutants,
    PmatTdg,
    PmatComplexity,
    PmatSatd,
    PmatDeadCode,
    PmatRustProjectScore,
    PmatFiveWhys,
    PmatChurn,
}

/// Evidence from a signal source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalEvidence {
    pub source: SignalSource,
    pub raw_message: String,
    pub error_code: Option<String>,
    pub weight: f32,
}

/// Code location for a defect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeLocation {
    pub file_path: PathBuf,
    pub line: usize,
    pub column: Option<usize>,
    pub span_end_line: Option<usize>,
}

/// Suggested fix for a defect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedFix {
    pub description: String,
    pub confidence: f32,
    pub fix_type: FixType,
}

/// Type of fix to apply
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FixType {
    /// Use `cargo clippy --fix`
    ClippyAutoFix,
    /// Apply a unified diff patch
    DiffPatch(String),
    /// Simple text replacement
    Replacement { old: String, new: String },
    /// Insert content after a line
    InsertAfter { anchor: String, content: String },
    /// Delete lines
    DeleteLines { start: usize, end: usize },
}

/// Oracle decision for a defect
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OracleDecision {
    /// Auto-apply fix (confidence >= threshold)
    AutoApply,
    /// Queue for human review
    HumanReview,
    /// Skip (confidence too low)
    Skip,
}

/// Unified defect report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefectReport {
    pub id: String,
    pub category: DefectCategory,
    pub severity: Severity,
    pub confidence: f32,
    pub location: CodeLocation,
    pub signals: Vec<SignalEvidence>,
    pub suggested_fixes: Vec<SuggestedFix>,
    pub decision: OracleDecision,
}

impl DefectReport {
    /// Create a new defect report
    pub fn new(category: DefectCategory, severity: Severity, location: CodeLocation) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            category,
            severity,
            confidence: 0.0,
            location,
            signals: Vec::new(),
            suggested_fixes: Vec::new(),
            decision: OracleDecision::Skip,
        }
    }

    /// Add signal evidence
    pub fn add_signal(&mut self, signal: SignalEvidence) {
        self.signals.push(signal);
        self.recalculate_confidence();
    }

    /// Recalculate confidence based on signals
    ///
    /// Uses multiplicative combination: category_confidence * max_signal_weight
    /// This ensures low-weight signals properly reduce overall confidence.
    fn recalculate_confidence(&mut self) {
        if self.signals.is_empty() {
            self.confidence = 0.0;
            return;
        }

        // Get max signal weight as confidence modifier
        let max_weight = self.signals.iter().map(|s| s.weight).fold(0.0f32, f32::max);

        // Confidence = category base confidence * signal strength
        self.confidence = self.category.rustc_confidence() * max_weight;
    }

    /// Update oracle decision based on thresholds
    pub fn update_decision(&mut self, auto_apply_threshold: f32, review_threshold: f32) {
        self.decision = if self.confidence >= auto_apply_threshold {
            OracleDecision::AutoApply
        } else if self.confidence >= review_threshold {
            OracleDecision::HumanReview
        } else {
            OracleDecision::Skip
        };
    }
}

/// Convergence targets for the oracle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvergenceTargets {
    pub test_coverage: f32,
    pub mutation_score: f32,
    pub max_compiler_errors: usize,
    pub max_clippy_warnings: usize,
    pub max_test_failures: usize,
    pub min_tdg_score: f32,
    pub min_rust_project_score: u32,
    pub max_satd_markers: usize,
    pub max_dead_code: usize,
    pub max_cyclomatic_complexity: u32,
    pub max_cognitive_complexity: u32,
    pub max_build_time: Duration,
}

impl Default for ConvergenceTargets {
    fn default() -> Self {
        Self {
            test_coverage: 0.95,
            mutation_score: 0.85,
            max_compiler_errors: 0,
            max_clippy_warnings: 0,
            max_test_failures: 0,
            min_tdg_score: 95.0,
            min_rust_project_score: 90,
            max_satd_markers: 0,
            max_dead_code: 0,
            max_cyclomatic_complexity: 15,
            max_cognitive_complexity: 25,
            max_build_time: Duration::from_secs(60),
        }
    }
}

/// Current project metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectMetrics {
    pub test_coverage: f32,
    pub mutation_score: f32,
    pub compiler_errors: usize,
    pub clippy_warnings: usize,
    pub test_failures: usize,
    pub tdg_score: f32,
    pub rust_project_score: u32,
    pub satd_markers: usize,
    pub dead_code_items: usize,
    pub max_cyclomatic_complexity: u32,
    pub max_cognitive_complexity: u32,
    pub build_time: Duration,
}

/// Convergence status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConvergenceStatus {
    Converged,
    NotConverged { remaining: Vec<String> },
}

impl ConvergenceTargets {
    /// Check if metrics meet convergence criteria
    pub fn check(&self, metrics: &ProjectMetrics) -> ConvergenceStatus {
        let mut failures = Vec::new();

        if metrics.test_coverage < self.test_coverage {
            failures.push(format!(
                "Coverage: {:.1}% < {:.1}%",
                metrics.test_coverage * 100.0,
                self.test_coverage * 100.0
            ));
        }

        if metrics.mutation_score < self.mutation_score {
            failures.push(format!(
                "Mutation score: {:.1}% < {:.1}%",
                metrics.mutation_score * 100.0,
                self.mutation_score * 100.0
            ));
        }

        if metrics.compiler_errors > self.max_compiler_errors {
            failures.push(format!(
                "Compiler errors: {} > {}",
                metrics.compiler_errors, self.max_compiler_errors
            ));
        }

        if metrics.clippy_warnings > self.max_clippy_warnings {
            failures.push(format!(
                "Clippy warnings: {} > {}",
                metrics.clippy_warnings, self.max_clippy_warnings
            ));
        }

        if metrics.test_failures > self.max_test_failures {
            failures.push(format!(
                "Test failures: {} > {}",
                metrics.test_failures, self.max_test_failures
            ));
        }

        if metrics.tdg_score < self.min_tdg_score {
            failures.push(format!(
                "TDG score: {:.1} < {:.1}",
                metrics.tdg_score, self.min_tdg_score
            ));
        }

        if metrics.rust_project_score < self.min_rust_project_score {
            failures.push(format!(
                "Rust project score: {} < {}",
                metrics.rust_project_score, self.min_rust_project_score
            ));
        }

        if metrics.satd_markers > self.max_satd_markers {
            failures.push(format!(
                "SATD markers: {} > {}",
                metrics.satd_markers, self.max_satd_markers
            ));
        }

        if metrics.dead_code_items > self.max_dead_code {
            failures.push(format!(
                "Dead code items: {} > {}",
                metrics.dead_code_items, self.max_dead_code
            ));
        }

        if metrics.max_cyclomatic_complexity > self.max_cyclomatic_complexity {
            failures.push(format!(
                "Cyclomatic complexity: {} > {}",
                metrics.max_cyclomatic_complexity, self.max_cyclomatic_complexity
            ));
        }

        if metrics.max_cognitive_complexity > self.max_cognitive_complexity {
            failures.push(format!(
                "Cognitive complexity: {} > {}",
                metrics.max_cognitive_complexity, self.max_cognitive_complexity
            ));
        }

        if failures.is_empty() {
            ConvergenceStatus::Converged
        } else {
            ConvergenceStatus::NotConverged {
                remaining: failures,
            }
        }
    }
}

/// Oracle configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleConfig {
    pub max_iterations: usize,
    pub min_progress_per_iteration: f32,
    pub stagnation_threshold: usize,
    pub andon_enabled: bool,
    pub require_human_approval_above: Option<usize>,
    pub auto_apply_threshold: f32,
    pub review_threshold: f32,
    pub batch_size: usize,
}

impl Default for OracleConfig {
    fn default() -> Self {
        Self {
            max_iterations: 100,
            min_progress_per_iteration: 0.001,
            stagnation_threshold: 5,
            andon_enabled: true,
            require_human_approval_above: Some(10),
            auto_apply_threshold: 0.9,
            review_threshold: 0.7,
            batch_size: 10,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== DefectCategory Tests ====================

    #[test]
    fn test_defect_category_from_rustc_error_type_errors() {
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
    fn test_defect_category_from_rustc_error_ownership_borrow() {
        for code in ["E0382", "E0502", "E0503", "E0505", "E0499", "E0597", "E0716", "E0515"] {
            assert_eq!(
                DefectCategory::from_rustc_error(code),
                Some(DefectCategory::OwnershipBorrow),
                "Code {} should map to OwnershipBorrow",
                code
            );
        }
    }

    #[test]
    fn test_defect_category_from_rustc_error_memory_safety() {
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
    fn test_defect_category_from_rustc_error_trait_bounds() {
        assert_eq!(
            DefectCategory::from_rustc_error("E0277"),
            Some(DefectCategory::TraitBounds)
        );
    }

    #[test]
    fn test_defect_category_from_rustc_error_stdlib_mapping() {
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
    fn test_defect_category_from_rustc_error_unknown() {
        assert_eq!(DefectCategory::from_rustc_error("E9999"), None);
        assert_eq!(DefectCategory::from_rustc_error("unknown"), None);
    }

    #[test]
    fn test_defect_category_rustc_confidence() {
        assert_eq!(DefectCategory::TypeErrors.rustc_confidence(), 0.95);
        assert_eq!(DefectCategory::OwnershipBorrow.rustc_confidence(), 0.92);
        assert_eq!(DefectCategory::MemorySafety.rustc_confidence(), 0.90);
        assert_eq!(DefectCategory::TraitBounds.rustc_confidence(), 0.95);
        assert_eq!(DefectCategory::StdlibMapping.rustc_confidence(), 0.85);
        assert_eq!(DefectCategory::ASTTransform.rustc_confidence(), 0.85);
        assert_eq!(DefectCategory::OperatorPrecedence.rustc_confidence(), 0.80);
        assert_eq!(DefectCategory::Configuration.rustc_confidence(), 0.75);
        // Default for other categories
        assert_eq!(DefectCategory::Concurrency.rustc_confidence(), 0.70);
        assert_eq!(DefectCategory::PerformanceIssues.rustc_confidence(), 0.70);
    }

    // ==================== Severity Tests ====================

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Low < Severity::Medium);
        assert!(Severity::Medium < Severity::High);
        assert!(Severity::High < Severity::Critical);
    }

    #[test]
    fn test_severity_equality() {
        assert_eq!(Severity::Low, Severity::Low);
        assert_ne!(Severity::Low, Severity::High);
    }

    #[test]
    fn test_severity_serialization() {
        let severity = Severity::High;
        let json = serde_json::to_string(&severity).unwrap();
        let parsed: Severity = serde_json::from_str(&json).unwrap();
        assert_eq!(severity, parsed);
    }

    // ==================== SignalSource Tests ====================

    #[test]
    fn test_signal_source_serialization() {
        let sources = [
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
            let json = serde_json::to_string(&source).unwrap();
            let parsed: SignalSource = serde_json::from_str(&json).unwrap();
            assert_eq!(source, parsed);
        }
    }

    // ==================== SignalEvidence Tests ====================

    #[test]
    fn test_signal_evidence_creation() {
        let evidence = SignalEvidence {
            source: SignalSource::Rustc,
            raw_message: "type mismatch".to_string(),
            error_code: Some("E0308".to_string()),
            weight: 0.95,
        };

        assert_eq!(evidence.source, SignalSource::Rustc);
        assert!(evidence.error_code.is_some());
        assert_eq!(evidence.weight, 0.95);
    }

    #[test]
    fn test_signal_evidence_serialization() {
        let evidence = SignalEvidence {
            source: SignalSource::Clippy,
            raw_message: "warning".to_string(),
            error_code: None,
            weight: 0.8,
        };

        let json = serde_json::to_string(&evidence).unwrap();
        let parsed: SignalEvidence = serde_json::from_str(&json).unwrap();

        assert_eq!(evidence.source, parsed.source);
        assert_eq!(evidence.weight, parsed.weight);
    }

    // ==================== CodeLocation Tests ====================

    #[test]
    fn test_code_location_creation() {
        let location = CodeLocation {
            file_path: PathBuf::from("src/main.rs"),
            line: 42,
            column: Some(10),
            span_end_line: Some(45),
        };

        assert_eq!(location.line, 42);
        assert_eq!(location.column, Some(10));
    }

    #[test]
    fn test_code_location_serialization() {
        let location = CodeLocation {
            file_path: PathBuf::from("test.rs"),
            line: 1,
            column: None,
            span_end_line: None,
        };

        let json = serde_json::to_string(&location).unwrap();
        let parsed: CodeLocation = serde_json::from_str(&json).unwrap();

        assert_eq!(location.file_path, parsed.file_path);
        assert_eq!(location.line, parsed.line);
    }

    // ==================== FixType Tests ====================

    #[test]
    fn test_fix_type_clippy_auto() {
        let fix = FixType::ClippyAutoFix;
        let json = serde_json::to_string(&fix).unwrap();
        assert!(json.contains("ClippyAutoFix"));
    }

    #[test]
    fn test_fix_type_diff_patch() {
        let fix = FixType::DiffPatch("--- a/file\n+++ b/file".to_string());
        let json = serde_json::to_string(&fix).unwrap();
        assert!(json.contains("DiffPatch"));
    }

    #[test]
    fn test_fix_type_replacement() {
        let fix = FixType::Replacement {
            old: "old_code".to_string(),
            new: "new_code".to_string(),
        };
        let json = serde_json::to_string(&fix).unwrap();
        assert!(json.contains("Replacement"));
    }

    #[test]
    fn test_fix_type_insert_after() {
        let fix = FixType::InsertAfter {
            anchor: "fn main()".to_string(),
            content: "let x = 1;".to_string(),
        };
        let json = serde_json::to_string(&fix).unwrap();
        assert!(json.contains("InsertAfter"));
    }

    #[test]
    fn test_fix_type_delete_lines() {
        let fix = FixType::DeleteLines { start: 10, end: 20 };
        let json = serde_json::to_string(&fix).unwrap();
        assert!(json.contains("DeleteLines"));
    }

    // ==================== OracleDecision Tests ====================

    #[test]
    fn test_oracle_decision_serialization() {
        for decision in [
            OracleDecision::AutoApply,
            OracleDecision::HumanReview,
            OracleDecision::Skip,
        ] {
            let json = serde_json::to_string(&decision).unwrap();
            let parsed: OracleDecision = serde_json::from_str(&json).unwrap();
            assert_eq!(decision, parsed);
        }
    }

    // ==================== DefectReport Tests ====================

    fn create_test_location() -> CodeLocation {
        CodeLocation {
            file_path: PathBuf::from("test.rs"),
            line: 1,
            column: None,
            span_end_line: None,
        }
    }

    #[test]
    fn test_defect_report_new() {
        let report =
            DefectReport::new(DefectCategory::TypeErrors, Severity::High, create_test_location());

        assert_eq!(report.category, DefectCategory::TypeErrors);
        assert_eq!(report.severity, Severity::High);
        assert_eq!(report.confidence, 0.0);
        assert_eq!(report.decision, OracleDecision::Skip);
        assert!(report.signals.is_empty());
        assert!(report.suggested_fixes.is_empty());
    }

    #[test]
    fn test_defect_report_add_signal() {
        let mut report =
            DefectReport::new(DefectCategory::TypeErrors, Severity::High, create_test_location());

        let signal = SignalEvidence {
            source: SignalSource::Rustc,
            raw_message: "error".to_string(),
            error_code: Some("E0308".to_string()),
            weight: 1.0,
        };

        report.add_signal(signal);

        assert_eq!(report.signals.len(), 1);
        // Confidence should be recalculated
        assert!(report.confidence > 0.0);
    }

    #[test]
    fn test_defect_report_confidence_calculation() {
        let mut report =
            DefectReport::new(DefectCategory::TypeErrors, Severity::High, create_test_location());

        // TypeErrors has 0.95 base confidence
        // With weight 1.0, confidence should be 0.95
        report.add_signal(SignalEvidence {
            source: SignalSource::Rustc,
            raw_message: "error".to_string(),
            error_code: None,
            weight: 1.0,
        });

        assert!((report.confidence - 0.95).abs() < 0.01);
    }

    #[test]
    fn test_defect_report_confidence_with_low_weight() {
        let mut report =
            DefectReport::new(DefectCategory::TypeErrors, Severity::High, create_test_location());

        report.add_signal(SignalEvidence {
            source: SignalSource::Rustc,
            raw_message: "error".to_string(),
            error_code: None,
            weight: 0.5,
        });

        // 0.95 * 0.5 = 0.475
        assert!((report.confidence - 0.475).abs() < 0.01);
    }

    #[test]
    fn test_defect_report_update_decision_auto_apply() {
        let mut report =
            DefectReport::new(DefectCategory::TypeErrors, Severity::High, create_test_location());

        report.add_signal(SignalEvidence {
            source: SignalSource::Rustc,
            raw_message: "error".to_string(),
            error_code: None,
            weight: 1.0,
        });

        // With confidence 0.95, auto_apply threshold 0.9, should be AutoApply
        report.update_decision(0.9, 0.7);
        assert_eq!(report.decision, OracleDecision::AutoApply);
    }

    #[test]
    fn test_defect_report_update_decision_human_review() {
        let mut report =
            DefectReport::new(DefectCategory::TypeErrors, Severity::High, create_test_location());

        report.add_signal(SignalEvidence {
            source: SignalSource::Rustc,
            raw_message: "error".to_string(),
            error_code: None,
            weight: 0.8,
        });

        // 0.95 * 0.8 = 0.76
        report.update_decision(0.9, 0.7);
        assert_eq!(report.decision, OracleDecision::HumanReview);
    }

    #[test]
    fn test_defect_report_update_decision_skip() {
        let mut report =
            DefectReport::new(DefectCategory::TypeErrors, Severity::High, create_test_location());

        report.add_signal(SignalEvidence {
            source: SignalSource::Rustc,
            raw_message: "error".to_string(),
            error_code: None,
            weight: 0.5,
        });

        // 0.95 * 0.5 = 0.475, below review threshold
        report.update_decision(0.9, 0.7);
        assert_eq!(report.decision, OracleDecision::Skip);
    }

    // ==================== ConvergenceTargets Tests ====================

    #[test]
    fn test_convergence_targets_default() {
        let targets = ConvergenceTargets::default();

        assert_eq!(targets.test_coverage, 0.95);
        assert_eq!(targets.mutation_score, 0.85);
        assert_eq!(targets.max_compiler_errors, 0);
        assert_eq!(targets.max_clippy_warnings, 0);
        assert_eq!(targets.max_test_failures, 0);
        assert_eq!(targets.min_tdg_score, 95.0);
        assert_eq!(targets.min_rust_project_score, 90);
        assert_eq!(targets.max_satd_markers, 0);
        assert_eq!(targets.max_dead_code, 0);
        assert_eq!(targets.max_cyclomatic_complexity, 15);
        assert_eq!(targets.max_cognitive_complexity, 25);
        assert_eq!(targets.max_build_time, Duration::from_secs(60));
    }

    #[test]
    fn test_convergence_targets_check_converged() {
        let targets = ConvergenceTargets::default();
        let metrics = ProjectMetrics {
            test_coverage: 0.96,
            mutation_score: 0.86,
            compiler_errors: 0,
            clippy_warnings: 0,
            test_failures: 0,
            tdg_score: 96.0,
            rust_project_score: 91,
            satd_markers: 0,
            dead_code_items: 0,
            max_cyclomatic_complexity: 10,
            max_cognitive_complexity: 20,
            build_time: Duration::from_secs(30),
        };

        let status = targets.check(&metrics);
        match status {
            ConvergenceStatus::Converged => (),
            ConvergenceStatus::NotConverged { remaining } => {
                panic!("Should be converged, but got: {:?}", remaining);
            }
        }
    }

    #[test]
    fn test_convergence_targets_check_not_converged_coverage() {
        let targets = ConvergenceTargets::default();
        let metrics = ProjectMetrics {
            test_coverage: 0.80, // Below target
            ..Default::default()
        };

        let status = targets.check(&metrics);
        match status {
            ConvergenceStatus::Converged => panic!("Should not be converged"),
            ConvergenceStatus::NotConverged { remaining } => {
                assert!(remaining.iter().any(|s| s.contains("Coverage")));
            }
        }
    }

    #[test]
    fn test_convergence_targets_check_multiple_failures() {
        let targets = ConvergenceTargets::default();
        let metrics = ProjectMetrics {
            test_coverage: 0.80,
            compiler_errors: 5,
            tdg_score: 50.0,
            ..Default::default()
        };

        let status = targets.check(&metrics);
        match status {
            ConvergenceStatus::Converged => panic!("Should not be converged"),
            ConvergenceStatus::NotConverged { remaining } => {
                assert!(remaining.len() >= 3);
            }
        }
    }

    // ==================== OracleConfig Tests ====================

    #[test]
    fn test_oracle_config_default() {
        let config = OracleConfig::default();

        assert_eq!(config.max_iterations, 100);
        assert_eq!(config.min_progress_per_iteration, 0.001);
        assert_eq!(config.stagnation_threshold, 5);
        assert!(config.andon_enabled);
        assert_eq!(config.require_human_approval_above, Some(10));
        assert_eq!(config.auto_apply_threshold, 0.9);
        assert_eq!(config.review_threshold, 0.7);
        assert_eq!(config.batch_size, 10);
    }

    #[test]
    fn test_oracle_config_serialization() {
        let config = OracleConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let parsed: OracleConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.max_iterations, parsed.max_iterations);
        assert_eq!(config.batch_size, parsed.batch_size);
    }

    // ==================== SuggestedFix Tests ====================

    #[test]
    fn test_suggested_fix_creation() {
        let fix = SuggestedFix {
            description: "Apply clippy suggestion".to_string(),
            confidence: 0.95,
            fix_type: FixType::ClippyAutoFix,
        };

        assert_eq!(fix.confidence, 0.95);
    }

    #[test]
    fn test_suggested_fix_serialization() {
        let fix = SuggestedFix {
            description: "Fix it".to_string(),
            confidence: 0.8,
            fix_type: FixType::Replacement {
                old: "a".to_string(),
                new: "b".to_string(),
            },
        };

        let json = serde_json::to_string(&fix).unwrap();
        let parsed: SuggestedFix = serde_json::from_str(&json).unwrap();

        assert_eq!(fix.description, parsed.description);
        assert_eq!(fix.confidence, parsed.confidence);
    }

    // ==================== ProjectMetrics Tests ====================

    #[test]
    fn test_project_metrics_default() {
        let metrics = ProjectMetrics::default();

        assert_eq!(metrics.test_coverage, 0.0);
        assert_eq!(metrics.mutation_score, 0.0);
        assert_eq!(metrics.compiler_errors, 0);
        assert_eq!(metrics.clippy_warnings, 0);
        assert_eq!(metrics.tdg_score, 0.0);
        assert_eq!(metrics.build_time, Duration::default());
    }

    #[test]
    fn test_project_metrics_serialization() {
        let metrics = ProjectMetrics {
            test_coverage: 0.85,
            mutation_score: 0.75,
            compiler_errors: 2,
            clippy_warnings: 5,
            test_failures: 1,
            tdg_score: 80.0,
            rust_project_score: 85,
            satd_markers: 3,
            dead_code_items: 2,
            max_cyclomatic_complexity: 12,
            max_cognitive_complexity: 18,
            build_time: Duration::from_secs(45),
        };

        let json = serde_json::to_string(&metrics).unwrap();
        let parsed: ProjectMetrics = serde_json::from_str(&json).unwrap();

        assert_eq!(metrics.test_coverage, parsed.test_coverage);
        assert_eq!(metrics.compiler_errors, parsed.compiler_errors);
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_defect_report_full_workflow() {
        let mut report = DefectReport::new(
            DefectCategory::OwnershipBorrow,
            Severity::High,
            CodeLocation {
                file_path: PathBuf::from("src/lib.rs"),
                line: 42,
                column: Some(10),
                span_end_line: Some(42),
            },
        );

        // Add signals from multiple sources
        report.add_signal(SignalEvidence {
            source: SignalSource::Rustc,
            raw_message: "cannot move out of `x` because it is borrowed".to_string(),
            error_code: Some("E0505".to_string()),
            weight: 1.0,
        });

        report.add_signal(SignalEvidence {
            source: SignalSource::PmatComplexity,
            raw_message: "High complexity at this location".to_string(),
            error_code: None,
            weight: 0.7,
        });

        // Add suggested fix
        report.suggested_fixes.push(SuggestedFix {
            description: "Clone the value before borrowing".to_string(),
            confidence: 0.85,
            fix_type: FixType::Replacement {
                old: "let y = &x;".to_string(),
                new: "let y = x.clone();".to_string(),
            },
        });

        // Update decision
        report.update_decision(0.9, 0.7);

        // OwnershipBorrow has 0.92 confidence * max_weight(1.0) = 0.92
        assert!(report.confidence > 0.9);
        assert_eq!(report.decision, OracleDecision::AutoApply);
        assert_eq!(report.signals.len(), 2);
        assert_eq!(report.suggested_fixes.len(), 1);
    }
}
