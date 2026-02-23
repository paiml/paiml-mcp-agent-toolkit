//! Types, enums, and structs for the SATD detection system.

use regex::RegexSet;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

/// Self-Admitted Technical Debt detector with pattern matching
pub struct SATDDetector {
    #[allow(dead_code)]
    pub(crate) patterns: RegexSet,
    pub(crate) debt_classifier: DebtClassifier,
}

/// Detected technical debt item
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TechnicalDebt {
    pub category: DebtCategory,
    pub severity: Severity,
    pub text: String,
    pub file: PathBuf,
    pub line: u32,
    pub column: u32,
    pub context_hash: [u8; 16], // BLAKE3 hash for identity tracking
}

/// SATD analysis result containing all detected debt items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SATDAnalysisResult {
    pub items: Vec<TechnicalDebt>,
    pub summary: SATDSummary,
    pub total_files_analyzed: usize,
    pub files_with_debt: usize,
    pub analysis_timestamp: chrono::DateTime<chrono::Utc>,
}

/// Summary statistics for SATD analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SATDSummary {
    pub total_items: usize,
    pub by_severity: std::collections::HashMap<String, usize>,
    pub by_category: std::collections::HashMap<String, usize>,
    pub files_with_satd: usize,
    pub avg_age_days: f64,
}

/// Test-only structures for SATD metrics
#[cfg(test)]
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct DebtFileMetrics {
    pub(crate) file: PathBuf,
    pub(crate) count: usize,
    pub(crate) critical_count: usize,
    pub(crate) categories: Vec<String>,
    pub(crate) lines: Vec<usize>,
}

#[cfg(test)]
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct DebtCategoryMetrics {
    pub(crate) count: usize,
    pub(crate) critical_count: usize,
    pub(crate) files: Vec<PathBuf>,
}

/// Categories of technical debt
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DebtCategory {
    Design,      // HACK, KLUDGE, SMELL - Architectural compromises
    Defect,      // BUG, FIXME, BROKEN - Known defects
    Requirement, // TODO, FEAT, ENHANCEMENT - Missing features
    Test,        // FAILING, SKIP, DISABLED - Test debt
    Performance, // SLOW, OPTIMIZE, PERF - Performance issues
    Security,    // SECURITY, VULN, UNSAFE - Security concerns
}

impl DebtCategory {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            DebtCategory::Design => "Design",
            DebtCategory::Defect => "Defect",
            DebtCategory::Requirement => "Requirement",
            DebtCategory::Test => "Test",
            DebtCategory::Performance => "Performance",
            DebtCategory::Security => "Security",
        }
    }
}

impl std::fmt::Display for DebtCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Severity levels for technical debt
/// EXTREME TDD FIX: Reordered Low->Critical for correct derive(Ord) behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Severity {
    Low,      // TODOs, minor enhancements
    Medium,   // Design issues, performance problems
    High,     // Defects, broken functionality
    Critical, // Security vulnerabilities, data loss risks
}

impl Severity {
    /// Escalate severity by one level
    ///
    /// # Examples
    ///
    /// ```
    /// use pmat::services::satd_detector::Severity;
    ///
    /// assert_eq!(Severity::Low.escalate(), Severity::Medium);
    /// assert_eq!(Severity::Medium.escalate(), Severity::High);
    /// assert_eq!(Severity::High.escalate(), Severity::Critical);
    /// assert_eq!(Severity::Critical.escalate(), Severity::Critical); // Already at max
    /// ```
    #[must_use]
    pub fn escalate(self) -> Self {
        match self {
            Severity::Low => Severity::Medium,
            Severity::Medium => Severity::High,
            Severity::High => Severity::Critical,
            Severity::Critical => Severity::Critical,
        }
    }

