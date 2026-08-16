#![allow(unused)]
//! Types, enums, and structs for the SATD detection system.

use regex::RegexSet;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

/// Self-Admitted Technical Debt detector with pattern matching
pub struct SATDDetector {
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
    /// Files found and deliberately not read, by reason — the denominator that
    /// tells "measured clean" apart from "measured almost nothing".
    #[serde(default)]
    pub skipped: SkipCounts,
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
#[derive(Debug, Clone)]
pub(crate) struct DebtFileMetrics {
    pub(crate) file: PathBuf,
    pub(crate) count: usize,
    pub(crate) critical_count: usize,
    pub(crate) categories: Vec<String>,
    pub(crate) lines: Vec<usize>,
}

#[cfg(test)]
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
/// Type classification for ast node.
pub enum AstNodeType {
    SecurityFunction,
    DataValidation,
    TestFunction,
    MockImplementation,
    Regular,
}

/// Marker-based debt classifier.
///
/// The marker table lives in `classifier.rs`; the only per-instance state is
/// how demanding the marker match is, plus the prose-admission phrases (and,
/// in extended mode, the euphemisms of #149).
pub struct DebtClassifier {
    pub(crate) mode: super::classifier::MarkerMode,
    pub(crate) phrases: Vec<DebtPattern>,
    pub(crate) compiled_phrases: RegexSet,
}

#[derive(Debug, Clone)]
pub(crate) struct DebtPattern {
    pub(crate) regex: String,
    pub(crate) category: DebtCategory,
    pub(crate) severity: Severity,

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
/// Category metrics.
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn new(is_rust_file: bool) -> Self {
        Self {
            is_rust_file,
            in_test_block: false,
            test_block_depth: 0,
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
    /// Files the walk found and then declined to read, by reason.
    ///
    /// #923 stopped counting these as *analysed*, which was right — a file
    /// nothing can be reported from was not analysed. But the report then said
    /// nothing about them at all, so "SATD: 0" read identically whether the
    /// tree was clean or whether every candidate in it had been skipped. The
    /// counts are carried out to the report so a reader can see the scope the
    /// number was measured over.
    pub(crate) skipped: SkipCounts,
}

/// Why files were not read, so an absent finding can be told apart from an
/// absent measurement.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkipCounts {
    /// Test files, when `--include-tests` was not given.
    pub tests: usize,
    /// `examples/`, `demo/`, fuzz targets, generated and vendored files.
    pub out_of_scope: usize,
    /// Minified or vendored bundles.
    pub minified_or_vendor: usize,
    /// Files past the large-file threshold.
    pub too_large: usize,
}

/// One reason a file was not read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SkipReason {
    Test,
    OutOfScope,
    MinifiedOrVendor,
    TooLarge,
}

impl SkipReason {
    pub(crate) fn record(self, counts: &mut SkipCounts) {
        match self {
            Self::Test => counts.tests += 1,
            Self::OutOfScope => counts.out_of_scope += 1,
            Self::MinifiedOrVendor => counts.minified_or_vendor += 1,
            Self::TooLarge => counts.too_large += 1,
        }
    }
}

impl SkipCounts {
    #[must_use]
    pub fn total(&self) -> usize {
        self.tests + self.out_of_scope + self.minified_or_vendor + self.too_large
    }

    /// A one-line note for the human-readable report, or `None` when nothing
    /// was skipped and there is therefore nothing to disclose.
    #[must_use]
    pub fn note(&self) -> Option<String> {
        if self.total() == 0 {
            return None;
        }
        let mut parts = Vec::new();
        if self.tests > 0 {
            parts.push(format!("{} test (use --include-tests)", self.tests));
        }
        if self.out_of_scope > 0 {
            parts.push(format!(
                "{} examples/demo/fuzz/generated",
                self.out_of_scope
            ));
        }
        if self.minified_or_vendor > 0 {
            parts.push(format!("{} minified/vendor", self.minified_or_vendor));
        }
        if self.too_large > 0 {
            parts.push(format!("{} too large", self.too_large));
        }
        Some(format!(
            "{} file(s) not read: {}",
            self.total(),
            parts.join(", ")
        ))
    }
}

impl ProjectAnalysisStats {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn new() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod types_tests {
    //! Covers DebtCategory::as_str + Display, Severity::escalate/reduce
    //! saturation, TestBlockTracker state machine, and ProjectAnalysisStats
    //! defaults in satd_detector/types.rs (299 lines, 0 prior tests).
    use super::*;

    // ── DebtCategory::as_str + Display ──

    #[test]
    fn test_debt_category_as_str_all_six_arms() {
        assert_eq!(DebtCategory::Design.as_str(), "Design");
        assert_eq!(DebtCategory::Defect.as_str(), "Defect");
        assert_eq!(DebtCategory::Requirement.as_str(), "Requirement");
        assert_eq!(DebtCategory::Test.as_str(), "Test");
        assert_eq!(DebtCategory::Performance.as_str(), "Performance");
        assert_eq!(DebtCategory::Security.as_str(), "Security");
    }

    #[test]
    fn test_debt_category_display_delegates_to_as_str() {
        assert_eq!(format!("{}", DebtCategory::Design), "Design");
        assert_eq!(format!("{}", DebtCategory::Security), "Security");
    }

    // ── Severity::escalate + reduce ──

