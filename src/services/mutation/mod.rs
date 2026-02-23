#![cfg_attr(coverage_nightly, coverage(off))]
//! Mutation testing engine for PMAT
//!
//! AST-based mutation testing and fuzzing system for language-agnostic
//! test suite quality evaluation.

#![allow(ambiguous_glob_reexports)]

pub mod cargo_mutants_wrapper;
pub mod ci_cd_learning;
pub mod coverage;
pub mod cpp_adapter;
pub mod cpp_mutation_generator;
pub mod cpp_tree_sitter_mutations;
pub mod distributed;
pub mod engine;
pub mod equivalent_detector;
pub mod executor;
pub mod fuzzing;
pub mod go_adapter;
pub mod go_mutation_generator;
pub mod go_tree_sitter_mutations;
pub mod guard;
pub mod json_parser; // Sprint 70: cargo-mutants JSON parser
pub mod language;
pub mod language_detector; // Sprint 63: Multi-language support
pub mod lua_adapter;
pub mod ml_predictor;
pub mod operators;
pub mod python_adapter;
pub mod python_mutation_generator;
pub mod python_tree_sitter_mutations;
pub mod rust_adapter;
pub mod rust_mutation_generator;
pub mod rust_tree_sitter_mutations;
pub mod scoring;
pub mod state;
pub mod temp_file;
pub mod tree_sitter_operators;
pub mod types;
pub mod typescript_adapter;
pub mod typescript_mutation_generator;
pub mod typescript_tree_sitter_mutations;
pub mod wasm_adapter;
pub mod worker_monitor; // Sprint 70: cargo-mutants subprocess wrapper

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod typescript_adapter_tests;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod python_adapter_tests;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod go_adapter_tests;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod cpp_adapter_tests;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod advanced_operators_tests;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod fuzzing_integration_tests;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod ml_predictor_tests;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod cross_validation_test;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod equivalent_detector_tests;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod ml_integration_tests;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests;

pub use ci_cd_learning::*;
pub use coverage::*;
pub use cpp_adapter::*;
pub use cpp_mutation_generator::*;
pub use cpp_tree_sitter_mutations::*;
pub use distributed::*;
pub use engine::*;
pub use equivalent_detector::*;
pub use executor::*;
pub use fuzzing::*;
pub use go_adapter::*;
pub use go_mutation_generator::*;
pub use go_tree_sitter_mutations::*;
pub use guard::*;
pub use language::*;
pub use language_detector::*; // Sprint 63: Multi-language support
pub use lua_adapter::*;
pub use ml_predictor::*;
pub use operators::*;
pub use python_adapter::*;
pub use python_mutation_generator::*;
pub use python_tree_sitter_mutations::*;
pub use rust_adapter::*;
pub use rust_mutation_generator::*;
pub use rust_tree_sitter_mutations::*;
pub use scoring::*;
pub use state::*;
pub use temp_file::*;
pub use tree_sitter_operators::*;
pub use types::*;
pub use typescript_adapter::*;
pub use typescript_mutation_generator::*;
pub use typescript_tree_sitter_mutations::*;
pub use wasm_adapter::*;
pub use worker_monitor::*;
