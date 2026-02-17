#![cfg_attr(coverage_nightly, coverage(off))]
//! Zero-overhead complexity analysis system
//!
//! This module provides code complexity analysis without increasing binary size
//! beyond 2% by leveraging existing AST infrastructure. It calculates multiple
//! complexity metrics including cyclomatic, cognitive, and nesting complexity.
//!
//! ## Key Features
//!
//! - **`McCabe` Cyclomatic Complexity**: Measures the number of linearly independent paths
//! - **Cognitive Complexity**: Sonar method for measuring how hard code is to understand
//! - **Nesting Depth**: Maximum depth of nested control structures
//! - **Halstead Metrics**: Software science metrics for program complexity
//!
//! ## Quick Start
//!
//! ```rust
//! use pmat::services::complexity::{ComplexityMetrics, HalsteadMetrics};
//!
//! // Create basic complexity metrics
//! let metrics = ComplexityMetrics::new(5, 8, 3, 42);
//! assert_eq!(metrics.cyclomatic, 5);
//! assert_eq!(metrics.cognitive, 8);
//! assert_eq!(metrics.nesting_max, 3);
//! assert_eq!(metrics.lines, 42);
//! assert!(metrics.halstead.is_none());
//!
//! // Create with Halstead metrics
//! let halstead = HalsteadMetrics::new(10, 5, 20, 8);
//! let metrics_with_halstead = ComplexityMetrics::with_halstead(5, 8, 3, 42, halstead);
//! assert!(metrics_with_halstead.halstead.is_some());
//! ```

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Core complexity metrics for a code unit (function/method/class).
///
/// This struct encapsulates various complexity measurements that help assess
/// code maintainability and difficulty of understanding. All metrics follow
/// industry standards for software quality measurement.
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::services::complexity::{ComplexityMetrics, HalsteadMetrics};
///
/// // Simple function with low complexity
/// let simple = ComplexityMetrics::new(1, 1, 1, 5);
/// assert_eq!(simple.cyclomatic, 1);
/// assert!(simple.is_simple());
///
/// // Complex function requiring attention
/// let complex = ComplexityMetrics::new(15, 20, 5, 100);
/// assert_eq!(complex.cyclomatic, 15);
/// assert!(!complex.is_simple());
/// assert!(complex.needs_refactoring());
/// ```
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default)]
pub struct ComplexityMetrics {
    /// `McCabe` cyclomatic complexity - counts decision points + 1
    pub cyclomatic: u16,
    /// Cognitive complexity (Sonar method) - measures understandability
    pub cognitive: u16,
    /// Maximum nesting depth of control structures
    pub nesting_max: u8,
    /// Logical lines of code (excluding comments and blank lines)
    pub lines: u16,
    /// Halstead software science metrics (optional)
    pub halstead: Option<HalsteadMetrics>,
}

impl ComplexityMetrics {
    /// Creates new complexity metrics with core measurements.
    ///
    /// This is the primary constructor following Toyota Way principle of
    /// having one clear way to create objects.
    ///
    /// # Arguments
    ///
    /// * `cyclomatic` - `McCabe` cyclomatic complexity (decision points + 1)
    /// * `cognitive` - Cognitive complexity (Sonar method)
    /// * `nesting_max` - Maximum nesting depth
    /// * `lines` - Logical lines of code
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::services::complexity::ComplexityMetrics;
    ///
    /// let metrics = ComplexityMetrics::new(3, 5, 2, 25);
    /// assert_eq!(metrics.cyclomatic, 3);
    /// assert_eq!(metrics.cognitive, 5);
    /// assert_eq!(metrics.nesting_max, 2);
    /// assert_eq!(metrics.lines, 25);
    /// assert!(metrics.halstead.is_none());
    /// ```
    #[must_use]
    pub fn new(cyclomatic: u16, cognitive: u16, nesting_max: u8, lines: u16) -> Self {
        Self {
            cyclomatic,
            cognitive,
            nesting_max,
            lines,
            halstead: None, // Always initialized to None by default
        }
    }

    /// Creates complexity metrics with Halstead measurements included.
    ///
    /// Use this constructor when you have calculated Halstead software
    /// science metrics in addition to the core complexity measurements.
    ///
    /// # Arguments
    ///
    /// * `cyclomatic` - `McCabe` cyclomatic complexity
    /// * `cognitive` - Cognitive complexity
    /// * `nesting_max` - Maximum nesting depth
    /// * `lines` - Logical lines of code
    /// * `halstead` - Halstead software science metrics
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::services::complexity::{ComplexityMetrics, HalsteadMetrics};
    ///
    /// let halstead = HalsteadMetrics::new(8, 4, 16, 10);
    /// let metrics = ComplexityMetrics::with_halstead(3, 5, 2, 25, halstead);
    /// assert_eq!(metrics.cyclomatic, 3);
    /// assert!(metrics.halstead.is_some());
    /// assert_eq!(metrics.halstead.unwrap().operators_unique, 8);
    /// ```
    #[must_use]
    pub fn with_halstead(
        cyclomatic: u16,
        cognitive: u16,
        nesting_max: u8,
        lines: u16,
        halstead: HalsteadMetrics,
    ) -> Self {
        Self {
            cyclomatic,
            cognitive,
            nesting_max,
            lines,
            halstead: Some(halstead),
        }
    }

    /// Checks if the code unit has low complexity (easy to understand).
    ///
    /// Returns true if both cyclomatic and cognitive complexity are below
    /// typical thresholds for simple code.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::services::complexity::ComplexityMetrics;
    ///
    /// let simple = ComplexityMetrics::new(2, 3, 1, 10);
    /// assert!(simple.is_simple());
    ///
    /// let complex = ComplexityMetrics::new(12, 15, 4, 50);
    /// assert!(!complex.is_simple());
    /// ```
    #[must_use]
    pub fn is_simple(&self) -> bool {
        self.cyclomatic <= 5 && self.cognitive <= 7
    }

