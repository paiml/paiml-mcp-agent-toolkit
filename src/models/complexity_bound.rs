#![cfg_attr(coverage_nightly, coverage(off))]
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

/// Flags for complexity bound properties
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[repr(transparent)]
pub struct ComplexityFlags(u8);

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
    ///
    /// `#[serde(skip)]`: this is `#[repr(C)]` alignment filler, not data. It
    /// was reaching MCP clients as `"_padding":[0,0]` beside real measurements
    /// — a field a reader has to guess the meaning of, whose only possible
    /// value is zero.
    #[serde(skip)]
    _padding: [u8; 2],
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

/// A single recursive call description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecursiveCall {
    /// Factor by which input size is reduced (e.g., n/2 has factor 2)
    pub division_factor: u32,
    /// Constant reduction in size (e.g., n-1 has reduction 1)
    pub size_reduction: u32,
    /// Number of such calls
    pub count: u32,
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

// Impl blocks for BigOClass, InputVariable, ComplexityFlags,
// and their Display trait implementations
include!("complexity_bound_impls.rs");

// Impl blocks for ComplexityBound, CacheComplexity, RecurrenceRelation,
// and their Default/Display trait implementations
include!("complexity_bound_core.rs");

// Unit tests and property tests
include!("complexity_bound_tests.rs");
