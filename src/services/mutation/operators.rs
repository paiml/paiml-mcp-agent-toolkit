#![cfg_attr(coverage_nightly, coverage(off))]
//! Mutation operators for AST transformation

use super::types::*;
use anyhow::Result;
use syn::{BinOp, Expr, UnOp};

/// Trait for mutation operators
pub trait MutationOperator: Send + Sync {
    /// Name of this operator
    fn name(&self) -> &str;

    /// Operator type
    fn operator_type(&self) -> MutationOperatorType;

    /// Can this operator mutate the given AST node?
    fn can_mutate(&self, expr: &Expr) -> bool;

    /// Generate mutants for the given AST node
    fn mutate(&self, expr: &Expr, location: SourceLocation) -> Result<Vec<Expr>>;

    /// Estimated kill probability (0.0 - 1.0)
    fn kill_probability(&self) -> f64 {
        0.5 // Default 50%
    }
}

include!("operators_basic.rs");
include!("operators_advanced.rs");
include!("operators_tests.rs");
