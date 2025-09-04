//! Rust AST analysis - MIGRATION IN PROGRESS
//!
//! This module is being migrated to the new unified AST architecture.
//! It now acts as a facade, delegating to the compatibility layer.
//!
//! Migration status: Using compatibility shim
//! Target: server/src/ast/languages/rust.rs

// Re-export compatibility functions
pub use super::ast_rust_compat::{
    analyze_rust_file,
    analyze_rust_file_with_classifier,
    analyze_rust_file_with_complexity,
    analyze_rust_file_with_complexity_and_classifier,
};

// Keep any other types that were exported
// (The rest of the original implementation is preserved in ast_rust_compat.rs)