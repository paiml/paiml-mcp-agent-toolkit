#![allow(unused)]
#![cfg_attr(coverage_nightly, coverage(off))]
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use syn::{self, visit::Visit, Expr, Stmt};

// Core types: Complexity enum, AlgorithmPattern enum, path_to_string helper
include!("efficiency_enhanced_types.rs");

// SymbolicExecutor struct and core analysis methods
include!("efficiency_enhanced_symbolic_executor.rs");

// Algorithm pattern detection (analyze_algorithm_patterns, is_dynamic_programming)
include!("efficiency_enhanced_algorithm_detection.rs");

// Visit trait implementations for SymbolicExecutor and RecursionDetector
include!("efficiency_enhanced_visitors.rs");

// SpaceComplexityAnalyzer, Allocation types, and Visit implementation
include!("efficiency_enhanced_space_analysis.rs");

// Tests: Complexity, SymbolicExecutor, AlgorithmPattern
include!("efficiency_enhanced_tests.rs");

// Tests: SpaceComplexityAnalyzer, algorithm detection, recursion, DP
include!("efficiency_enhanced_tests_part2.rs");
