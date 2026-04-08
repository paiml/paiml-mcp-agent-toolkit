#![cfg_attr(coverage_nightly, coverage(off))]
//! Verified complexity analyzer with multiple complexity metrics
//!
//! This module provides accurate complexity analysis using industry-standard
//! metrics including cyclomatic complexity, cognitive complexity (Sonar rules),
//! essential complexity, and Halstead software science metrics. It operates
//! on the unified AST to provide consistent analysis across languages.
//!
//! # Complexity Metrics
//!
//! - **Cyclomatic Complexity**: Measures independent paths through code (`McCabe`)
//! - **Cognitive Complexity**: Measures how difficult code is to understand (Sonar)
//! - **Essential Complexity**: Measures irreducible complexity after simplification
//! - **Halstead Metrics**: Software science metrics based on operators and operands
//!
//! # Cognitive Complexity Rules
//!
//! Following Sonar's cognitive complexity specification:
//! - +1 for each control flow statement (if, while, for, etc.)
//! - +1 for each logical operator in boolean expressions
//! - +nesting level for nested structures
//! - +1 for switch/match cases
//! - +1 for recursive calls
//!
//! # Example
//!
//! ```ignore
//! use pmat::services::verified_complexity::VerifiedComplexityAnalyzer;
//! use pmat::models::unified_ast::UnifiedAstNode;
//!
//! let mut analyzer = VerifiedComplexityAnalyzer::new();
//!
//! // Analyze a function AST node
//! let ast = UnifiedAstNode::default(); // Your AST here
//! let metrics = analyzer.analyze_function(&ast);
//!
//! println!("Cyclomatic Complexity: {}", metrics.cyclomatic);
//! println!("Cognitive Complexity: {}", metrics.cognitive);
//! println!("Halstead Volume: {:.2}", metrics.halstead.volume());
//! println!("Halstead Difficulty: {:.2}", metrics.halstead.difficulty());
//!
//! // Thresholds for code quality
//! if metrics.cognitive > 15 {
//!     println!("⚠️ High cognitive complexity - consider refactoring");
//! }
//! ```ignore

use crate::models::unified_ast::{AstKind, ExprKind, StmtKind, UnifiedAstNode};
use std::collections::HashMap;

/// Verified complexity analyzer implementing cognitive complexity per Sonar rules
pub struct VerifiedComplexityAnalyzer {
    /// Current nesting level for cognitive complexity calculation
    nesting_level: u32,
}

/// Complexity metrics for a function/method
#[derive(Debug, Clone, Copy)]
pub struct ComplexityMetrics {
    pub cyclomatic: u32,
    pub cognitive: u32,
    pub essential: u32,
    pub halstead: HalsteadMetrics,
}

/// Halstead software science metrics
#[derive(Debug, Clone, Copy, Default)]
#[allow(non_snake_case)]
pub struct HalsteadMetrics {
    pub n1: u32, // Number of distinct operators
    pub n2: u32, // Number of distinct operands
    pub N1: u32, // Total number of operators
    pub N2: u32, // Total number of operands
}

impl HalsteadMetrics {
    /// Calculate derived Halstead metrics
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn volume(&self) -> f64 {
        let n = f64::from(self.n1 + self.n2);
        #[allow(non_snake_case)]
        let N = f64::from(self.N1 + self.N2);
        N * n.log2()
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Difficulty.
    pub fn difficulty(&self) -> f64 {
        if self.n2 == 0 {
            return 0.0;
        }
        (f64::from(self.n1) / 2.0) * (f64::from(self.N2) / f64::from(self.n2))
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Effort.
    pub fn effort(&self) -> f64 {
        self.volume() * self.difficulty()
    }
}

impl Default for VerifiedComplexityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

// Analysis methods for VerifiedComplexityAnalyzer
include!("verified_complexity_analysis.rs");

// Unit tests - core metrics and Halstead
include!("verified_complexity_tests.rs");

// Visitor, cognitive, and Halstead token tests + property tests
include!("verified_complexity_visitor_tests.rs");
