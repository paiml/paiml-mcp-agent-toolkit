#![cfg_attr(coverage_nightly, coverage(off))]

mod compiler;
mod examples;
mod fluent;
mod macros;

// Test submodules
mod coverage_tests_compiler;
mod coverage_tests_conditionals;
mod coverage_tests_examples;
mod coverage_tests_fluent;
mod tests;

// Re-export everything publicly so external code doesn't break
pub use compiler::DslCompiler;
pub use examples::WORKFLOW_DSL_EXAMPLE;
pub use fluent::{ConditionalElse, ConditionalFlow, FluentWorkflow};

// Re-export parent types used by submodules
use super::*;
