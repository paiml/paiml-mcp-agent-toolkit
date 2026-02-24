#![cfg_attr(coverage_nightly, coverage(off))]
//! Refactoring state machine and operation models.
//!
//! This module defines the state machine that orchestrates the automated
//! refactoring process. The state machine ensures refactoring operations
//! are applied systematically with proper validation at each step.
//!
//! # State Machine Flow
//!
//! 1. **Scan**: Identify target files for refactoring
//! 2. **Analyze**: Analyze code quality metrics
//! 3. **Plan**: Generate refactoring plan based on violations
//! 4. **Refactor**: Apply refactoring operations
//! 5. **Test**: Run tests to validate changes
//! 6. **Lint**: Check code style and quality
//! 7. **Emit**: Report results
//! 8. **Complete**: Summarize refactoring session
//!
//! # Example
//!
//! ```
//! use pmat::models::refactor::{RefactorStateMachine, RefactorConfig, State};
//! use std::path::PathBuf;
//!
//! let config = RefactorConfig::default();
//! let targets = vec![PathBuf::from("src/complex_module.rs")];
//!
//! let mut state_machine = RefactorStateMachine::new(targets, config);
//!
//! // Start with scanning
//! match &state_machine.current {
//!     State::Scan { targets } => {
//!         println!("Scanning {} files", targets.len());
//!     }
//!     _ => {}
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// Type definitions: structs, enums, and repr types
include!("refactor_types.rs");

// Impl blocks: Default impls, RefactorStateMachine, Violation::to_op
include!("refactor_impls.rs");

// Unit tests
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    include!("refactor_tests.rs");
    include!("refactor_tests_part2.rs");
}

// Property-based tests
include!("refactor_property_tests.rs");