    /// Checks if the code unit needs refactoring due to high complexity.
    ///
    /// Returns true if either cyclomatic or cognitive complexity exceeds
    /// recommended thresholds for maintainable code.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::services::complexity::ComplexityMetrics;
    ///
    /// let simple = ComplexityMetrics::new(3, 4, 2, 15);
    /// assert!(!simple.needs_refactoring());
    ///
    /// let complex = ComplexityMetrics::new(15, 20, 5, 100);
    /// assert!(complex.needs_refactoring());
    /// ```
    #[must_use]
    pub fn needs_refactoring(&self) -> bool {
        self.cyclomatic > 10 || self.cognitive > 15
    }

    /// Calculates a composite complexity score.
    ///
    /// Combines multiple complexity metrics into a single score for ranking
    /// and comparison purposes. Higher scores indicate more complex code.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::services::complexity::ComplexityMetrics;
    ///
    /// let simple = ComplexityMetrics::new(1, 1, 1, 5);
    /// let complex = ComplexityMetrics::new(10, 15, 4, 80);
    ///
    /// assert!(complex.complexity_score() > simple.complexity_score());
    /// ```
    #[must_use]
    pub fn complexity_score(&self) -> f64 {
        // Weighted combination of complexity metrics
        (f64::from(self.cyclomatic) * 1.0)
            + (f64::from(self.cognitive) * 1.2)
            + (f64::from(self.nesting_max) * 2.0)
            + (f64::from(self.lines) * 0.1)
    }
}

/// Halstead software science metrics for quantitative program analysis.
///
/// These metrics were developed by Maurice Halstead in 1977 to provide
/// objective measurements of program complexity based on the number of
/// operators and operands in the source code.
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::services::complexity::HalsteadMetrics;
///
/// // Create Halstead metrics for a simple function
/// let metrics = HalsteadMetrics::new(6, 4, 12, 8);
/// assert_eq!(metrics.operators_unique, 6);
/// assert_eq!(metrics.operands_unique, 4);
/// assert_eq!(metrics.operators_total, 12);
/// assert_eq!(metrics.operands_total, 8);
///
/// // Calculate derived metrics
/// let calculated = metrics.calculate_derived();
/// assert!(calculated.volume > 0.0);
/// assert!(calculated.effort > 0.0);
/// ```
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default)]
pub struct HalsteadMetrics {
    /// n1: Number of distinct operators (unique operators like +, -, if, while)
    pub operators_unique: u32,
    /// n2: Number of distinct operands (unique variables, constants, identifiers)
    pub operands_unique: u32,
    /// N1: Total number of operators used
    pub operators_total: u32,
    /// N2: Total number of operands used
    pub operands_total: u32,
    /// V: Program volume (N * log2(n))
    pub volume: f64,
    /// D: Program difficulty (n1/2 * N2/n2)
    pub difficulty: f64,
    /// E: Programming effort (V * D)
    pub effort: f64,
    /// T: Time to program in hours (E / 18 seconds per mental discrimination)
    pub time: f64,
    /// B: Delivered bugs estimate (E^(2/3) / 3000)
    pub bugs: f64,
}

impl HalsteadMetrics {
    /// Creates new Halstead metrics with basic counts.
    ///
    /// This constructor initializes the base measurements and sets
    /// derived metrics to zero. Use `calculate_derived()` to compute
    /// volume, difficulty, effort, time, and bugs.
    ///
    /// # Arguments
    ///
    /// * `operators_unique` - Number of distinct operators
    /// * `operands_unique` - Number of distinct operands
    /// * `operators_total` - Total count of operators used
    /// * `operands_total` - Total count of operands used
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::services::complexity::HalsteadMetrics;
    ///
    /// let metrics = HalsteadMetrics::new(8, 6, 20, 15);
    /// assert_eq!(metrics.operators_unique, 8);
    /// assert_eq!(metrics.operands_unique, 6);
    /// assert_eq!(metrics.volume, 0.0); // Not calculated yet
    /// ```
    #[must_use]
    pub fn new(
        operators_unique: u32,
        operands_unique: u32,
        operators_total: u32,
        operands_total: u32,
    ) -> Self {
        Self {
            operators_unique,
            operands_unique,
            operators_total,
            operands_total,
            volume: 0.0,
            difficulty: 0.0,
            effort: 0.0,
            time: 0.0,
            bugs: 0.0,
        }
    }

    /// Calculates all derived Halstead metrics from the base counts.
    ///
    /// Computes volume, difficulty, effort, programming time, and bug estimates
    /// using the standard Halstead formulas.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::services::complexity::HalsteadMetrics;
    ///
    /// let base = HalsteadMetrics::new(10, 8, 25, 20);
    /// let calculated = base.calculate_derived();
    ///
    /// assert!(calculated.volume > 0.0);
    /// assert!(calculated.difficulty > 0.0);
    /// assert!(calculated.effort > 0.0);
    /// assert!(calculated.time > 0.0);
    /// assert!(calculated.bugs >= 0.0);
    /// ```
    #[must_use]
    pub fn calculate_derived(mut self) -> Self {
        // Prevent division by zero
        if self.operators_unique == 0 || self.operands_unique == 0 {
            return self;
        }

        let total = self.operators_total + self.operands_total;
        let unique = self.operators_unique + self.operands_unique;

        // V = N * log2(n) - Program Volume
        if unique > 0 {
            self.volume = f64::from(total) * f64::from(unique).log2();
        }

        // D = (n1/2) * (N2/n2) - Program Difficulty
        if self.operands_unique > 0 {
            self.difficulty = (f64::from(self.operators_unique) / 2.0)
                * (f64::from(self.operands_total) / f64::from(self.operands_unique));
        }

        // E = V * D - Programming Effort
        self.effort = self.volume * self.difficulty;

        // T = E / 18 - Time to program in hours (18 mental discriminations per second)
        self.time = self.effort / 18.0;

        // B = E^(2/3) / 3000 - Delivered bugs estimate
        self.bugs = self.effort.powf(2.0 / 3.0) / 3000.0;

        self
    }
}

