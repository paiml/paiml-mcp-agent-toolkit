//! Mutation testing engine for PMAT
//!
//! AST-based mutation testing and fuzzing system for language-agnostic
//! test suite quality evaluation.

pub mod types;
pub mod operators;
pub mod language;
pub mod engine;
pub mod scoring;
pub mod rust_adapter;
pub mod typescript_adapter;
pub mod python_adapter;

#[cfg(test)]
mod typescript_adapter_tests;

#[cfg(test)]
mod python_adapter_tests;

pub use types::*;
pub use operators::*;
pub use language::*;
pub use engine::*;
pub use scoring::*;
pub use rust_adapter::*;
pub use typescript_adapter::*;
pub use python_adapter::*;
