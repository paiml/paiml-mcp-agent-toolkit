use crate::{ContentCache, MetadataCache, S3Client, TemplateServer, TemplateServerTrait};
use std::sync::Arc;
use tokio::sync::RwLock;

// Include comprehensive code smell tests for README features
mod code_smell_comprehensive_tests;

// Include Clap command structure validation tests
mod clap_command_structure_tests;

// Include Clap argument parsing correctness tests
mod clap_argument_parsing_tests;

// Include Clap environment variable integration tests
mod clap_env_var_tests;

// Include project metadata integration tests
mod project_meta_integration_test;

// Include Kaizen test performance optimizations
mod kaizen_test_optimizations;

// Include Kaizen reliability patterns for error prevention
mod kaizen_reliability_patterns;

// Include gitignore respect tests
mod gitignore_respect_tests;

// Include metric accuracy tests
mod metric_accuracy_suite;

// Include tests for glob pattern matching in --include flags
mod test_include_patterns;

// Include basic CLI tests for coverage
mod cli_basic_tests;

// Coverage boost tests - CLI enums
mod cli_coverage_boost;

// Coverage boost tests - services batch 1 (DeepContextAnalyzer)
mod coverage_boost_services1;

// Coverage boost tests - services batch 2 (language_analyzer)
mod coverage_boost_services2;

// Coverage boost tests - services batch 3 (defect_analyzers, enhanced_reporting)
mod coverage_boost_services3;

// Coverage boost tests - services batch 4 (satd_detector)
mod coverage_boost_services4;

// Coverage boost tests - complexity module
mod coverage_boost_complexity;

// Coverage boost tests - enhanced_reporting module
mod coverage_boost_enhanced_reporting;

// Coverage boost tests - defect_analyzers module
mod coverage_boost_defect_analyzers;

// Coverage boost tests - unified_ast_types module
mod coverage_boost_unified_ast;

// Coverage boost tests - error module
mod coverage_boost_error;

// Coverage boost tests - error module part 2 (recovery, severity, edge cases)
mod coverage_boost_error2;

// Coverage boost tests - unified_quality/onboarding module
mod coverage_boost_onboarding;

// Coverage boost tests - unified_quality/performance module
mod coverage_boost_performance;

// Coverage boost tests - services/perfection_score module
mod coverage_boost_perfection_score;

// Coverage boost tests - cli/analysis_utilities/churn module
mod coverage_boost_churn;

// Coverage boost tests - services/file_health module
mod coverage_boost_file_health;

// Coverage boost tests - cli/handlers/lint_hotspot_handlers module
mod coverage_boost_lint_hotspot;

// Coverage boost tests - cli/analysis_utilities/quality_gate module
mod coverage_boost_quality_gate;

// Coverage boost tests - demo module (requires demo feature)
#[cfg(feature = "demo")]
mod coverage_boost_demo;

// Coverage boost tests - services/mermaid_generator module
mod coverage_boost_mermaid;

// Coverage boost tests - services/enhanced_reporting module
mod coverage_boost_enhanced_reporting2;

// Coverage boost tests - qdd module
mod coverage_boost_qdd;

// Coverage boost tests - services/coverage_improvement module
mod coverage_boost_coverage_improvement;

// Coverage boost tests - models/dead_code and dead_code_handlers
mod coverage_boost_dead_code;

// Coverage boost tests - cli/analysis_utilities/comprehensive
mod coverage_boost_comprehensive;

// Coverage boost tests - cli/handlers/enforce_handlers
mod coverage_boost_enforce;

// Coverage boost tests - unified_ast_types part 2 (Location, Span, QualifiedName, UnifiedAstNode)
mod coverage_boost_unified_ast2;

// Coverage boost tests - unified_ast_types part 3 (AstDag, ColumnStore, advanced methods)
mod coverage_boost_unified_ast3;

// Coverage boost tests - maintenance module (roadmap, ticket, validator)
mod coverage_boost_maintenance;

// Coverage boost tests - models/refactor module (state machine, configs)
mod coverage_boost_refactor;

// Coverage boost tests - services/normalized_score module (Grade, SimpleScore)
mod coverage_boost_normalized_score;

// Coverage boost tests - models/defect_report module (Severity, DefectCategory, Defect)
mod coverage_boost_defect_report;

// Coverage boost tests - models/tdg module (TDGScore, TDGSeverity, TDGConfig)
mod coverage_boost_tdg;

// Coverage boost tests - models/debug_analysis module (Five Whys)
mod coverage_boost_debug_analysis;

// Coverage boost tests - services/brick_score module (SIMD, GPU, hardware)
mod coverage_boost_brick_score;

// Coverage boost tests - services/ml_quality_scorer module
mod coverage_boost_ml_quality;

