//! Unified AST representation for cross-language code analysis
//!
//! This module provides a language-agnostic AST representation that enables
//! consistent analysis across Rust, TypeScript/JavaScript, and Python codebases.
//! Enhanced with formal verification metadata for proof-enriched ASTs.

mod kinds;
mod location;
mod node;
mod proof;
mod storage;
mod types;

// Re-export all public types for backward compatibility
pub use kinds::*;
pub use location::*;
pub use node::*;
pub use proof::*;
pub use storage::*;
pub use types::*;

// Tests extracted to core_tests.rs for file health compliance (CB-040)
#[cfg(test)]
#[path = "core_tests.rs"]
mod tests;