/// Complexity metrics for an entire file
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileComplexityMetrics {
    pub path: String,
    pub total_complexity: ComplexityMetrics,
    pub functions: Vec<FunctionComplexity>,
    pub classes: Vec<ClassComplexity>,
}

/// Complexity metrics for a single function
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FunctionComplexity {
    pub name: String,
    pub line_start: u32,
    pub line_end: u32,
    pub metrics: ComplexityMetrics,
}

/// Complexity metrics for a class
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClassComplexity {
    pub name: String,
    pub line_start: u32,
    pub line_end: u32,
    pub metrics: ComplexityMetrics,
    pub methods: Vec<FunctionComplexity>,
}

/// Configuration thresholds for complexity rules
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComplexityThresholds {
    pub cyclomatic_warn: u16,
    pub cyclomatic_error: u16,
    pub cognitive_warn: u16,
    pub cognitive_error: u16,
    pub nesting_max: u8,
    pub method_length: u16,
}

impl Default for ComplexityThresholds {
    fn default() -> Self {
        Self {
            cyclomatic_warn: 10,
            cyclomatic_error: 20,
            cognitive_warn: 15,
            cognitive_error: 30,
            nesting_max: 5,
            method_length: 50,
        }
    }
}

/// A violation of complexity thresholds
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "severity", rename_all = "lowercase")]
pub enum Violation {
    Error {
        rule: String,
        message: String,
        value: u16,
        threshold: u16,
        file: String,
        line: u32,
        function: Option<String>,
    },
    Warning {
        rule: String,
        message: String,
        value: u16,
        threshold: u16,
        file: String,
        line: u32,
        function: Option<String>,
    },
}

/// Summary statistics for complexity analysis
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ComplexitySummary {
    pub total_files: usize,
    pub total_functions: usize,
    pub median_cyclomatic: f32,
    pub median_cognitive: f32,
    pub max_cyclomatic: u16,
    pub max_cognitive: u16,
    pub p90_cyclomatic: u16,
    pub p90_cognitive: u16,
    pub technical_debt_hours: f32,
}

/// A hotspot of high complexity in the codebase
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComplexityHotspot {
    pub file: String,
    pub function: Option<String>,
    pub line: u32,
    pub complexity: u16,
    pub complexity_type: String,
}

/// Complete complexity analysis report
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComplexityReport {
    pub summary: ComplexitySummary,
    pub violations: Vec<Violation>,
    pub hotspots: Vec<ComplexityHotspot>,
    pub files: Vec<FileComplexityMetrics>,
}

/// Zero-allocation complexity visitor for AST traversal
pub struct ComplexityVisitor<'a> {
    pub complexity: &'a mut ComplexityMetrics,
    pub nesting_level: u8,
    pub current_function: Option<String>,
    pub functions: Vec<FunctionComplexity>,
    pub classes: Vec<ClassComplexity>,
}

impl<'a> ComplexityVisitor<'a> {
    pub fn new(complexity: &'a mut ComplexityMetrics) -> Self {
        Self {
            complexity,
            nesting_level: 0,
            current_function: None,
            functions: Vec::new(),
            classes: Vec::new(),
        }
    }

    /// Calculate cognitive complexity increment based on node type and nesting
    #[inline(always)]
    #[must_use]
    pub fn calculate_cognitive_increment(&self, is_nesting_construct: bool) -> u16 {
        if is_nesting_construct {
            1 + u16::from(self.nesting_level.saturating_sub(1))
        } else {
            1
        }
    }

    /// Enter a nesting level
    #[inline(always)]
    pub fn enter_nesting(&mut self) {
        self.nesting_level = self.nesting_level.saturating_add(1);
        if self.nesting_level > self.complexity.nesting_max {
            self.complexity.nesting_max = self.nesting_level;
        }
    }

    /// Exit a nesting level
    #[inline(always)]
    pub fn exit_nesting(&mut self) {
        self.nesting_level = self.nesting_level.saturating_sub(1);
    }
}

/// Cache key computation for complexity metrics
/// Computes a cache key for complexity analysis based on file path and content
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::services::complexity::compute_complexity_cache_key;
/// use std::path::Path;
///
/// let path = Path::new("src/main.rs");
/// let content = b"fn main() { println!(\"Hello\"); }";
///
/// let key = compute_complexity_cache_key(path, content);
/// assert!(key.starts_with("cx:"));
/// assert!(key.len() > 10);
/// ```
#[must_use]
pub fn compute_complexity_cache_key(path: &Path, content: &[u8]) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    path.hash(&mut hasher);
    format!("cx:{:x}", hasher.finish())
}

/// Trait for complexity rules
pub trait ComplexityRule: Send + Sync {
    fn evaluate(
        &self,
        metrics: &ComplexityMetrics,
        file: &str,
        line: u32,
        function: Option<&str>,
    ) -> Option<Violation>;

    #[inline(always)]
    fn exceeds_threshold(&self, value: u16, threshold: u16) -> bool {
        value > threshold
    }
}

/// Cyclomatic complexity rule implementation
pub struct CyclomaticComplexityRule {
    warn_threshold: u16,
    error_threshold: u16,
}

impl CyclomaticComplexityRule {
    #[must_use]
    pub fn new(thresholds: &ComplexityThresholds) -> Self {
        Self {
            warn_threshold: thresholds.cyclomatic_warn,
            error_threshold: thresholds.cyclomatic_error,
        }
    }
}