    #[test]
    fn test_severity_escalate_chain() {
        assert_eq!(Severity::Low.escalate(), Severity::Medium);
        assert_eq!(Severity::Medium.escalate(), Severity::High);
        assert_eq!(Severity::High.escalate(), Severity::Critical);
        // Critical → Critical (saturate at max).
        assert_eq!(Severity::Critical.escalate(), Severity::Critical);
    }

    #[test]
    fn test_severity_reduce_chain() {
        assert_eq!(Severity::Critical.reduce(), Severity::High);
        assert_eq!(Severity::High.reduce(), Severity::Medium);
        assert_eq!(Severity::Medium.reduce(), Severity::Low);
        // Low → Low (saturate at min).
        assert_eq!(Severity::Low.reduce(), Severity::Low);
    }

    #[test]
    fn test_severity_ord_low_lt_critical() {
        // Reordered Low → Critical for derive(Ord) — verify low < critical.
        assert!(Severity::Low < Severity::Medium);
        assert!(Severity::Medium < Severity::High);
        assert!(Severity::High < Severity::Critical);
    }

    // ── TestBlockTracker state machine ──

    #[test]
    fn test_test_block_tracker_non_rust_file_ignores_all_lines() {
        let mut t = TestBlockTracker::new(false);
        t.update_from_line("#[cfg(test)]");
        t.update_from_line("mod tests { fn x() {} }");
        // Non-Rust files never enter test blocks.
        assert!(!t.is_in_test_block());
    }

    #[test]
    fn test_test_block_tracker_enters_on_cfg_test() {
        let mut t = TestBlockTracker::new(true);
        assert!(!t.is_in_test_block());
        t.update_from_line("#[cfg(test)]");
        // After #[cfg(test)] the next line is inside the block.
        assert!(t.is_in_test_block());
    }

    #[test]
    fn test_test_block_tracker_brace_balanced_block_exit() {
        let mut t = TestBlockTracker::new(true);
        t.update_from_line("#[cfg(test)]");
        t.update_from_line("mod tests {");
        // depth=1, still inside.
        assert!(t.is_in_test_block());
        t.update_from_line("    fn a() {}");
        // open + close balance, depth back near 0 but still inside (not ended).
        // The block ends only when a line closes to depth 0 AND ends with '}'.
        t.update_from_line("}");
        assert!(!t.is_in_test_block());
    }

    #[test]
    fn test_test_block_tracker_nested_braces_keep_block_open() {
        let mut t = TestBlockTracker::new(true);
        t.update_from_line("#[cfg(test)]");
        t.update_from_line("mod tests {");
        t.update_from_line("    fn a() {");
        // 2 opens, 0 closes → depth=2.
        t.update_from_line("        let _ = || { };");
        // open+close on same line → depth still 2.
        assert!(t.is_in_test_block());
        t.update_from_line("    }");
        // depth=1, still inside.
        assert!(t.is_in_test_block());
        t.update_from_line("}");
        // depth=0 + ends with '}' → block ends.
        assert!(!t.is_in_test_block());
    }

    // ── ProjectAnalysisStats ──

    #[test]
    fn test_project_analysis_stats_new_is_default_with_empty_fields() {
        let s = ProjectAnalysisStats::new();
        assert!(s.all_debts.is_empty());
        assert_eq!(s.files_with_debt, 0);
        assert_eq!(s.total_files_analyzed, 0);
    }

    // ── DuplicateDetectionConfig::default — already covered via parent
    //    duplicate_detector_tests but defaults are independent values worth
    //    pinning as a sanity check ──

    #[test]
    fn test_severity_escalate_is_idempotent_after_3_calls_from_low() {
        // Mutation kill: ensure escalate isn't a no-op or wraps.
        let mut s = Severity::Low;
        for _ in 0..3 {
            s = s.escalate();
        }
        assert_eq!(s, Severity::Critical);
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod skip_counts_tests {
    use super::SkipCounts;

    /// Nothing skipped means nothing to disclose — the note must not clutter
    /// the summary of a run that genuinely read everything.
    #[test]
    fn silent_when_nothing_was_skipped() {
        assert_eq!(SkipCounts::default().total(), 0);
        assert!(SkipCounts::default().note().is_none());
    }

    /// #923: "Found 0 SATD violations in 0 files" was the same sentence whether
    /// the tree was clean or whether every candidate had been skipped. The note
    /// is the denominator that tells them apart, so it must name both the count
    /// and the reason — a bare number would not tell a reader that passing
    /// `--include-tests` changes the answer.
    #[test]
    fn note_names_the_count_and_the_reason() {
        let counts = SkipCounts {
            tests: 9,
            out_of_scope: 66,
            minified_or_vendor: 0,
            too_large: 2,
        };
        assert_eq!(counts.total(), 77);
        let note = counts
            .note()
            .expect("something was skipped, so there is something to say");
        assert!(note.contains("77 file(s) not read"), "{note}");
        assert!(note.contains("9 test"), "{note}");
        assert!(
            note.contains("--include-tests"),
            "the actionable flag must be named: {note}"
        );
        assert!(note.contains("66 examples/demo/fuzz/generated"), "{note}");
        assert!(note.contains("2 too large"), "{note}");
        // A reason with a zero count is noise, not disclosure.
        assert!(
            !note.contains("minified"),
            "zero-count reason listed: {note}"
        );
    }
}
