#![cfg_attr(coverage_nightly, coverage(off))]
//! Core services for code analysis and refactoring.
//!
//! This module contains all the analysis engines, detectors, and services that power
//! PMAT's code quality toolkit. Services are designed following the Toyota Way
//! principles of modularity and single responsibility.
//!
//! # Service Architecture (Per SPECIFICATION.md Section 2)
//!
//! Services now implement a unified `Service` trait for consistency and composability.
//! - **`service_base`**: Core Service trait and `ServiceRegistry`
//! - **`analysis_service`**: Unified analysis service
//! - **`quality_gate_service`**: Quality enforcement service
//!
//! # Service Categories
//!
//! ## Analysis Services
//! - **analyzer**: Unified analyzer framework (Toyota Way consolidation)
//! - **complexity**: Cyclomatic complexity analysis
//! - **`satd_detector`**: Self-Admitted Technical Debt detection
//! - **`dead_code_detector`**: Unused code identification
//! - **`duplicate_detector`**: Code duplication analysis
//! - **`big_o_analyzer`**: Algorithmic complexity analysis
//! - **`coupling_analyzer`**: Module coupling metrics
//!
//! ## AST Services
//! - **`ast_rust`**: Rust AST analysis
//! - **`ast_typescript`**: TypeScript/JavaScript analysis
//! - **`ast_python`**: Python code analysis
//! - **`ast_c/ast_cpp`**: C/C++ analysis
//! - **`ast_kotlin`**: Kotlin analysis
//!
//! ## Core Services
//! - **`context_generator`**: AI-ready context generation
//! - **`refactor_engine`**: Automated refactoring
//! - **`quality_gate`**: Code quality enforcement
//! - **`template_engine`**: Code generation templates
//!
//! # Example
//!
//! ```ignore
//! use pmat::services::ast_rust::analyze_rust_file_with_complexity;
//! use std::path::Path;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let result = analyze_rust_file_with_complexity(Path::new("src/main.rs")).await?;
//! println!("Total complexity: {:?}", result.total_complexity);
//! # Ok(())
//! # }
//! ```

// Service architecture (SPECIFICATION.md Section 2)
pub mod analysis_service;
pub mod analyzer; // Toyota Way: Unified analyzer framework
pub mod asset_validator; // R-10: Non-code asset layout validation service
pub mod ast; // Toyota Way: Unified AST module (consolidates 25+ AST files)
pub mod commit_classifier; // Sovereign Stack: ML-based commit classification
pub mod contract_index; // R-9: O(1) contract binding lookup from binding-index.json
#[cfg(feature = "dap")]
pub mod dap; // Sprint 71: Debug Adapter Protocol server (optional - 13 transitive deps)
pub mod detection; // Toyota Way: Unified detection framework (consolidates detection services)
pub mod facades;
pub mod language_analyzer;
pub mod language_registry;
pub mod languages;
pub mod memory_integration;
pub mod memory_manager;
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
pub mod memory_property_tests;
pub mod metric_trends; // Phase 3: O(1) Quality Gates trend analysis
pub mod perfection_score; // master-plan-pmat-work-system.md: 200-point unified quality score
pub mod quality_gate_service;
pub mod service_adapter;
pub mod service_base;
pub mod service_communication;
pub mod service_composition;
pub mod service_lifecycle;
pub mod service_registry;

