#![cfg_attr(coverage_nightly, coverage(off))]
//! Incremental formal verification for WASM modules

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use wasmparser::{Operator, ValType};

// ---------------------------------------------------------------------------
// Type definitions
// ---------------------------------------------------------------------------

/// Incremental verifier for property-based safety checks
pub struct IncrementalVerifier {
    invariants: InvariantChecker,
    stack_analyzer: StackAnalyzer,
}

/// Verification result types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VerificationResult {
    Safe,
    OutOfBounds {
        offset: usize,
        size: usize,
    },
    TypeError {
        expected: String,
        found: Option<String>,
    },
    StackUnderflow,
    StackOverflow,
    InvalidIndirectCall,
    IntegerOverflow,
}

/// Invariant checker for safety properties
#[derive(Debug, Clone)]
struct InvariantChecker {
    invariants: Vec<SafetyInvariant>,
}

/// Safety invariant types
#[derive(Debug, Clone)]
enum SafetyInvariant {
    MemoryBounds,
    StackBalance,
    TypeSafety,
    NoIntegerOverflow,
}

/// Invariant violation record
#[derive(Debug, Clone)]

struct InvariantViolation {
    invariant: SafetyInvariant,
    location: usize,
    description: String,
}

/// Stack type analyzer for type checking
#[derive(Debug, Clone)]
struct StackAnalyzer {
    type_stack: Vec<ValType>,
}

/// Differential testing for cross-runtime validation
pub struct DifferentialTester {
    test_cases: Vec<TestCase>,
}

/// Test case for differential testing
#[derive(Debug, Clone, PartialEq)]
pub struct TestCase {
    pub inputs: Vec<i32>,
    pub expected_output: Option<Vec<i32>>,
}

/// Differential testing result
#[derive(Debug, Clone, PartialEq)]
pub enum DifferentialResult {
    Consistent,
    Divergence {
        test_case: TestCase,
        runtime1_result: Vec<i32>,
        runtime2_result: Vec<i32>,
    },
}

// ---------------------------------------------------------------------------
// Implementations (split into include files)
// ---------------------------------------------------------------------------

include!("verifier_core.rs");
include!("verifier_stack.rs");
include!("verifier_differential.rs");
include!("verifier_tests.rs");
include!("verifier_tests_stack.rs");
