#![cfg_attr(coverage_nightly, coverage(off))]
//! Enhanced AST parser using syn for Rust code analysis

mod parser;
mod types;
pub(crate) mod visitor;

// Re-export public types
pub use parser::EnhancedParser;
pub use types::{CacheStats, CachedSyntax};

#[cfg(test)]
mod coverage_tests_advanced;
#[cfg(test)]
mod coverage_tests_core;
#[cfg(test)]
mod property_tests;
#[cfg(test)]
mod tests;