impl ComplexityRule for CyclomaticComplexityRule {
    fn evaluate(
        &self,
        metrics: &ComplexityMetrics,
        file: &str,
        line: u32,
        function: Option<&str>,
    ) -> Option<Violation> {
        if self.exceeds_threshold(metrics.cyclomatic, self.error_threshold) {
            Some(Violation::Error {
                rule: "cyclomatic-complexity".to_string(),
                message: format!(
                    "Cyclomatic complexity of {} exceeds maximum allowed complexity of {}",
                    metrics.cyclomatic, self.error_threshold
                ),
                value: metrics.cyclomatic,
                threshold: self.error_threshold,
                file: file.to_string(),
                line,
                function: function.map(String::from),
            })
        } else if self.exceeds_threshold(metrics.cyclomatic, self.warn_threshold) {
            Some(Violation::Warning {
                rule: "cyclomatic-complexity".to_string(),
                message: format!(
                    "Cyclomatic complexity of {} exceeds recommended complexity of {}",
                    metrics.cyclomatic, self.warn_threshold
                ),
                value: metrics.cyclomatic,
                threshold: self.warn_threshold,
                file: file.to_string(),
                line,
                function: function.map(String::from),
            })
        } else {
            None
        }
    }
}

/// Cognitive complexity rule implementation
pub struct CognitiveComplexityRule {
    warn_threshold: u16,
    error_threshold: u16,
}

impl CognitiveComplexityRule {
    #[must_use]
    pub fn new(thresholds: &ComplexityThresholds) -> Self {
        Self {
            warn_threshold: thresholds.cognitive_warn,
            error_threshold: thresholds.cognitive_error,
        }
    }
}

impl ComplexityRule for CognitiveComplexityRule {
    fn evaluate(
        &self,
        metrics: &ComplexityMetrics,
        file: &str,
        line: u32,
        function: Option<&str>,
    ) -> Option<Violation> {
        if self.exceeds_threshold(metrics.cognitive, self.error_threshold) {
            Some(Violation::Error {
                rule: "cognitive-complexity".to_string(),
                message: format!(
                    "Cognitive complexity of {} exceeds maximum allowed complexity of {}",
                    metrics.cognitive, self.error_threshold
                ),
                value: metrics.cognitive,
                threshold: self.error_threshold,
                file: file.to_string(),
                line,
                function: function.map(String::from),
            })
        } else if self.exceeds_threshold(metrics.cognitive, self.warn_threshold) {
            Some(Violation::Warning {
                rule: "cognitive-complexity".to_string(),
                message: format!(
                    "Cognitive complexity of {} exceeds recommended complexity of {}",
                    metrics.cognitive, self.warn_threshold
                ),
                value: metrics.cognitive,
                threshold: self.warn_threshold,
                file: file.to_string(),
                line,
                function: function.map(String::from),
            })
        } else {
            None
        }
    }
}

/// Aggregate complexity results from multiple files
/// Aggregates file-level complexity metrics into a comprehensive report
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::services::complexity::{aggregate_results, FileComplexityMetrics};
///
/// let metrics = vec![];
/// let report = aggregate_results(metrics);
/// assert_eq!(report.summary.total_files, 0);
/// ```
/// Aggregates complexity metrics from multiple files into a summary report
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::services::complexity::{aggregate_results, FileComplexityMetrics, ComplexityMetrics};
///
/// let file = FileComplexityMetrics {
///     path: "src/main.rs".to_string(),
///     total_complexity: ComplexityMetrics {
///         cyclomatic: 10,
///         cognitive: 8,
///         nesting_max: 3,
///         lines: 50,
///         halstead: None,
///     },
///     functions: vec![],
///     classes: vec![],
/// };
///
/// let report = aggregate_results(vec![file]);
/// assert_eq!(report.files.len(), 1);
/// ```
#[must_use]
pub fn aggregate_results(file_metrics: Vec<FileComplexityMetrics>) -> ComplexityReport {
    aggregate_results_with_thresholds(file_metrics, None, None)
}

/// Aggregate complexity results with custom thresholds
///
/// This function allows customizing the complexity thresholds used to determine violations,
/// addressing issue #32 where `--max-cyclomatic` didn't affect report output.
///
/// # Arguments
///
/// * `file_metrics` - Vector of file complexity metrics to aggregate
/// * `max_cyclomatic` - Optional custom maximum cyclomatic complexity threshold
/// * `max_cognitive` - Optional custom maximum cognitive complexity threshold
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::services::complexity::*;
///
/// let metrics = ComplexityMetrics {
///     cyclomatic: 25,
///     cognitive: 30,
///     nesting_max: 3,
///     lines: 100,
///     halstead: None,
/// };
///
/// let func = FunctionComplexity {
///     name: "complex_function".to_string(),
///     line_start: 10,
///     line_end: 50,
///     metrics,
/// };
///
/// let file = FileComplexityMetrics {
///     path: "src/main.rs".to_string(),
///     total_complexity: metrics,
///     functions: vec![func],
///     classes: vec![],
/// };
///
/// // With custom threshold of 20, the function with complexity 25 will be a violation
/// let report = aggregate_results_with_thresholds(vec![file.clone()], Some(20), None);
/// assert_eq!(report.violations.len(), 2); // Both cyclomatic (25) and cognitive (30) exceed 20
/// assert!(matches!(report.violations[0], Violation::Error { .. }));
///
/// // With cyclomatic threshold of 35 and cognitive threshold of 35, no violations
/// let report2 = aggregate_results_with_thresholds(vec![file], Some(35), Some(35));
/// assert_eq!(report2.violations.len(), 0);
/// ```
#[must_use]
pub fn aggregate_results_with_thresholds(
    file_metrics: Vec<FileComplexityMetrics>,
    max_cyclomatic: Option<u16>,
    max_cognitive: Option<u16>,
) -> ComplexityReport {
    let thresholds = build_custom_thresholds(max_cyclomatic, max_cognitive);
    let rules = create_complexity_rules(&thresholds);
    let mut analysis_data = analyze_file_metrics(&file_metrics, &rules, &thresholds);
    let summary_stats = calculate_summary_statistics(&mut analysis_data);
    let technical_debt = calculate_technical_debt(&analysis_data.violations);

    build_complexity_report(file_metrics, analysis_data, summary_stats, technical_debt)
}

