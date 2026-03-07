#![cfg_attr(coverage_nightly, coverage(off))]
//! Ruchy Language Support for PMAT
//!
//! This module provides AST parsing and complexity analysis for the Ruchy programming language.
//! Ruchy is a Rust-like language with Swift/Kotlin ergonomics that transpiles to Rust.
//!
//! ## Ruchy Language Features
//! - Functions: `fun name(params) -> ReturnType { ... }`
//! - Control flow: `if`, `while`, `for`, `match`
//! - Classes: `class Name { ... }`
//! - Actors: `actor Name { ... }`
//! - Traits: `trait Name { ... }`
//! - Pattern matching and pipeline operators
//!
//! ## Example
//! ```ruchy
//! fun fibonacci(n: i32) -> i32 {
//!     if n <= 1 {
//!         n
//!     } else {
//!         fibonacci(n - 1) + fibonacci(n - 2)
//!     }
//! }
//! ```

mod complexity;
mod lexer;
mod parser;
mod types;

#[cfg(test)]
mod tests;

// Re-export public types
pub use complexity::RuchyComplexityAnalyzer;
pub use lexer::RuchyLexer;
pub use parser::analyze_ruchy_file;
pub use types::{
    ActorInfo, DeadlockWarning, MessageFlow, RuchyActorAnalysis, RuchyAst, RuchyDeadCode,
    RuchyImport, RuchyToken, RuchyType,
};

#[cfg(feature = "ruchy-ast")]
pub use parser::{analyze_ruchy_file_with_parser, RuchyAstAnalyzer};