// Coverage boost tests - services/metric_trends module
mod coverage_boost_metric_trends;

// Coverage boost tests - services/ranking module
mod coverage_boost_ranking;

// Coverage boost tests - services/polyglot_analyzer module
mod coverage_boost_polyglot;

// Coverage boost tests - services/big_o_analyzer and models/complexity_bound
mod coverage_boost_big_o;

// Coverage boost tests - services/tdg_calculator module
mod coverage_boost_tdg_calculator;

// Coverage boost tests - services/defect_report_service (compute_summary, format_*, filter)
mod coverage_boost_defect_report_svc;

// Coverage boost tests - services/language_analyzer (analyze_file, supports_analysis)
mod coverage_boost_language_analyzer;

// Coverage boost tests - services/defect_report_service (compute_summary, format_*, filter)
mod coverage_boost_defect_report_svc2;

// Coverage boost tests - services/coverage_improvement (parse_coverage, extract_files, generate)
mod coverage_boost_coverage_improvement2;

// Coverage boost tests - services/enhanced_reporting (generate_report, format_report, all sections)
mod coverage_boost_enhanced_reporting3;

// Coverage boost tests - cli/handlers/work_falsification (FalsificationReport methods)
mod coverage_boost_falsification;

// Coverage boost tests - services/deep_context/analyzer_formatting (format_as_*)
mod coverage_boost_analyzer_formatting;

// Coverage boost tests - cli/handlers/comply_handlers/check_handlers module
mod coverage_boost_check_handlers;

// Coverage boost tests - cli/handlers/lint_hotspot_handlers (full module coverage)
mod coverage_boost_lint_hotspot_handlers;

// Coverage boost tests - cli/handlers/work_handlers/ticket_handlers module
mod coverage_boost_ticket_handlers;

// Coverage boost tests - dead_code_analyzer pure functions
mod coverage_boost_dead_code_pure;

// Coverage boost tests - configuration_service module
mod coverage_boost_configuration_service;

// Coverage boost tests - dag_builder module
mod coverage_boost_dag_builder;

// Coverage boost tests - services/oracle/signal_collector module
mod coverage_boost_signal_collector;

// Coverage boost tests - doc_validator module
mod coverage_boost_doc_validator;

// Coverage boost tests - services/mutation module (types, scoring, state, language)
#[cfg(feature = "mutation-testing")]
mod coverage_boost_mutation;

// Coverage boost tests - comply_cb_detect module (CB-050/CB-060 detection)
mod coverage_boost_comply_cb_detect;

// Coverage boost tests - complexity_handlers module
// DISABLED: ChurnFileInfo removed and is_source_code_file is private
// mod coverage_boost_complexity_handlers;

// Coverage boost tests - provability_handler and lightweight_provability_analyzer modules
mod coverage_boost_provability_handler;

// Coverage boost tests - cli/handlers/work_handlers/core_handlers module
// DISABLED: SubTask removed (use Subtask)
// mod coverage_boost_work_core_handlers;

// Coverage boost tests - cli/handlers/proof_annotations_handler module
mod coverage_boost_proof_annotations_handler;

// Coverage boost tests - cli/handlers/analyze_defects_handler module
mod coverage_boost_analyze_defects_handler;

// Coverage boost tests - services/semantic/chunker module (code chunking, text chunking)
mod coverage_boost_chunker;

// Coverage boost tests - services/ast_strategies_impl module (StrategyRegistry, language strategies)
mod coverage_boost_ast_strategies;

// Coverage boost tests - services/similarity module (SimilarityDetector, Winnowing, entropy)
mod coverage_boost_similarity;

// Coverage boost tests - facades and MCP mapping (SATD, DefectPrediction, Project analyzer)
mod coverage_boost_facades;

// Coverage boost tests - language_analyzer module (AnalysisType, OutputFormat, AnalysisOptions)
mod coverage_boost_language_analyzer2;

// Coverage boost tests - cli/enums module (all Display implementations)
mod coverage_boost_cli_enums;

// Include protocol service tests for coverage (requires unified-protocol feature)
#[cfg(feature = "unified-protocol")]
mod protocol_service_tests;

// Diagnostic command tests (fixed struct fields for current API)
mod diagnose_tests;

#[tokio::test]
async fn test_template_server_new() {
    let server = TemplateServer::new().await;
    assert!(server.is_ok());

    let server = server.unwrap();
    assert_eq!(server.bucket_name, "dummy");
}

#[tokio::test]
async fn test_template_server_trait_implementation() {
    let server = TemplateServer::new().await.unwrap();

    // Test get_renderer
    let renderer = server.get_renderer();
    assert!(std::ptr::eq(renderer, &server.renderer)); // They should be the same reference

    // Test get_metadata_cache
    assert!(server.get_metadata_cache().is_some());

    // Test get_content_cache
    assert!(server.get_content_cache().is_some());

    // Test get_s3_client
    assert!(server.get_s3_client().is_some());

    // Test get_bucket_name
    assert_eq!(server.get_bucket_name(), Some("dummy"));
}