/// Build custom thresholds from optional parameters
fn build_custom_thresholds(
    max_cyclomatic: Option<u16>,
    max_cognitive: Option<u16>,
) -> ComplexityThresholds {
    let mut thresholds = ComplexityThresholds::default();

    if let Some(max_cyc) = max_cyclomatic {
        // Narrow warning band: only warn within 2 of error threshold
        thresholds.cyclomatic_warn = max_cyc.saturating_sub(2).max(1);
        thresholds.cyclomatic_error = max_cyc;
    }
    if let Some(max_cog) = max_cognitive {
        thresholds.cognitive_warn = max_cog.saturating_sub(2).max(1);
        thresholds.cognitive_error = max_cog;
    }

    thresholds
}

/// Create complexity rules from thresholds
fn create_complexity_rules(
    thresholds: &ComplexityThresholds,
) -> (CyclomaticComplexityRule, CognitiveComplexityRule) {
    let cyclomatic_rule = CyclomaticComplexityRule::new(thresholds);
    let cognitive_rule = CognitiveComplexityRule::new(thresholds);
    (cyclomatic_rule, cognitive_rule)
}

/// Intermediate data structure for analysis results
struct AnalysisData {
    all_cyclomatic: Vec<u16>,
    all_cognitive: Vec<u16>,
    violations: Vec<Violation>,
    hotspots: Vec<ComplexityHotspot>,
    total_functions: usize,
}

/// Analyze file metrics and collect data
fn analyze_file_metrics(
    file_metrics: &[FileComplexityMetrics],
    rules: &(CyclomaticComplexityRule, CognitiveComplexityRule),
    thresholds: &ComplexityThresholds,
) -> AnalysisData {
    let mut data = AnalysisData {
        all_cyclomatic: Vec::new(),
        all_cognitive: Vec::new(),
        violations: Vec::new(),
        hotspots: Vec::new(),
        total_functions: 0,
    };

    for file in file_metrics {
        process_file_functions(file, rules, thresholds, &mut data);
        process_file_classes(file, rules, &mut data);
    }

    data
}

/// Process functions in a file
fn process_file_functions(
    file: &FileComplexityMetrics,
    rules: &(CyclomaticComplexityRule, CognitiveComplexityRule),
    thresholds: &ComplexityThresholds,
    data: &mut AnalysisData,
) {
    let (cyclomatic_rule, cognitive_rule) = rules;

    for func in &file.functions {
        data.total_functions += 1;
        data.all_cyclomatic.push(func.metrics.cyclomatic);
        data.all_cognitive.push(func.metrics.cognitive);

        check_function_violations(
            func,
            file,
            cyclomatic_rule,
            cognitive_rule,
            &mut data.violations,
        );
        check_function_hotspots(func, file, thresholds, &mut data.hotspots);
    }
}

/// Process classes and their methods in a file
fn process_file_classes(
    file: &FileComplexityMetrics,
    rules: &(CyclomaticComplexityRule, CognitiveComplexityRule),
    data: &mut AnalysisData,
) {
    let (cyclomatic_rule, cognitive_rule) = rules;

    for class in &file.classes {
        for method in &class.methods {
            data.total_functions += 1;
            data.all_cyclomatic.push(method.metrics.cyclomatic);
            data.all_cognitive.push(method.metrics.cognitive);

            check_method_violations(
                method,
                file,
                cyclomatic_rule,
                cognitive_rule,
                &mut data.violations,
            );
        }
    }
}

/// Check function for complexity violations
fn check_function_violations(
    func: &FunctionComplexity,
    file: &FileComplexityMetrics,
    cyclomatic_rule: &CyclomaticComplexityRule,
    cognitive_rule: &CognitiveComplexityRule,
    violations: &mut Vec<Violation>,
) {
    if let Some(violation) =
        cyclomatic_rule.evaluate(&func.metrics, &file.path, func.line_start, Some(&func.name))
    {
        violations.push(violation);
    }

    if let Some(violation) =
        cognitive_rule.evaluate(&func.metrics, &file.path, func.line_start, Some(&func.name))
    {
        violations.push(violation);
    }
}

/// Check method for complexity violations
fn check_method_violations(
    method: &FunctionComplexity,
    file: &FileComplexityMetrics,
    cyclomatic_rule: &CyclomaticComplexityRule,
    cognitive_rule: &CognitiveComplexityRule,
    violations: &mut Vec<Violation>,
) {
    if let Some(violation) = cyclomatic_rule.evaluate(
        &method.metrics,
        &file.path,
        method.line_start,
        Some(&method.name),
    ) {
        violations.push(violation);
    }

    if let Some(violation) = cognitive_rule.evaluate(
        &method.metrics,
        &file.path,
        method.line_start,
        Some(&method.name),
    ) {
        violations.push(violation);
    }
}

/// Check function for complexity hotspots
fn check_function_hotspots(
    func: &FunctionComplexity,
    file: &FileComplexityMetrics,
    thresholds: &ComplexityThresholds,
    hotspots: &mut Vec<ComplexityHotspot>,
) {
    if func.metrics.cyclomatic > thresholds.cyclomatic_warn {
        hotspots.push(ComplexityHotspot {
            file: file.path.clone(),
            function: Some(func.name.clone()),
            line: func.line_start,
            complexity: func.metrics.cyclomatic,
            complexity_type: "cyclomatic".to_string(),
        });
    }
}