pub mod accurate_complexity_analyzer;
#[cfg(feature = "analytics-simd")]
pub mod analytics_backend; // Issue #79: Backend selection (GPU/SIMD/Scalar) and statistical helpers
#[cfg(feature = "analytics-simd")]
pub mod analytics_top_k; // Issue #79: Top-K selection algorithm (O(N) vs O(N log N))
pub mod artifact_writer;
#[cfg(feature = "c-ast")]
pub mod ast_c;
#[cfg(feature = "c-ast")]
pub mod ast_c_compat; // Compatibility layer for C AST migration
#[cfg(feature = "cpp-ast")]
pub mod ast_cpp;
#[cfg(feature = "cpp-ast")]
pub mod ast_cpp_compat; // Compatibility layer for C++ AST migration
#[cfg(feature = "kotlin-ast")]
pub mod ast_kotlin;
#[cfg(feature = "python-ast")]
pub mod ast_python;
#[cfg(feature = "python-ast")]
pub mod ast_python_compat; // Compatibility layer for Python AST migration
pub mod ast_rust;
pub mod ast_rust_compat; // Compatibility layer during migration
pub mod ast_strategies;
#[cfg(feature = "typescript-ast")]
pub mod ast_typescript;
#[cfg(feature = "typescript-ast")]
pub mod ast_typescript_compat; // Compatibility layer for TypeScript AST migration
pub mod big_o_analyzer;
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod big_o_analyzer_property_tests;
pub mod cache;
pub mod canonical_query;
pub mod cargo_dead_code_analyzer;
pub mod clippy_fix;
pub mod code_intelligence;
pub mod complexity;
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod complexity_analyzer_tests;
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod complexity_file_extraction_tests;
pub mod complexity_patterns;
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod complexity_property_tests;
pub mod configuration_service;
pub mod context;
pub mod context_graph; // O(1) Context Graph: trueno-graph integration for fast symbol lookups
pub mod coupling_analyzer;
pub mod coverage_improvement;
pub mod dag_builder;
pub mod dag_call_edges;
pub mod dag_complexity;
pub mod dag_pipeline;
pub mod dead_code_analyzer;
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod dead_code_analyzer_tests;
pub mod dead_code_multi_language; // BUG-004: Multi-language dead code analysis
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod dead_code_property_tests;
pub mod dead_code_prover;
pub mod deep_context;
pub mod defect_analyzer;
pub mod defect_analyzers;
pub mod defect_detector; // Known Defects detection (Sprint: Known Defects v2.1)
pub mod defect_report_service;
// pub mod deep_context_orchestrator; // TRACKED: Fix AST node compatibility
pub mod brick_score; // ComputeBrick Profiling Score (PMAT-446) - trueno/realizar ecosystem
pub mod debug_formatters;
pub mod defect_probability;
pub mod deterministic_mermaid_engine;
pub mod doc_validator;
pub mod dogfooding_engine;
pub mod duplicate_detector;
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod duplicate_detector_property_tests;
pub mod embedded_templates;
pub mod enhanced_ast_visitor;
pub mod enhanced_language_detection; // BUG-011: Multi-language detection with confidence
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
pub mod enhanced_naming_tests;
pub mod enhanced_python_visitor;
pub mod enhanced_reporting;
pub mod enhanced_typescript_visitor;
pub mod error_capture; // Issue #81: Bug report error capture
pub mod fault_localization; // Issue #103: Tarantula SBFL fault localization
pub mod file_classifier;
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod file_classifier_property_tests;
pub mod file_discovery;
pub mod file_health; // File Health Score: max-lines enforcement and TLR analysis
pub mod file_split; // File Split: semantic splitting via Louvain community detection
pub mod five_whys_analyzer;
pub mod fixed_graph_builder;
pub mod git_analysis;
#[cfg(feature = "git-lib")]
pub mod git_clone;
pub mod git_history; // GH-RAG: Git history RAG integration (separate index for commit messages)
pub mod git_test_filter; // Git-aware test filtering for targeted quality gates
#[cfg(feature = "http-client")]
pub mod github_integration;
pub mod hallucination_detector; // Sprint 37: Semantic entropy-based hallucination detection
pub mod hardcoded_paths; // Machine-specific absolute paths baked into source
pub mod incremental_churn;
pub mod incremental_coverage_analyzer;
pub mod infra_score; // Infra Score: CI/CD infrastructure quality scoring (PMAT-150)
pub mod language_override; // BUG-012: CLI language override support
pub mod lightweight_provability_analyzer;
pub mod makefile_compressor;
pub mod makefile_linter;
pub mod mermaid_generator;
pub mod ml_seed; // ML Reproducibility: Seed management for deterministic operations
pub mod normalized_score; // PMAT-454: Universal 0-100 score normalization
pub mod parallel_git;
pub mod parsed_file_cache;
pub mod path_glob; // R22-2 / D102: Shared glob expansion for MCP dispatchers
pub mod pdmt_quality_integration;
pub mod pdmt_service;
pub mod popper_score; // Popper Falsifiability Score v1.1 (Popperian science standards)
pub mod progress;
pub mod project_analyzer;
pub mod project_meta_detector;
pub mod proof_annotator;
pub mod quality_gates;
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
pub mod real_world_enhanced_naming_test;
pub mod repo_score; // Repository health scoring system
pub mod rust_project_score; // Rust Project Score v1.1 (evidence-based quality scoring)
pub mod similarity; // Advanced similarity and entropy detection
pub use quality_gates as quality_gate;
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod deep_context_property_tests;
#[cfg(feature = "deep-wasm")]
pub mod deep_wasm;
pub mod local_semantic; // Local semantic analysis using aprender (no external API required)
pub mod ml_quality_scorer; // GH-97: ML-based quality scoring using aprender
pub mod polyglot_analyzer;
pub mod quality_proxy;
pub mod ranking;
pub mod ranking_utils;
pub mod reachability;
pub mod readme_compressor;
pub mod recommendation_engine;
pub mod refactor_engine;
pub mod renderer;
pub mod rust_borrow_checker;
#[cfg(feature = "deep-wasm")]
pub mod rust_wasm_analyzer;
pub mod satd_detector;
pub mod semantic; // PMAT-SEARCH-001: Semantic code search services
pub mod semantic_naming;
pub mod simple_deep_context;
pub mod source_line_index; // Measured function line spans (#652/#656/#686)
pub mod spec_falsification; // RAG-powered Popperian falsification for specs
pub mod spec_parser; // Part C: Specification parsing for pmat qa command
pub mod symbol_table;
pub mod tdg_calculator;
pub mod template_service;
pub mod unified_ast_engine; // Stub for backward compatibility
#[cfg(feature = "shell-ast")]
pub mod unified_bash_analyzer; // TICKET-3006: Single-pass Bash/Shell analyzer
#[cfg(feature = "go-ast")]
pub mod unified_go_analyzer; // TICKET-3004: Single-pass Go analyzer (requires go-ast feature)
#[cfg(feature = "python-ast")]
pub mod unified_python_analyzer; // TICKET-3003: Single-pass Python analyzer
pub mod unified_refactor_analyzer; // Stub for backward compatibility
pub mod unified_rust_analyzer; // TICKET-3001: Single-pass AST+Complexity analyzer
pub mod unified_typescript_analyzer; // TICKET-3002: Single-pass TypeScript/JavaScript analyzer
#[cfg(feature = "wasm-ast")]
pub mod unified_wasm_analyzer; // TICKET-3005: Single-pass WebAssembly analyzer (requires wasm-ast feature)
pub mod vacuous_tests; // #[test] fns that cannot fail
pub mod verified_complexity;
pub mod wasm;
pub mod workspace_init; // `pmat init`: agent workspace bootstrap (#1030/#1031/#1032)

// Mutation testing engine (Phase 1 foundation - Phase 2: Optional)
#[cfg(feature = "mutation-testing")]
pub mod mutation;

// PMAT Oracle - PDCA loop for automated quality improvement
pub mod oracle;

// PMAT-REPORT-V1: Universal Rich Reporting with Data Science and ASCII Visualization
pub mod rich_reporter;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod satd_property_tests;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod mcp_property_tests;

#[cfg(all(test, feature = "git-lib"))]
mod git_clone_property_tests;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod quality_proxy_property_tests;

pub mod agent_context;
pub mod changelog_manager;
pub mod gaming_detector; // PMAT Work Contract: Anti-gaming detection for coverage metrics
#[cfg(feature = "github-api")]
pub mod github_client; // Issue #75: GitHub API integration (optional - 255 transitive deps)
pub mod hook_manager; // Issue #75 Phase 6: Git hooks for workflow
pub mod roadmap_service;
pub mod telemetry_service; // Issue #75 Phase 7: CHANGELOG automation // PMAT-470: RAG-powered agent context with semantic code search

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    #[test]
    fn test_mod_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }
}