    /// Reduce severity by one level
    ///
    /// # Examples
    ///
    /// ```
    /// use pmat::services::satd_detector::Severity;
    ///
    /// assert_eq!(Severity::Critical.reduce(), Severity::High);
    /// assert_eq!(Severity::High.reduce(), Severity::Medium);
    /// assert_eq!(Severity::Medium.reduce(), Severity::Low);
    /// assert_eq!(Severity::Low.reduce(), Severity::Low); // Already at min
    /// ```
    #[must_use]
    pub fn reduce(self) -> Self {
        match self {
            Severity::Critical => Severity::High,
            Severity::High => Severity::Medium,
            Severity::Medium => Severity::Low,
            Severity::Low => Severity::Low,
        }
    }
}

/// Context information for debt classification
#[derive(Debug, Clone)]
pub struct AstContext {
    pub node_type: AstNodeType,
    pub parent_function: String,
    pub complexity: u32,
    pub siblings_count: usize,
    pub nesting_depth: usize,
    pub surrounding_statements: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AstNodeType {
    SecurityFunction,
    DataValidation,
    TestFunction,
    MockImplementation,
    Regular,
}

/// Pattern-based debt classifier
pub struct DebtClassifier {
    pub(crate) patterns: Vec<DebtPattern>,
    pub(crate) compiled_patterns: RegexSet,
}

#[derive(Debug, Clone)]
pub(crate) struct DebtPattern {
    pub(crate) regex: String,
    pub(crate) category: DebtCategory,
    pub(crate) severity: Severity,
    #[allow(dead_code)]
    pub(crate) description: String,
}

/// Evolution tracking for technical debt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebtEvolution {
    pub total_introduced: usize,
    pub total_resolved: usize,
    pub current_debt_age_p50: f64,
    pub debt_velocity: f64,
}

/// Project-wide SATD metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SATDMetrics {
    pub total_debts: usize,
    pub debt_density_per_kloc: f64,
    pub by_category: BTreeMap<String, CategoryMetrics>,
    pub critical_debts: Vec<TechnicalDebt>,
    pub debt_age_distribution: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryMetrics {
    pub count: usize,
    pub files: BTreeSet<String>,
    pub avg_severity: f64,
}

/// Tracks test block boundaries in Rust files to exclude test-only technical debt
pub(crate) struct TestBlockTracker {
    is_rust_file: bool,
    in_test_block: bool,
    test_block_depth: usize,
}

impl TestBlockTracker {
    pub(crate) fn new(is_rust_file: bool) -> Self {
        Self {
            is_rust_file,
            in_test_block: false,
            test_block_depth: 0,
        }
    }

    pub(crate) fn update_from_line(&mut self, trimmed_line: &str) {
        if !self.is_rust_file {
            return;
        }

        if self.is_test_block_start(trimmed_line) {
            self.start_test_block();
        } else if self.in_test_block {
            self.update_test_block_depth(trimmed_line);
        }
    }

    pub(crate) fn is_in_test_block(&self) -> bool {
        self.in_test_block
    }

    fn is_test_block_start(&self, trimmed_line: &str) -> bool {
        trimmed_line.starts_with("#[cfg(test)]")
    }

    fn start_test_block(&mut self) {
        self.in_test_block = true;
        self.test_block_depth = 0;
    }

    fn update_test_block_depth(&mut self, trimmed_line: &str) {
        self.add_opening_braces(trimmed_line);
        self.subtract_closing_braces(trimmed_line);
    }

    fn add_opening_braces(&mut self, trimmed_line: &str) {
        if trimmed_line.contains('{') {
            self.test_block_depth += trimmed_line.matches('{').count();
        }
    }

    fn subtract_closing_braces(&mut self, trimmed_line: &str) {
        if trimmed_line.contains('}') {
            self.test_block_depth = self
                .test_block_depth
                .saturating_sub(trimmed_line.matches('}').count());

            if self.test_block_depth == 0 && trimmed_line.ends_with('}') {
                self.in_test_block = false;
            }
        }
    }
}

/// Toyota Way: Data-Driven Design - encapsulate project analysis state
#[derive(Default)]
pub(crate) struct ProjectAnalysisStats {
    pub(crate) all_debts: Vec<TechnicalDebt>,
    pub(crate) files_with_debt: usize,
    pub(crate) total_files_analyzed: usize,
}

impl ProjectAnalysisStats {
    pub(crate) fn new() -> Self {
        Self::default()
    }
}