/// Summary statistics structure
struct SummaryStats {
    median_cyclomatic: f32,
    median_cognitive: f32,
    max_cyclomatic: u16,
    max_cognitive: u16,
    p90_cyclomatic: u16,
    p90_cognitive: u16,
}

/// Calculate summary statistics from analysis data
fn calculate_summary_statistics(data: &mut AnalysisData) -> SummaryStats {
    data.all_cyclomatic.sort_unstable();
    data.all_cognitive.sort_unstable();

    let p90_stats = calculate_percentiles(&data.all_cyclomatic, &data.all_cognitive);
    let median_stats = calculate_medians(&data.all_cyclomatic, &data.all_cognitive);
    let max_stats = calculate_max_values(&data.all_cyclomatic, &data.all_cognitive);

    // Sort and limit hotspots
    data.hotspots
        .sort_unstable_by(|a, b| b.complexity.cmp(&a.complexity));
    data.hotspots.truncate(10);

    SummaryStats {
        median_cyclomatic: median_stats.0,
        median_cognitive: median_stats.1,
        max_cyclomatic: max_stats.0,
        max_cognitive: max_stats.1,
        p90_cyclomatic: p90_stats.0,
        p90_cognitive: p90_stats.1,
    }
}

/// Calculate 90th percentile values
fn calculate_percentiles(all_cyclomatic: &[u16], all_cognitive: &[u16]) -> (u16, u16) {
    let p90_index = (all_cyclomatic.len() as f32 * 0.9) as usize;
    let p90_cyclomatic = all_cyclomatic.get(p90_index).copied().unwrap_or(0);
    let p90_cognitive = all_cognitive.get(p90_index).copied().unwrap_or(0);
    (p90_cyclomatic, p90_cognitive)
}

/// Calculate median values
fn calculate_medians(all_cyclomatic: &[u16], all_cognitive: &[u16]) -> (f32, f32) {
    let median_cyclomatic = calculate_median(all_cyclomatic);
    let median_cognitive = calculate_median(all_cognitive);
    (median_cyclomatic, median_cognitive)
}

/// Calculate median for a sorted array
fn calculate_median(values: &[u16]) -> f32 {
    if values.is_empty() {
        return 0.0;
    }

    let mid = values.len() / 2;
    if values.len() % 2 == 0 {
        f32::from(values[mid - 1] + values[mid]) / 2.0
    } else {
        f32::from(values[mid])
    }
}

/// Calculate maximum values
fn calculate_max_values(all_cyclomatic: &[u16], all_cognitive: &[u16]) -> (u16, u16) {
    let max_cyclomatic = all_cyclomatic.iter().max().copied().unwrap_or(0);
    let max_cognitive = all_cognitive.iter().max().copied().unwrap_or(0);
    (max_cyclomatic, max_cognitive)
}

/// Calculate technical debt hours from violations
fn calculate_technical_debt(violations: &[Violation]) -> f32 {
    let debt_minutes: f32 = violations
        .iter()
        .map(|v| match v {
            Violation::Error {
                value, threshold, ..
            } => f32::from(value - threshold) * 30.0,
            Violation::Warning {
                value, threshold, ..
            } => f32::from(value - threshold) * 15.0,
        })
        .sum();
    debt_minutes / 60.0
}

/// Build the final complexity report
fn build_complexity_report(
    file_metrics: Vec<FileComplexityMetrics>,
    analysis_data: AnalysisData,
    summary_stats: SummaryStats,
    technical_debt_hours: f32,
) -> ComplexityReport {
    ComplexityReport {
        summary: ComplexitySummary {
            total_files: file_metrics.len(),
            total_functions: analysis_data.total_functions,
            median_cyclomatic: summary_stats.median_cyclomatic,
            median_cognitive: summary_stats.median_cognitive,
            max_cyclomatic: summary_stats.max_cyclomatic,
            max_cognitive: summary_stats.max_cognitive,
            p90_cyclomatic: summary_stats.p90_cyclomatic,
            p90_cognitive: summary_stats.p90_cognitive,
            technical_debt_hours,
        },
        violations: analysis_data.violations,
        hotspots: analysis_data.hotspots,
        files: file_metrics,
    }
}

