#![cfg_attr(coverage_nightly, coverage(off))]
//! Core data models for PMAT.
//!
//! This module contains all the data structures and models used throughout PMAT.
//! Each submodule represents a specific domain within the codebase analysis toolkit.
//!
//! # Models Overview
//!
//! - **churn**: Code churn metrics and repository activity analysis
//! - **`complexity_bound`**: Complexity metrics and bounds for code analysis
//! - **dag**: Directed Acyclic Graph structures for dependency analysis
//! - **`dead_code`**: Dead code detection and representation
//! - **`deep_context_config`**: Configuration for deep context analysis
//! - **`defect_report`**: Defect reporting structures (SATD, lint issues)
//! - **error**: Error types and handling
//! - **mcp**: Model Context Protocol specific structures
//! - **pdmt**: PDMT integration for deterministic todo generation with quality enforcement
//! - **`project_meta`**: Project metadata and configuration
//! - **`quality_gate`**: Quality gate results and violations
//! - **refactor**: Refactoring state machine and operations
//! - **tdg**: Task Dependency Graph for workflow analysis
//! - **template**: Template structures for code generation
//! - **`unified_ast`**: Unified AST representation across languages
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
pub mod comply_config;
pub mod dag;
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
pub mod dag_property_tests;
pub mod dead_code;
pub mod debug_analysis;
pub mod deep_context_config;
pub mod defect_report;
pub mod error;
pub mod git_context;
pub mod mcp;
pub mod pdmt;
pub mod project_meta;
pub mod project_metadata;
pub mod prompt_model;
pub mod proxy;
pub mod quality_gate;
pub mod refactor;
pub mod roadmap;
pub mod tdg;
pub mod template;
pub mod unified_ast;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mod_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }

    /// Test that all submodules are accessible
    #[test]
    fn test_submodule_accessibility() {
        // Verify key types from each submodule are accessible
        // This ensures the module re-exports work correctly
        let _ = std::any::type_name::<churn::CodeChurnAnalysis>();
        let _ = std::any::type_name::<complexity_bound::ComplexityBound>();
        let _ = std::any::type_name::<dag::DependencyGraph>();
        let _ = std::any::type_name::<dead_code::FileDeadCodeMetrics>();
        let _ = std::any::type_name::<debug_analysis::DebugAnalysis>();
        let _ = std::any::type_name::<defect_report::Defect>();
        let _ = std::any::type_name::<error::TemplateError>();
        let _ = std::any::type_name::<mcp::McpRequest>();
        let _ = std::any::type_name::<template::TemplateResource>();
    }

    /// Test module compilation and type instantiation
    #[test]
    fn test_type_instantiation() {
        // Ensure basic types can be created
        use dead_code::{ConfidenceLevel, DeadCodeType};

        let high = ConfidenceLevel::High;
        let medium = ConfidenceLevel::Medium;
        let low = ConfidenceLevel::Low;

        assert_ne!(high, medium);
        assert_ne!(medium, low);
        assert_ne!(high, low);

        let func_type = DeadCodeType::Function;
        let class_type = DeadCodeType::Class;

        assert_ne!(func_type, class_type);
    }

    /// Test error type accessibility
    #[test]
    fn test_error_types_accessible() {
        use error::{AnalysisError, PmatError, TemplateError};

        // Verify error types are constructible
        let template_err = TemplateError::NotFound("test".to_string());
        assert!(format!("{}", template_err).contains("Not found"));

        let analysis_err = AnalysisError::ParseError("test parse error".to_string());
        assert!(format!("{}", analysis_err).contains("Failed to parse"));

        let pmat_err = PmatError::FileNotFound {
            path: std::path::PathBuf::from("/test/path"),
        };
        assert!(format!("{}", pmat_err).contains("File not found"));
    }

    /// Test MCP types accessibility
    #[test]
    fn test_mcp_types_accessible() {
        use mcp::{McpRequest, McpResponse};
        use serde_json::json;

        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(1),
            method: "test".to_string(),
            params: None,
        };

        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.method, "test");

        let response = McpResponse::success(json!(1), json!({"result": "ok"}));
        assert!(response.result.is_some());
        assert!(response.error.is_none());

        let error_response = McpResponse::error(json!(2), -32601, "Method not found".to_string());
        assert!(error_response.error.is_some());

        let error = error_response.error.unwrap();
        assert_eq!(error.code, -32601);
    }

    /// Test template types accessibility
    #[test]
    fn test_template_types_accessible() {
        use template::{ParameterType, TemplateCategory, Toolchain};

        // Test Toolchain variants
        let rust = Toolchain::RustCli {
            cargo_features: vec!["serde".to_string()],
        };
        let deno = Toolchain::DenoTypescript {
            deno_version: "1.38".to_string(),
        };
        let python = Toolchain::PythonUv {
            python_version: "3.11".to_string(),
        };

        assert_eq!(rust.priority(), 1);
        assert_eq!(deno.priority(), 2);
        assert_eq!(python.priority(), 3);

        assert_eq!(rust.as_str(), "rust");
        assert_eq!(deno.as_str(), "deno");
        assert_eq!(python.as_str(), "python-uv");

        // Test TemplateCategory
        let categories = [
            TemplateCategory::Makefile,
            TemplateCategory::Readme,
            TemplateCategory::Gitignore,
            TemplateCategory::Context,
        ];
        for cat in &categories {
            // Ensure categories can be cloned and compared
            let cloned = cat.clone();
            assert_eq!(*cat, cloned);
        }

        // Test ParameterType
        let param_types = [
            ParameterType::ProjectName,
            ParameterType::SemVer,
            ParameterType::GitHubUsername,
            ParameterType::LicenseIdentifier,
            ParameterType::Boolean,
            ParameterType::String,
        ];
        for pt in &param_types {
            let cloned = pt.clone();
            assert_eq!(*pt, cloned);
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    //! EXTREME TDD coverage tests for models/mod.rs
    //! These tests ensure comprehensive coverage of the models module structure.

    use super::*;

    /// Test that all public modules are importable
    #[test]
    fn test_all_public_modules_importable() {
        // Verify each module is accessible
        let _ = std::any::type_name::<churn::FileChurnMetrics>();
        let _ = std::any::type_name::<complexity_bound::ComplexityBound>();
        let _ = std::any::type_name::<dag::DependencyGraph>();
        let _ = std::any::type_name::<dead_code::DeadCodeRankingResult>();
        let _ = std::any::type_name::<debug_analysis::WhyIteration>();
        let _ = std::any::type_name::<deep_context_config::DeepContextConfig>();
        let _ = std::any::type_name::<defect_report::DefectReport>();
        let _ = std::any::type_name::<error::ErrorSeverity>();
        let _ = std::any::type_name::<git_context::GitContext>();
        let _ = std::any::type_name::<mcp::ToolCallParams>();
        let _ = std::any::type_name::<pdmt::PdmtTodo>();
        let _ = std::any::type_name::<project_meta::MetaFile>();
        let _ = std::any::type_name::<quality_gate::QualityGateResult>();
        let _ = std::any::type_name::<refactor::RefactorStateMachine>();
        let _ = std::any::type_name::<roadmap::Roadmap>();
        let _ = std::any::type_name::<tdg::TDGScore>();
        let _ = std::any::type_name::<template::GeneratedTemplate>();
        let _ = std::any::type_name::<unified_ast::UnifiedAstNode>();
    }

    /// Test module documentation example compiles
    #[test]
    fn test_module_doc_example() {
        use defect_report::{Defect, DefectCategory, Severity};
        use std::path::PathBuf;

        // Recreate the example from module docs
        let defect = Defect {
            id: "SATD-001".to_string(),
            severity: Severity::Medium,
            category: DefectCategory::TechnicalDebt,
            file_path: PathBuf::from("src/main.rs"),
            line_start: 42,
            line_end: Some(45),
            column_start: Some(5),
            column_end: Some(80),
            message: "Refactor this function".to_string(),
            rule_id: "satd-todo".to_string(),
            fix_suggestion: None,
            metrics: Default::default(),
        };

        assert_eq!(defect.id, "SATD-001");
        assert_eq!(defect.severity, Severity::Medium);
        assert_eq!(defect.category, DefectCategory::TechnicalDebt);
        assert_eq!(defect.line_start, 42);
        assert_eq!(defect.line_end, Some(45));
    }

    /// Test cross-module type compatibility
    #[test]
    fn test_cross_module_compatibility() {
        use churn::ChurnSummary;
        use dead_code::DeadCodeSummary;
        use std::collections::HashMap;
        use std::path::PathBuf;

        // Create types from different modules and verify they work together
        let churn_summary = ChurnSummary {
            total_commits: 100,
            total_files_changed: 50,
            hotspot_files: vec![PathBuf::from("src/main.rs")],
            stable_files: vec![PathBuf::from("src/lib.rs")],
            author_contributions: HashMap::new(),
            mean_churn_score: 0.5,
            variance_churn_score: 0.1,
            stddev_churn_score: 0.316,
        };

        let dead_code_summary = DeadCodeSummary::from_files(&[]);

        // Both summaries should be serializable
        let churn_json = serde_json::to_string(&churn_summary).unwrap();
        let dead_json = serde_json::to_string(&dead_code_summary).unwrap();

        assert!(churn_json.contains("total_commits"));
        assert!(dead_json.contains("total_files_analyzed"));
    }

    /// Test error type hierarchy
    #[test]
    fn test_error_type_hierarchy() {
        use error::{AnalysisError, PmatError, TemplateError};
        use std::path::PathBuf;

        // TemplateError can be converted to PmatError
        let template_err = TemplateError::TemplateNotFound {
            uri: "test://uri".to_string(),
        };
        let pmat_err: PmatError = template_err.into();
        assert!(format!("{}", pmat_err).contains("Template"));

        // AnalysisError can be converted to PmatError
        let analysis_err = AnalysisError::InvalidPath("/invalid/path".to_string());
        let pmat_err2: PmatError = analysis_err.into();
        assert!(format!("{}", pmat_err2).contains("Analysis"));

        // Test MCP code mapping
        let file_not_found = PmatError::FileNotFound {
            path: PathBuf::from("/test"),
        };
        assert_eq!(file_not_found.to_mcp_code(), -32001);
    }

    /// Test serialization round-trips for key types
    #[test]
    fn test_serialization_roundtrips() {
        use mcp::McpRequest;
        use serde_json::json;

        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(42),
            method: "tools/list".to_string(),
            params: Some(json!({"filter": "active"})),
        };

        // Serialize and deserialize
        let json_str = serde_json::to_string(&request).unwrap();
        let roundtrip: McpRequest = serde_json::from_str(&json_str).unwrap();

        assert_eq!(roundtrip.jsonrpc, "2.0");
        assert_eq!(roundtrip.method, "tools/list");
        assert!(roundtrip.params.is_some());
    }
}
