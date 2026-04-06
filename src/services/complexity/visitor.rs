#![cfg_attr(coverage_nightly, coverage(off))]
//! Zero-allocation complexity visitor for AST traversal.

use super::types::{ClassComplexity, ComplexityMetrics, FunctionComplexity};

/// Zero-allocation complexity visitor for AST traversal
pub struct ComplexityVisitor<'a> {
    pub complexity: &'a mut ComplexityMetrics,
    pub nesting_level: u8,
    pub current_function: Option<String>,
    pub functions: Vec<FunctionComplexity>,
    pub classes: Vec<ClassComplexity>,
}

impl<'a> ComplexityVisitor<'a> {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn calculate_cognitive_increment(&self, is_nesting_construct: bool) -> u16 {
        if is_nesting_construct {
            1 + u16::from(self.nesting_level.saturating_sub(1))
        } else {
            1
        }
    }

    /// Enter a nesting level
    #[inline(always)]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn enter_nesting(&mut self) {
        self.nesting_level = self.nesting_level.saturating_add(1);
        if self.nesting_level > self.complexity.nesting_max {
            self.complexity.nesting_max = self.nesting_level;
        }
    }

    /// Exit a nesting level
    #[inline(always)]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn exit_nesting(&mut self) {
        self.nesting_level = self.nesting_level.saturating_sub(1);
    }
}
