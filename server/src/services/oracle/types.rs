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
    pub fn new(
        category: DefectCategory,
        severity: Severity,
        location: CodeLocation,
    ) -> Self {
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
            ConvergenceStatus::NotConverged { remaining: failures }
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