/// Format complexity summary for CLI output
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::services::complexity::*;
///
/// let file_metrics = vec![
///     FileComplexityMetrics {
///         path: "src/main.rs".to_string(),
///         total_complexity: ComplexityMetrics {
///             cyclomatic: 5,
///             cognitive: 7,
///             nesting_max: 2,
///             lines: 30,
///             halstead: None,
///         },
///         functions: vec![],
///         classes: vec![],
///     },
///     FileComplexityMetrics {
///         path: "src/lib.rs".to_string(),
///         total_complexity: ComplexityMetrics {
///             cyclomatic: 3,
///             cognitive: 4,
///             nesting_max: 1,
///             lines: 20,
///             halstead: None,
///         },
///         functions: vec![],
///         classes: vec![],
///     },
/// ];
///
/// let report = aggregate_results(file_metrics);
/// let summary = format_complexity_summary(&report);
///
/// assert!(summary.contains("# Complexity Analysis Summary"));
/// assert!(summary.contains("**Files analyzed**: 2"));
/// assert!(summary.contains("## Top Files by Complexity"));
/// assert!(summary.contains("main.rs")); // First file (higher complexity)
/// assert!(summary.contains("lib.rs"));  // Second file
/// ```
#[must_use]
pub fn format_complexity_summary(report: &ComplexityReport) -> String {
    let mut output = String::new();

    output.push_str("# Complexity Analysis Summary\n\n");

    output.push_str(&format!(
        "📊 **Files analyzed**: {}\n",
        report.summary.total_files
    ));
    output.push_str(&format!(
        "🔧 **Total functions**: {}\n\n",
        report.summary.total_functions
    ));

    output.push_str("## Complexity Metrics\n\n");
    output.push_str(&format!(
        "- **Median Cyclomatic**: {:.1}\n",
        report.summary.median_cyclomatic
    ));
    output.push_str(&format!(
        "- **Median Cognitive**: {:.1}\n",
        report.summary.median_cognitive
    ));
    output.push_str(&format!(
        "- **Max Cyclomatic**: {}\n",
        report.summary.max_cyclomatic
    ));
    output.push_str(&format!(
        "- **Max Cognitive**: {}\n",
        report.summary.max_cognitive
    ));
    output.push_str(&format!(
        "- **90th Percentile Cyclomatic**: {}\n",
        report.summary.p90_cyclomatic
    ));
    output.push_str(&format!(
        "- **90th Percentile Cognitive**: {}\n\n",
        report.summary.p90_cognitive
    ));

    if report.summary.technical_debt_hours > 0.0 {
        output.push_str(&format!(
            "⏱️  **Estimated Refactoring Time**: {:.1} hours\n\n",
            report.summary.technical_debt_hours
        ));
    }

    // Violations summary
    let error_count = report
        .violations
        .iter()
        .filter(|v| matches!(v, Violation::Error { .. }))
        .count();
    let warning_count = report
        .violations
        .iter()
        .filter(|v| matches!(v, Violation::Warning { .. }))
        .count();

    if error_count > 0 || warning_count > 0 {
        output.push_str("## Issues Found\n\n");
        if error_count > 0 {
            output.push_str(&format!("❌ **Errors**: {error_count}\n"));
        }
        if warning_count > 0 {
            output.push_str(&format!("⚠️  **Warnings**: {warning_count}\n"));
        }
        output.push('\n');
    }

    // Top files by complexity
    if !report.files.is_empty() {
        output.push_str("## Top Files by Complexity\n\n");

        // Sort files by total complexity (cyclomatic + cognitive)
        let mut files_with_score: Vec<_> = report
            .files
            .iter()
            .map(|f| {
                let total_score = f64::from(f.total_complexity.cyclomatic)
                    + f64::from(f.total_complexity.cognitive);
                (f, total_score)
            })
            .collect();
        files_with_score
            .sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        for (i, (file, _score)) in files_with_score.iter().take(10).enumerate() {
            // Use relative path for better identification, not just filename
            let display_path = file.path.strip_prefix("./").unwrap_or(&file.path);
            output.push_str(&format!(
                "{}. `{}` - Cyclomatic: {}, Cognitive: {}, Functions: {}\n",
                i + 1,
                display_path,
                file.total_complexity.cyclomatic,
                file.total_complexity.cognitive,
                file.functions.len()
            ));
        }
        output.push('\n');

        // Show all functions when there's only one file (e.g., single file analysis)
        if report.files.len() == 1 && !report.files[0].functions.is_empty() {
            output.push_str("## Functions in File\n\n");

            // Sort functions by total complexity
            let mut functions_with_score: Vec<_> = report.files[0]
                .functions
                .iter()
                .map(|f| {
                    let total = f64::from(f.metrics.cyclomatic) + f64::from(f.metrics.cognitive);
                    (f, total)
                })
                .collect();
            functions_with_score.sort_unstable_by(|a, b| {
                b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
            });

            for (i, (func, _)) in functions_with_score.iter().enumerate() {
                output.push_str(&format!(
                    "{}. `{}` (line {}-{}) - Cyclomatic: {}, Cognitive: {}\n",
                    i + 1,
                    func.name,
                    func.line_start,
                    func.line_end,
                    func.metrics.cyclomatic,
                    func.metrics.cognitive
                ));
            }
            output.push('\n');
        }
    }

    // Top hotspots
    if !report.hotspots.is_empty() {
        output.push_str("## Top Complexity Hotspots\n\n");
        for (i, hotspot) in report.hotspots.iter().take(5).enumerate() {
            let display_path = hotspot.file.strip_prefix("./").unwrap_or(&hotspot.file);
            let func_name = hotspot.function.as_deref().unwrap_or("<file>");
            output.push_str(&format!(
                "{}. `{}` {}:{} - {} complexity: {}\n",
                i + 1,
                func_name,
                display_path,
                hotspot.line,
                hotspot.complexity_type,
                hotspot.complexity
            ));
        }
    }

    output
}

/// Format full complexity report for CLI output
#[must_use]
pub fn format_complexity_report(report: &ComplexityReport) -> String {
    let mut output = format_complexity_summary(report);

    output.push_str("\n## Detailed Violations\n\n");

    // Group violations by file
    let mut violations_by_file: rustc_hash::FxHashMap<&str, Vec<&Violation>> =
        rustc_hash::FxHashMap::default();
    for violation in &report.violations {
        let file = match violation {
            Violation::Error { file, .. } | Violation::Warning { file, .. } => file.as_str(),
        };
        violations_by_file.entry(file).or_default().push(violation);
    }

    for (file, violations) in violations_by_file {
        output.push_str(&format!("### {file}\n\n"));

        for violation in violations {
            match violation {
                Violation::Error {
                    rule,
                    message,
                    line,
                    function,
                    ..
                } => {
                    output.push_str(&format!(
                        "❌ **{}:{}** {} - {}\n",
                        line,
                        function.as_deref().unwrap_or(""),
                        rule,
                        message
                    ));
                }
                Violation::Warning {
                    rule,
                    message,
                    line,
                    function,
                    ..
                } => {
                    output.push_str(&format!(
                        "⚠️  **{}:{}** {} - {}\n",
                        line,
                        function.as_deref().unwrap_or(""),
                        rule,
                        message
                    ));
                }
            }
        }
        output.push('\n');
    }

    output
}