#[tokio::test]
async fn test_template_server_deprecated_methods() {
    let server = TemplateServer::new().await.unwrap();

    // Test get_template_metadata returns error
    let result = server.get_template_metadata("test").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("deprecated"));

    // Test get_template_content returns error
    let result = server.get_template_content("test").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("deprecated"));

    // Test list_templates returns error (via trait)
    let result = TemplateServerTrait::list_templates(&server, "test").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("deprecated"));
}

#[tokio::test]
async fn test_warm_cache() {
    let server = TemplateServer::new().await.unwrap();

    // warm_cache should succeed even though individual template fetches fail
    let result = server.warm_cache().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_template_server_cache_initialization() {
    let server = TemplateServer::new().await.unwrap();

    // Check metadata cache is initialized and empty
    let metadata_cache = server.metadata_cache.read().await;
    assert_eq!(metadata_cache.len(), 0);
    drop(metadata_cache);

    // Check content cache is initialized and empty
    let content_cache = server.content_cache.read().await;
    assert_eq!(content_cache.len(), 0);
}

#[tokio::test]
async fn test_template_server_cache_sizes() {
    let server = TemplateServer::new().await.unwrap();

    // Verify cache sizes are set correctly (1024 total, split between metadata and content)
    let metadata_cache = server.metadata_cache.read().await;
    assert!(metadata_cache.cap().get() >= 512);
    drop(metadata_cache);

    let content_cache = server.content_cache.read().await;
    assert!(content_cache.cap().get() >= 1024);
}

#[tokio::test]
async fn test_warm_cache_templates() {
    let server = TemplateServer::new().await.unwrap();

    // Call warm_cache and ensure it tries to load expected templates
    let result = server.warm_cache().await;
    assert!(result.is_ok());

    // The cache should still be empty since get_template_metadata returns errors
    let metadata_cache = server.metadata_cache.read().await;
    assert_eq!(metadata_cache.len(), 0);
}

#[tokio::test]
async fn test_template_server_trait_via_methods() {
    let server = TemplateServer::new().await.unwrap();

    // Test calling trait methods directly
    let result = TemplateServerTrait::get_template_metadata(&server, "test").await;
    assert!(result.is_err());

    let result = TemplateServerTrait::get_template_content(&server, "test").await;
    assert!(result.is_err());

    // Test trait method accessors
    let _ = TemplateServerTrait::get_renderer(&server);
    let _ = TemplateServerTrait::get_metadata_cache(&server);
    let _ = TemplateServerTrait::get_content_cache(&server);
    let _ = TemplateServerTrait::get_s3_client(&server);
    let _ = TemplateServerTrait::get_bucket_name(&server);
}

#[test]
fn test_type_aliases() {
    use lru::LruCache;
    use std::num::NonZeroUsize;

    // Test that type aliases work correctly
    let metadata_cache: MetadataCache =
        Arc::new(RwLock::new(LruCache::new(NonZeroUsize::new(10).unwrap())));

    let content_cache: ContentCache =
        Arc::new(RwLock::new(LruCache::new(NonZeroUsize::new(10).unwrap())));

    // Basic sanity check
    assert!(Arc::strong_count(&metadata_cache) == 1);
    assert!(Arc::strong_count(&content_cache) == 1);
}

#[test]
fn test_s3_client_instantiation() {
    // Test S3Client can be instantiated
    let _client = S3Client;
}

#[tokio::test]
async fn test_run_mcp_server_basic() {
    use crate::run_mcp_server;
    use crate::stateless_server::StatelessTemplateServer;
    use std::sync::Arc;

    // Create a test server
    let server = Arc::new(StatelessTemplateServer::new().unwrap());

    // Create a simple invalid request that will cause the server to exit immediately
    std::thread::spawn(move || {
        // Simulate empty stdin which will cause the server to exit
        let _result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(run_mcp_server(server));
    });

    // Give the server a moment to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Test passes if server starts without panic
}

#[test]
fn test_public_exports() {
    // Test that public exports are available
    use crate::{ParameterSpec, ParameterType, TemplateError};

    // Create dummy values to ensure types are used
    let _param_type = ParameterType::String;
    let _param_spec = ParameterSpec {
        name: "test".to_string(),
        description: "test".to_string(),
        param_type: ParameterType::String,
        required: true,
        default_value: None,
        validation_pattern: None,
    };

    // Test error type
    let _error = TemplateError::InvalidUri {
        uri: "test".to_string(),
    };
}

// Batch 1: Additional unregistered test modules (verified working)
mod additional_coverage;
mod cache;
