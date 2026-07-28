//! Python AST analysis - MIGRATION IN PROGRESS
//!
//! This module is being migrated to the new unified AST architecture.
//! It now acts as a facade, delegating to the compatibility layer.
//!
//! Migration status: Using compatibility shim
//! Target: server/src/ast/languages/python.rs

// Re-export compatibility functions
pub use super::ast_python_compat::{
    analyze_python_file, analyze_python_file_with_classifier, analyze_python_file_with_complexity,
};

// Keep any other types that were exported
// (The rest of the original implementation is preserved in ast_python_compat.rs)
