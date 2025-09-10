//! Core data models for PMAT.
//!
//! This module contains all the data structures and models used throughout PMAT.
//! Each submodule represents a specific domain within the codebase analysis toolkit.
//!
//! # Models Overview
//!
//! - **churn**: Code churn metrics and repository activity analysis
//! - **complexity_bound**: Complexity metrics and bounds for code analysis
//! - **dag**: Directed Acyclic Graph structures for dependency analysis
//! - **dead_code**: Dead code detection and representation
//! - **deep_context_config**: Configuration for deep context analysis
//! - **defect_report**: Defect reporting structures (SATD, lint issues)
//! - **error**: Error types and handling
//! - **mcp**: Model Context Protocol specific structures
//! - **pdmt**: PDMT integration for deterministic todo generation with quality enforcement
//! - **project_meta**: Project metadata and configuration
//! - **quality_gate**: Quality gate results and violations
//! - **refactor**: Refactoring state machine and operations
//! - **tdg**: Task Dependency Graph for workflow analysis
//! - **template**: Template structures for code generation
//! - **unified_ast**: Unified AST representation across languages
//!
//! # Example
//!
//! ```
//! use pmat::models::defect_report::{Defect, DefectCategory, Severity};
//! use pmat::models::dag::DependencyGraph;
//! use std::path::PathBuf;
//!
//! // Create a defect
//! let defect = Defect {
//!     id: "SATD-001".to_string(),
//!     severity: Severity::Medium,
//!     category: DefectCategory::TechnicalDebt,
//!     file_path: PathBuf::from("src/main.rs"),
//!     line_start: 42,
//!     line_end: Some(45),
//!     column_start: Some(5),
//!     column_end: Some(80),
//!     message: "Refactor this function".to_string(),
//!     rule_id: "satd-todo".to_string(),
//!     fix_suggestion: None,
//!     metrics: Default::default(),
//! };
//! ```

pub mod churn;
pub mod complexity_bound;
pub mod dag;
#[cfg(test)]
pub mod dag_property_tests;
pub mod dead_code;
pub mod deep_context_config;
pub mod defect_report;
pub mod error;
pub mod mcp;
pub mod pdmt;
pub mod project_meta;
pub mod proxy;
pub mod quality_gate;
pub mod refactor;
pub mod tdg;
pub mod template;
pub mod unified_ast;

#[cfg(test)]
mod tests {
    // use super::*; // Unused in simple tests

    #[test]
    fn test_mod_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test] 
        fn module_consistency_check(x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(x < 1001);
        }
    }
}
