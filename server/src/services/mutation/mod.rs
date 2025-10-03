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
pub mod go_adapter;
pub mod cpp_adapter;
pub mod fuzzing;
pub mod coverage;

#[cfg(test)]
mod typescript_adapter_tests;

#[cfg(test)]
mod python_adapter_tests;

#[cfg(test)]
mod go_adapter_tests;

#[cfg(test)]
mod cpp_adapter_tests;

#[cfg(test)]
mod advanced_operators_tests;

#[cfg(test)]
mod fuzzing_integration_tests;

pub use types::*;
pub use operators::*;
pub use language::*;
pub use engine::*;
pub use scoring::*;
pub use rust_adapter::*;
pub use typescript_adapter::*;
pub use python_adapter::*;
pub use go_adapter::*;
pub use cpp_adapter::*;
pub use fuzzing::*;
pub use coverage::*;
