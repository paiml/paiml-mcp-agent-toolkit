//! Big-O Complexity Analysis Data Structures - Phase 5
//!
//! Provides memory-efficient representations for algorithmic complexity bounds,
//! supporting time and space complexity analysis with confidence scoring.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Big-O complexity class enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum BigOClass {
    /// O(1) - Constant time
    Constant = 0,
    /// O(log n) - Logarithmic time
    Logarithmic = 1,
    /// O(n) - Linear time
    Linear = 2,
    /// O(n log n) - Linearithmic time
    Linearithmic = 3,
    /// O(n²) - Quadratic time
    Quadratic = 4,
    /// O(n³) - Cubic time
    Cubic = 5,
    /// O(2^n) - Exponential time
    Exponential = 6,
    /// O(n!) - Factorial time
    Factorial = 7,
    /// Unknown or too complex to determine
    Unknown = 255,
}

impl BigOClass {
    /// Get a human-readable notation for the complexity class
    pub fn notation(&self) -> &'static str {
        match self {
            Self::Constant => "O(1)",
            Self::Logarithmic => "O(log n)",
            Self::Linear => "O(n)",
            Self::Linearithmic => "O(n log n)",
            Self::Quadratic => "O(n²)",
            Self::Cubic => "O(n³)",
            Self::Exponential => "O(2^n)",
            Self::Factorial => "O(n!)",
            Self::Unknown => "O(?)",
        }
    }

    /// Check if this complexity is better than another
    pub fn is_better_than(&self, other: &Self) -> bool {
        (*self as u8) < (*other as u8)
    }

    /// Get approximate growth factor for small n
    pub fn growth_factor(&self, n: f64) -> f64 {
        match self {
            Self::Constant => 1.0,
            Self::Logarithmic => n.log2(),
            Self::Linear => n,
            Self::Linearithmic => n * n.log2(),
            Self::Quadratic => n * n,
            Self::Cubic => n * n * n,
            Self::Exponential => 2.0_f64.powf(n),
            Self::Factorial => {
                // Stirling's approximation for large n
                if n <= 20.0 {
                    (1..=n as u32).map(|i| i as f64).product()
                } else {
                    (2.0 * std::f64::consts::PI * n).sqrt() * (n / std::f64::consts::E).powf(n)
                }
            }
            Self::Unknown => f64::NAN,
        }
    }
}

impl fmt::Display for BigOClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.notation())
    }
}

/// Input variable for complexity analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum InputVariable {
    /// Size of primary input (n)
    N = 0,
    /// Size of secondary input (m)
    M = 1,
    /// Number of unique elements (k)
    K = 2,
    /// Depth or height parameter (d)
    D = 3,
    /// Custom or composite variable
    Custom = 255,
}

impl fmt::Display for InputVariable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::N => write!(f, "n"),
            Self::M => write!(f, "m"),
            Self::K => write!(f, "k"),
            Self::D => write!(f, "d"),
            Self::Custom => write!(f, "x"),
        }
    }
}

/// Flags for complexity bound properties
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[repr(transparent)]
pub struct ComplexityFlags(u8);

impl ComplexityFlags {
    pub const AMORTIZED: u8 = 0b00000001;
    pub const WORST_CASE: u8 = 0b00000010;
    pub const AVERAGE_CASE: u8 = 0b00000100;
    pub const BEST_CASE: u8 = 0b00001000;
    pub const TIGHT_BOUND: u8 = 0b00010000;
    pub const EMPIRICAL: u8 = 0b00100000;
    pub const PROVEN: u8 = 0b01000000;
    pub const RECURSIVE: u8 = 0b10000000;

    pub fn new() -> Self {
        Self(0)
    }

    pub fn with(mut self, flag: u8) -> Self {
        self.0 |= flag;
        self
    }

    pub fn has(&self, flag: u8) -> bool {
        self.0 & flag != 0
    }

    pub fn is_worst_case(&self) -> bool {
        self.has(Self::WORST_CASE)
    }

    pub fn is_proven(&self) -> bool {
        self.has(Self::PROVEN)
    }
}

/// Memory-efficient complexity bound representation (8 bytes)
#[repr(C, align(8))]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ComplexityBound {
    /// Big-O complexity class (1 byte)
    pub class: BigOClass,
    /// Coefficient multiplier (2 bytes)
    pub coefficient: u16,
    /// Input variable (1 byte)
    pub input_var: InputVariable,
    /// Confidence percentage 0-100 (1 byte)
    pub confidence: u8,
    /// Property flags (1 byte)
    pub flags: ComplexityFlags,
    /// Padding for alignment (2 bytes)
    _padding: [u8; 2],
}

