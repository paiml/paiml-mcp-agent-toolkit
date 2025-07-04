pub mod artifact_writer;
pub mod ast_based_dependency_analyzer;
#[cfg(feature = "c-ast")]
pub mod ast_c;
#[cfg(feature = "c-ast")]
pub mod ast_c_dispatch;
#[cfg(feature = "cpp-ast")]
pub mod ast_cpp;
#[cfg(feature = "cpp-ast")]
pub mod ast_cpp_dispatch;
#[cfg(feature = "kotlin-ast")]
pub mod ast_kotlin;
#[cfg(feature = "python-ast")]
pub mod ast_python;
pub mod ast_rust;
pub mod ast_rust_unified;
pub mod ast_strategies;
#[cfg(feature = "typescript-ast")]
pub mod ast_typescript;
#[cfg(feature = "typescript-ast")]
pub mod ast_typescript_dispatch;
pub mod big_o_analyzer;
pub mod cache;
pub mod canonical_query;
pub mod code_intelligence;
pub mod complexity;
pub mod complexity_patterns;
pub mod context;
pub mod coupling_analyzer;
pub mod dag_builder;
pub mod dead_code_analyzer;
pub mod dead_code_prover;
pub mod deep_context;
pub mod defect_analyzer;
pub mod defect_analyzers;
pub mod defect_report_service;
// pub mod deep_context_orchestrator; // TRACKED: Fix AST node compatibility
pub mod defect_probability;
pub mod deterministic_mermaid_engine;
pub mod dogfooding_engine;
pub mod duplicate_detector;
pub mod embedded_templates;
pub mod enhanced_reporting;
pub mod file_classifier;
pub mod file_discovery;
pub mod fixed_graph_builder;
pub mod git_analysis;
pub mod git_clone;
pub mod github_integration;
pub mod incremental_churn;
pub mod incremental_coverage_analyzer;
pub mod lightweight_provability_analyzer;
pub mod makefile_compressor;
pub mod makefile_linter;
pub mod mermaid_generator;
pub mod parallel_git;
pub mod parsed_file_cache;
pub mod progress;
pub mod project_analyzer;
pub mod project_meta_detector;
pub mod proof_annotator;
pub mod quality_gates;
pub mod ranking;
pub mod ranking_utils;
pub mod readme_compressor;
pub mod refactor_engine;
pub mod renderer;
pub mod rust_borrow_checker;
pub mod satd_detector;
pub mod semantic_naming;
pub mod simple_deep_context;
pub mod symbol_table;
pub mod tdg_calculator;
pub mod template_service;
pub mod unified_ast_engine;
pub mod unified_ast_parser;
pub mod unified_refactor_analyzer;
pub mod verified_complexity;
pub mod wasm;

#[cfg(test)]
mod ast_rust_property_tests;

#[cfg(all(test, feature = "typescript-ast"))]
mod ast_typescript_property_tests;

#[cfg(test)]
mod satd_property_tests;

#[cfg(test)]
mod tests {
    #[test]
    fn test_mod_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }
}
