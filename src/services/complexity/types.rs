#![cfg_attr(coverage_nightly, coverage(off))]
//! Core types and data structures for complexity analysis.

use serde::{Deserialize, Serialize};

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