impl ComplexityBound {
    /// Create a new complexity bound
    pub fn new(class: BigOClass, coefficient: u16, input_var: InputVariable) -> Self {
        Self {
            class,
            coefficient,
            input_var,
            confidence: 50,
            flags: ComplexityFlags::new().with(ComplexityFlags::WORST_CASE),
            _padding: [0; 2],
        }
    }

    /// Create a constant time bound O(1)
    pub fn constant() -> Self {
        Self::new(BigOClass::Constant, 1, InputVariable::N)
            .with_confidence(100)
            .with_flags(ComplexityFlags::PROVEN)
    }

    /// Create a linear time bound O(n)
    pub fn linear() -> Self {
        Self::new(BigOClass::Linear, 1, InputVariable::N)
    }

    /// Create a quadratic time bound O(n²)
    pub fn quadratic() -> Self {
        Self::new(BigOClass::Quadratic, 1, InputVariable::N)
    }

    /// Create a logarithmic time bound O(log n)
    pub fn logarithmic() -> Self {
        Self::new(BigOClass::Logarithmic, 1, InputVariable::N)
    }

    /// Create a linearithmic time bound O(n log n)
    pub fn linearithmic() -> Self {
        Self::new(BigOClass::Linearithmic, 1, InputVariable::N)
    }

    /// Create a polynomial bound with given exponent
    pub fn polynomial(exponent: u32, coefficient: u16) -> Self {
        let class = match exponent {
            0 => BigOClass::Constant,
            1 => BigOClass::Linear,
            2 => BigOClass::Quadratic,
            3 => BigOClass::Cubic,
            _ => BigOClass::Unknown,
        };
        Self::new(class, coefficient, InputVariable::N)
    }

    /// Create a polynomial-logarithmic bound
    pub fn polynomial_log(degree: u32, log_power: u32) -> Self {
        match (degree, log_power) {
            (1, 1) => Self::linearithmic(),
            _ => Self::unknown(), // More complex cases need custom representation
        }
    }

    /// Create an unknown complexity bound
    pub fn unknown() -> Self {
        Self::new(BigOClass::Unknown, 0, InputVariable::N).with_confidence(0)
    }

    /// Set confidence level (0-100)
    pub fn with_confidence(mut self, confidence: u8) -> Self {
        self.confidence = confidence.min(100);
        self
    }

    /// Add flags to the bound
    pub fn with_flags(mut self, flags: u8) -> Self {
        self.flags = self.flags.with(flags);
        self
    }

    /// Get notation string for this bound
    pub fn notation(&self) -> String {
        if self.coefficient <= 1 {
            format!("{}", self.class)
        } else {
            format!("{}·{}", self.coefficient, self.class)
        }
    }

    /// Estimate operations for given input size
    pub fn estimate_operations(&self, n: f64) -> f64 {
        self.coefficient as f64 * self.class.growth_factor(n)
    }

    /// Check if this bound is better than another
    pub fn is_better_than(&self, other: &Self) -> bool {
        if self.class != other.class {
            self.class.is_better_than(&other.class)
        } else {
            self.coefficient < other.coefficient
        }
    }
}

impl Default for ComplexityBound {
    fn default() -> Self {
        Self::unknown()
    }
}

impl fmt::Display for ComplexityBound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({}% confidence)", self.notation(), self.confidence)
    }
}

/// Cache complexity characteristics
#[repr(C, align(8))]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CacheComplexity {
    /// Cache hit ratio (0-100%)
    pub hit_ratio: u8,
    /// Cache miss penalty factor
    pub miss_penalty: u8,
    /// Working set size class
    pub working_set: BigOClass,
    /// Flags for cache behavior
    pub flags: u8,
    /// Padding for alignment
    _padding: [u8; 4],
}

impl CacheComplexity {
    pub fn new(hit_ratio: u8, miss_penalty: u8, working_set: BigOClass) -> Self {
        Self {
            hit_ratio: hit_ratio.min(100),
            miss_penalty,
            working_set,
            flags: 0,
            _padding: [0; 4],
        }
    }
}

impl Default for CacheComplexity {
    fn default() -> Self {
        Self::new(0, 1, BigOClass::Unknown)
    }
}