/// Format complexity report as SARIF for IDE integration
/// Formats a complexity report as SARIF (Static Analysis Results Interchange Format)
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::services::complexity::{format_as_sarif, ComplexityReport, ComplexitySummary};
///
/// let report = ComplexityReport {
///     summary: ComplexitySummary {
///         total_files: 1,
///         total_functions: 1,
///         median_cyclomatic: 5.0,
///         median_cognitive: 5.0,
///         max_cyclomatic: 10,
///         max_cognitive: 10,
///         p90_cyclomatic: 8,
///         p90_cognitive: 8,
///         technical_debt_hours: 1.0,
///     },
///     violations: vec![],
///     hotspots: vec![],
///     files: vec![],
/// };
///
/// let sarif = format_as_sarif(&report).unwrap();
/// assert!(sarif.contains("\"version\": \"2.1.0\""));
/// assert!(sarif.contains("cyclomatic-complexity"));
/// ```
pub fn format_as_sarif(report: &ComplexityReport) -> Result<String, serde_json::Error> {
    use serde_json::json;

    let rules = vec![
        json!({
            "id": "cyclomatic-complexity",
            "name": "Cyclomatic Complexity",
            "shortDescription": {
                "text": "Function has high cyclomatic complexity"
            },
            "fullDescription": {
                "text": "Cyclomatic complexity measures the number of linearly independent paths through a function"
            },
            "defaultConfiguration": {
                "level": "warning"
            }
        }),
        json!({
            "id": "cognitive-complexity",
            "name": "Cognitive Complexity",
            "shortDescription": {
                "text": "Function has high cognitive complexity"
            },
            "fullDescription": {
                "text": "Cognitive complexity measures how difficult the function is to understand"
            },
            "defaultConfiguration": {
                "level": "warning"
            }
        }),
    ];

    let mut results = Vec::new();
    for violation in &report.violations {
        let (rule_id, message, level, file, line, _function) = match violation {
            Violation::Error {
                rule,
                message,
                file,
                line,
                function,
                ..
            } => (rule, message, "error", file, line, function),
            Violation::Warning {
                rule,
                message,
                file,
                line,
                function,
                ..
            } => (rule, message, "warning", file, line, function),
        };

        results.push(json!({
            "ruleId": rule_id,
            "level": level,
            "message": {
                "text": message
            },
            "locations": [{
                "physicalLocation": {
                    "artifactLocation": {
                        "uri": file
                    },
                    "region": {
                        "startLine": line
                    }
                }
            }]
        }));
    }

    let sarif = json!({
        "version": "2.1.0",
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "pmat",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit",
                    "rules": rules
                }
            },
            "results": results
        }]
    });

    serde_json::to_string_pretty(&sarif)
}

/// Analyze file complexity WITHOUT using TDG cache (Issue #67 fix)
///
/// This function performs fresh analysis and always reports accurate
/// line numbers from the current file location. Use this for:
/// - `--file` parameter (single file analysis)
/// - `--force-refresh` flag
/// - Pre-commit hooks requiring accurate line numbers
///
/// # Root Cause (Issue #67)
///
/// The TDG cache uses content hash as the primary key. When functions are
/// extracted from one file to another, the content hash remains the same,
/// causing line numbers from the OLD location to be reported for the NEW file.
///
/// # Solution
///
/// This function bypasses the TDG cache entirely and performs fresh AST/heuristic
/// analysis, ensuring line numbers reflect the actual current file location.
///
/// # Arguments
///
/// * `path` - File path to analyze
/// * `content` - Optional file content (reads from disk if None)
///
/// # Returns
///
/// Fresh `FileComplexityMetrics` with accurate line numbers
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::services::complexity::analyze_file_complexity_uncached;
/// use std::path::Path;
///
/// # async fn example() -> anyhow::Result<()> {
/// // Analyze file with fresh line numbers (bypasses cache)
/// let path = Path::new("src/extracted_functions.rs");
/// let metrics = analyze_file_complexity_uncached(path, None).await?;
///
/// // Line numbers reflect CURRENT file location
/// for func in &metrics.functions {
///     println!("{} at lines {}-{}", func.name, func.line_start, func.line_end);
/// }
/// # Ok(())
/// # }
/// ```
///
/// # See Also
///
/// - Issue #67: https://github.com/paiml/paiml-mcp-agent-toolkit/issues/67
/// - Test suite: `complexity_file_extraction_tests.rs`
pub async fn analyze_file_complexity_uncached(
    path: &Path,
    content: Option<&str>,
) -> anyhow::Result<FileComplexityMetrics> {
    use anyhow::Context;

    // Read file content if not provided
    let file_content;
    let content_ref = if let Some(c) = content {
        c
    } else {
        file_content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;
        &file_content
    };

    // CRITICAL (Issue #67): Use heuristic analyzer for accurate line numbers
    // The AST analyzer returns approximate line numbers (i * 50), which breaks
    // extracted function scenarios. The heuristic analyzer provides EXACT line
    // numbers by parsing the actual file content.
    //
    // This ensures:
    // - Functions extracted from old_file.rs:500 to new_file.rs:148
    // - Report line 148 (CURRENT location), not line 500 (OLD cached location)
    let language = crate::cli::language_analyzer::Language::from_path(path);
    crate::cli::language_analyzer::analyze_with_heuristics(path, content_ref, language)
        .with_context(|| format!("Failed to analyze file complexity: {}", path.display()))
}

#[cfg(test)]
mod tests;

// BROKEN: complexity_tests_part1.rs truncated at line 500
#[cfg(all(test, feature = "broken-tests"))]
#[path = "complexity_tests.rs"]
mod broken_tests;
