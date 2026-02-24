#![cfg_attr(coverage_nightly, coverage(off))]
//! Pattern-based security analysis for WASM bytecode

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::ops::Range;
use wasmparser::{Operator, Payload};

/// Pattern detector for vulnerability scanning
#[derive(Debug, Clone)]
pub struct PatternDetector {
    patterns: Vec<VulnerabilityPattern>,
    found: Vec<VulnerabilityMatch>,
}

/// Vulnerability pattern definition
#[derive(Debug, Clone)]
pub struct VulnerabilityPattern {
    pub name: &'static str,
    pub opcodes: Vec<OpcodePattern>,
    pub severity: Severity,
}

/// Pattern matching strategies
#[derive(Debug, Clone)]
pub enum OpcodePattern {
    /// Exact sequence of operators
    Sequence(Vec<OperatorMatcher>),

    /// Operators within specified distance
    Within {
        distance: usize,
        operators: Vec<OperatorMatcher>,
    },

    /// Target not preceded by guards
    NotPrecededBy {
        target: OperatorMatcher,
        guards: Vec<OperatorMatcher>,
    },
}

/// Operator matcher for pattern matching
#[derive(Debug, Clone)]
pub enum OperatorMatcher {
    I32Const,
    I32Add,
    I32Sub,
    I32Mul,
    I32DivS,
    I32DivU,
    I32RemU,
    I32And,
    I32Or,
    I32Xor,
    I32Eqz,
    I32Eq,
    I32Ne,
    I32LtS,
    I32LtU,
    I32GtS,
    I32GtU,
    I32Load,
    I32Store,
    I64Load,
    I64Store,
    BrIf,
    Br,
    Call,
    CallIndirect,
    MemoryGrow,
    MemorySize,
    Any,
}

/// Vulnerability severity levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

/// Matched vulnerability instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityMatch {
    pub pattern: String,
    pub location: Range<usize>,
    pub severity: Severity,
    pub operator_index: usize,
}

// Pattern detection and vulnerability matching logic
include!("security_patterns.rs");

// Operator matching logic
include!("security_matcher.rs");

// Tests: property tests, pattern detection, and scanning
include!("security_tests.rs");

// Tests: operator matcher, severity, vulnerability match, default patterns
include!("security_tests_matcher.rs");