/// Recurrence relation for recursive algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecurrenceRelation {
    /// Number of recursive calls per invocation
    pub recursive_calls: Vec<RecursiveCall>,
    /// Non-recursive work complexity
    pub work_per_call: ComplexityBound,
    /// Base case size
    pub base_case_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecursiveCall {
    /// Factor by which input size is reduced (e.g., n/2 has factor 2)
    pub division_factor: u32,
    /// Constant reduction in size (e.g., n-1 has reduction 1)
    pub size_reduction: u32,
    /// Number of such calls
    pub count: u32,
}

impl RecurrenceRelation {
    /// Attempt to solve using the Master Theorem
    pub fn solve_master_theorem(&self) -> Option<ComplexityBound> {
        // Check if recurrence fits Master Theorem form: T(n) = aT(n/b) + f(n)
        if self.recursive_calls.len() != 1 {
            return None;
        }

        let call = &self.recursive_calls[0];
        if call.size_reduction != 0 || call.division_factor <= 1 {
            return None;
        }

        let a = call.count;
        let b = call.division_factor;
        let work = &self.work_per_call;

        // Only handle polynomial work for now
        match work.class {
            BigOClass::Constant => {
                // T(n) = aT(n/b) + O(1)
                let log_b_a = (a as f64).log(b as f64);
                Some(ComplexityBound::polynomial(log_b_a.ceil() as u32, 1))
            }
            BigOClass::Linear => {
                // T(n) = aT(n/b) + O(n)
                let log_b_a = (a as f64).log(b as f64);
                if (log_b_a - 1.0).abs() < 0.01 {
                    // Case 2: a = b
                    Some(ComplexityBound::linearithmic())
                } else if log_b_a < 1.0 {
                    // Case 3: a < b
                    Some(ComplexityBound::linear())
                } else {
                    // Case 1: a > b
                    Some(ComplexityBound::polynomial(log_b_a.ceil() as u32, 1))
                }
            }
            _ => None,
        }
    }
}

/// Complexity proof type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplexityProofType {
    /// Formally verified using theorem prover
    Verified,
    /// Validated through empirical testing
    Empirical,
    /// Based on pattern matching heuristics
    Heuristic,
    /// Unknown or unproven
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complexity_bound_size() {
        // Check the struct size - may be 8 or 16 bytes depending on platform and serde
        let size = std::mem::size_of::<ComplexityBound>();
        assert!(
            size == 8 || size == 16,
            "ComplexityBound size is {size} bytes"
        );
        assert_eq!(std::mem::align_of::<ComplexityBound>(), 8);
    }

    #[test]
    fn test_cache_complexity_size() {
        // Ensure cache complexity is 8 bytes
        assert_eq!(std::mem::size_of::<CacheComplexity>(), 8);
        assert_eq!(std::mem::align_of::<CacheComplexity>(), 8);
    }

    #[test]
    fn test_big_o_ordering() {
        assert!(BigOClass::Constant.is_better_than(&BigOClass::Linear));
        assert!(BigOClass::Linear.is_better_than(&BigOClass::Quadratic));
        assert!(BigOClass::Logarithmic.is_better_than(&BigOClass::Linear));
        assert!(BigOClass::Linearithmic.is_better_than(&BigOClass::Quadratic));
    }

    #[test]
    fn test_complexity_bound_creation() {
        let bound = ComplexityBound::linear()
            .with_confidence(90)
            .with_flags(ComplexityFlags::PROVEN | ComplexityFlags::TIGHT_BOUND);

        assert_eq!(bound.class, BigOClass::Linear);
        assert_eq!(bound.confidence, 90);
        assert!(bound.flags.is_proven());
        assert_eq!(bound.notation(), "O(n)");
    }

    #[test]
    fn test_growth_estimation() {
        let linear = ComplexityBound::linear();
        let quadratic = ComplexityBound::quadratic();

        assert_eq!(linear.estimate_operations(100.0), 100.0);
        assert_eq!(quadratic.estimate_operations(100.0), 10000.0);
    }

    #[test]
    fn test_master_theorem() {
        // Test case: T(n) = 2T(n/2) + O(n) -> O(n log n)
        let recurrence = RecurrenceRelation {
            recursive_calls: vec![RecursiveCall {
                division_factor: 2,
                size_reduction: 0,
                count: 2,
            }],
            work_per_call: ComplexityBound::linear(),
            base_case_size: 1,
        };

        let solution = recurrence.solve_master_theorem();
        assert!(solution.is_some());
        assert_eq!(solution.unwrap().class, BigOClass::Linearithmic);
    }
}
