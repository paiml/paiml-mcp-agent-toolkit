//! Mutation testing engine for PMAT
//!
//! AST-based mutation testing and fuzzing system for language-agnostic
//! test suite quality evaluation.

#![allow(ambiguous_glob_reexports)]

pub mod types;
pub mod operators;
pub mod language;
pub mod language_detector; // Sprint 63: Multi-language support
pub mod engine;
pub mod scoring;
pub mod rust_adapter;
pub mod typescript_adapter;
pub mod python_adapter;
pub mod go_adapter;
pub mod cpp_adapter;
pub mod wasm_adapter;
pub mod fuzzing;
pub mod coverage;
pub mod ml_predictor;
pub mod equivalent_detector;
pub mod distributed;
pub mod ci_cd_learning;
pub mod executor;
pub mod tree_sitter_operators;
pub mod typescript_tree_sitter_mutations;
pub mod typescript_mutation_generator;
pub mod python_tree_sitter_mutations;
pub mod python_mutation_generator;
pub mod go_tree_sitter_mutations;
pub mod go_mutation_generator;
pub mod cpp_tree_sitter_mutations;
pub mod cpp_mutation_generator;
pub mod rust_tree_sitter_mutations;
pub mod rust_mutation_generator;
pub mod guard;
pub mod state;
pub mod worker_monitor;
pub mod temp_file;

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

#[cfg(test)]
mod ml_predictor_tests;

#[cfg(test)]
mod cross_validation_test;

#[cfg(test)]
mod equivalent_detector_tests;

#[cfg(test)]
mod ml_integration_tests;

pub use types::*;
pub use operators::*;
pub use language::*;
pub use language_detector::*; // Sprint 63: Multi-language support
pub use engine::*;
pub use scoring::*;
pub use rust_adapter::*;
pub use typescript_adapter::*;
pub use python_adapter::*;
pub use go_adapter::*;
pub use cpp_adapter::*;
pub use wasm_adapter::*;
pub use fuzzing::*;
pub use coverage::*;
pub use ml_predictor::*;
pub use equivalent_detector::*;
pub use distributed::*;
pub use ci_cd_learning::*;
pub use executor::*;
pub use guard::*;
pub use state::*;
pub use worker_monitor::*;
pub use temp_file::*;
pub use tree_sitter_operators::*;
pub use typescript_tree_sitter_mutations::*;
pub use typescript_mutation_generator::*;
pub use python_tree_sitter_mutations::*;
pub use python_mutation_generator::*;
pub use go_tree_sitter_mutations::*;
pub use go_mutation_generator::*;
pub use cpp_tree_sitter_mutations::*;
pub use cpp_mutation_generator::*;
pub use rust_tree_sitter_mutations::*;
pub use rust_mutation_generator::*;
