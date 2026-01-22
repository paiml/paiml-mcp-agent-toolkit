//! C and C++ language AST parsing strategies
//!
//! This module provides parsing strategies for C and C++ languages.
//! Split into parts for CB-040 compliance.

mod c_cpp_strategy;
mod c_cpp_tests;
mod c_cpp_tests_feature;
mod c_cpp_visitor;

pub use c_cpp_strategy::{CStrategy, CppStrategy};
pub use c_cpp_visitor::CTreeSitterVisitor;
