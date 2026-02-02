//! Coverage boost tests for cli/enums.rs
//! Tests all Display implementations and as_str methods

use crate::cli::enums::*;

// ============ EnforceOutputFormat Tests ============

#[test]
fn test_enforce_output_format_display() {
    assert_eq!(EnforceOutputFormat::Summary.to_string(), "summary");
    assert_eq!(EnforceOutputFormat::Json.to_string(), "json");
    assert_eq!(EnforceOutputFormat::Progress.to_string(), "progress");
    assert_eq!(EnforceOutputFormat::Sarif.to_string(), "sarif");
}

#[test]
fn test_enforce_output_format_clone() {
    let f = EnforceOutputFormat::Json;
    let cloned = f;
    assert_eq!(cloned.to_string(), "json");
}

// ============ RefactorAutoOutputFormat Tests ============

#[test]
fn test_refactor_auto_output_format_display() {
    assert_eq!(RefactorAutoOutputFormat::Summary.to_string(), "summary");
    assert_eq!(RefactorAutoOutputFormat::Detailed.to_string(), "detailed");
    assert_eq!(RefactorAutoOutputFormat::Json.to_string(), "json");
}

// ============ RefactorDocsOutputFormat Tests ============

#[test]
fn test_refactor_docs_output_format_display() {
    assert_eq!(RefactorDocsOutputFormat::Summary.to_string(), "summary");
    assert_eq!(RefactorDocsOutputFormat::Detailed.to_string(), "detailed");
    assert_eq!(RefactorDocsOutputFormat::Json.to_string(), "json");
    assert_eq!(RefactorDocsOutputFormat::Interactive.to_string(), "interactive");
}

// ============ QualityProfile Tests ============

#[test]
fn test_quality_profile_default() {
    let profile = QualityProfile::default();
    assert_eq!(profile, QualityProfile::Extreme);
}

#[test]
fn test_quality_profile_display() {
    assert_eq!(QualityProfile::Standard.to_string(), "standard");
    assert_eq!(QualityProfile::Strict.to_string(), "strict");
    assert_eq!(QualityProfile::Extreme.to_string(), "extreme");
}

// ============ ContextFormat Tests ============

#[test]
fn test_context_format_display() {
    assert_eq!(ContextFormat::Markdown.to_string(), "markdown");
    assert_eq!(ContextFormat::Json.to_string(), "json");
    assert_eq!(ContextFormat::Sarif.to_string(), "sarif");
    assert_eq!(ContextFormat::LlmOptimized.to_string(), "llm-optimized");
}

// ============ LintHotspotOutputFormat Tests ============

#[test]
fn test_lint_hotspot_output_format_display() {
    assert_eq!(LintHotspotOutputFormat::Summary.to_string(), "summary");
    assert_eq!(LintHotspotOutputFormat::Detailed.to_string(), "detailed");
    assert_eq!(LintHotspotOutputFormat::Json.to_string(), "json");
    assert_eq!(LintHotspotOutputFormat::EnforcementJson.to_string(), "enforcement-json");
    assert_eq!(LintHotspotOutputFormat::Sarif.to_string(), "sarif");
}

// ============ ProvabilityOutputFormat Tests ============

#[test]
fn test_provability_output_format_display() {
    assert_eq!(ProvabilityOutputFormat::Summary.to_string(), "summary");
    assert_eq!(ProvabilityOutputFormat::Full.to_string(), "full");
    assert_eq!(ProvabilityOutputFormat::Json.to_string(), "json");
    assert_eq!(ProvabilityOutputFormat::Sarif.to_string(), "sarif");
    assert_eq!(ProvabilityOutputFormat::Markdown.to_string(), "markdown");
}

// ============ DuplicateType Tests ============

#[test]
fn test_duplicate_type_display() {
    assert_eq!(DuplicateType::Exact.to_string(), "exact");
    assert_eq!(DuplicateType::Renamed.to_string(), "renamed");
    assert_eq!(DuplicateType::Gapped.to_string(), "gapped");
    assert_eq!(DuplicateType::Semantic.to_string(), "semantic");
    assert_eq!(DuplicateType::Fuzzy.to_string(), "fuzzy");
    assert_eq!(DuplicateType::All.to_string(), "all");
}

// ============ DefectPredictionOutputFormat Tests ============

#[test]
fn test_defect_prediction_output_format_display() {
    assert_eq!(DefectPredictionOutputFormat::Summary.to_string(), "summary");
    assert_eq!(DefectPredictionOutputFormat::Detailed.to_string(), "detailed");
    assert_eq!(DefectPredictionOutputFormat::Json.to_string(), "json");
    assert_eq!(DefectPredictionOutputFormat::Csv.to_string(), "csv");
    assert_eq!(DefectPredictionOutputFormat::Sarif.to_string(), "sarif");
}

// ============ ComprehensiveOutputFormat Tests ============

#[test]
fn test_comprehensive_output_format_display() {
    assert_eq!(ComprehensiveOutputFormat::Summary.to_string(), "summary");
    assert_eq!(ComprehensiveOutputFormat::Detailed.to_string(), "detailed");
    assert_eq!(ComprehensiveOutputFormat::Json.to_string(), "json");
    assert_eq!(ComprehensiveOutputFormat::Markdown.to_string(), "markdown");
    assert_eq!(ComprehensiveOutputFormat::Sarif.to_string(), "sarif");
}

// ============ GraphMetricType Tests ============

#[test]
fn test_graph_metric_type_display() {
    assert_eq!(GraphMetricType::Centrality.to_string(), "centrality");
    assert_eq!(GraphMetricType::Betweenness.to_string(), "betweenness");
    assert_eq!(GraphMetricType::Closeness.to_string(), "closeness");
    assert_eq!(GraphMetricType::PageRank.to_string(), "pagerank");
    assert_eq!(GraphMetricType::Clustering.to_string(), "clustering");
    assert_eq!(GraphMetricType::Components.to_string(), "components");
    assert_eq!(GraphMetricType::All.to_string(), "all");
}

// ============ GraphMetricsOutputFormat Tests ============

#[test]
fn test_graph_metrics_output_format_display() {
    assert_eq!(GraphMetricsOutputFormat::Summary.to_string(), "summary");
    assert_eq!(GraphMetricsOutputFormat::Detailed.to_string(), "detailed");
    assert_eq!(GraphMetricsOutputFormat::Human.to_string(), "human");
    assert_eq!(GraphMetricsOutputFormat::Json.to_string(), "json");
    assert_eq!(GraphMetricsOutputFormat::Csv.to_string(), "csv");
    assert_eq!(GraphMetricsOutputFormat::GraphML.to_string(), "graphml");
    assert_eq!(GraphMetricsOutputFormat::Markdown.to_string(), "markdown");
}

// ============ NameSimilarityOutputFormat Tests ============

#[test]
fn test_name_similarity_output_format_display() {
    assert_eq!(NameSimilarityOutputFormat::Summary.to_string(), "summary");
    assert_eq!(NameSimilarityOutputFormat::Detailed.to_string(), "detailed");
    assert_eq!(NameSimilarityOutputFormat::Human.to_string(), "human");
    assert_eq!(NameSimilarityOutputFormat::Json.to_string(), "json");
    assert_eq!(NameSimilarityOutputFormat::Csv.to_string(), "csv");
    assert_eq!(NameSimilarityOutputFormat::Markdown.to_string(), "markdown");
}

// ============ DuplicateOutputFormat Tests ============

#[test]
fn test_duplicate_output_format_display() {
    assert_eq!(DuplicateOutputFormat::Summary.to_string(), "summary");
    assert_eq!(DuplicateOutputFormat::Detailed.to_string(), "detailed");
    assert_eq!(DuplicateOutputFormat::Human.to_string(), "human");
    assert_eq!(DuplicateOutputFormat::Json.to_string(), "json");
    assert_eq!(DuplicateOutputFormat::Csv.to_string(), "csv");
    assert_eq!(DuplicateOutputFormat::Sarif.to_string(), "sarif");
}

// ============ DeepContextDagType Tests ============

#[test]
fn test_deep_context_dag_type_display() {
    assert_eq!(DeepContextDagType::CallGraph.to_string(), "call-graph");
    assert_eq!(DeepContextDagType::ImportGraph.to_string(), "import-graph");
    assert_eq!(DeepContextDagType::Inheritance.to_string(), "inheritance");
    assert_eq!(DeepContextDagType::FullDependency.to_string(), "full-dependency");
}

// ============ DeepContextCacheStrategy Tests ============

#[test]
fn test_deep_context_cache_strategy_display() {
    assert_eq!(DeepContextCacheStrategy::Normal.to_string(), "normal");
    assert_eq!(DeepContextCacheStrategy::ForceRefresh.to_string(), "force-refresh");
    assert_eq!(DeepContextCacheStrategy::Offline.to_string(), "offline");
}

// ============ ProofAnnotationOutputFormat Tests ============

#[test]
fn test_proof_annotation_output_format_display() {
    assert_eq!(ProofAnnotationOutputFormat::Summary.to_string(), "summary");
    assert_eq!(ProofAnnotationOutputFormat::Full.to_string(), "full");
    assert_eq!(ProofAnnotationOutputFormat::Json.to_string(), "json");
    assert_eq!(ProofAnnotationOutputFormat::Markdown.to_string(), "markdown");
    assert_eq!(ProofAnnotationOutputFormat::Sarif.to_string(), "sarif");
}

// ============ PropertyTypeFilter Tests ============

#[test]
fn test_property_type_filter_display() {
    assert_eq!(PropertyTypeFilter::MemorySafety.to_string(), "memory-safety");
    assert_eq!(PropertyTypeFilter::ThreadSafety.to_string(), "thread-safety");
    assert_eq!(PropertyTypeFilter::DataRaceFreeze.to_string(), "data-race-freeze");
    assert_eq!(PropertyTypeFilter::Termination.to_string(), "termination");
    assert_eq!(PropertyTypeFilter::FunctionalCorrectness.to_string(), "functional-correctness");
    assert_eq!(PropertyTypeFilter::ResourceBounds.to_string(), "resource-bounds");
    assert_eq!(PropertyTypeFilter::All.to_string(), "all");
}

// ============ VerificationMethodFilter Tests ============

#[test]
fn test_verification_method_filter_display() {
    assert_eq!(VerificationMethodFilter::FormalProof.to_string(), "formal-proof");
    assert_eq!(VerificationMethodFilter::ModelChecking.to_string(), "model-checking");
    assert_eq!(VerificationMethodFilter::StaticAnalysis.to_string(), "static-analysis");
    assert_eq!(VerificationMethodFilter::AbstractInterpretation.to_string(), "abstract-interpretation");
    assert_eq!(VerificationMethodFilter::BorrowChecker.to_string(), "borrow-checker");
    assert_eq!(VerificationMethodFilter::All.to_string(), "all");
}

// ============ IncrementalCoverageOutputFormat Tests ============

#[test]
fn test_incremental_coverage_output_format_display() {
    assert_eq!(IncrementalCoverageOutputFormat::Summary.to_string(), "summary");
    assert_eq!(IncrementalCoverageOutputFormat::Detailed.to_string(), "detailed");
    assert_eq!(IncrementalCoverageOutputFormat::Json.to_string(), "json");
    assert_eq!(IncrementalCoverageOutputFormat::Markdown.to_string(), "markdown");
    assert_eq!(IncrementalCoverageOutputFormat::Lcov.to_string(), "lcov");
    assert_eq!(IncrementalCoverageOutputFormat::Delta.to_string(), "delta");
    assert_eq!(IncrementalCoverageOutputFormat::Sarif.to_string(), "sarif");
}

// ============ QualityGateOutputFormat Tests ============

#[test]
fn test_quality_gate_output_format_display() {
    assert_eq!(QualityGateOutputFormat::Summary.to_string(), "summary");
    assert_eq!(QualityGateOutputFormat::Detailed.to_string(), "detailed");
    assert_eq!(QualityGateOutputFormat::Human.to_string(), "human");
    assert_eq!(QualityGateOutputFormat::Json.to_string(), "json");
    assert_eq!(QualityGateOutputFormat::Junit.to_string(), "junit");
    assert_eq!(QualityGateOutputFormat::Markdown.to_string(), "markdown");
}

// ============ ReportOutputFormat Tests ============

#[test]
fn test_report_output_format_display() {
    assert_eq!(ReportOutputFormat::Json.to_string(), "json");
    assert_eq!(ReportOutputFormat::Csv.to_string(), "csv");
    assert_eq!(ReportOutputFormat::Markdown.to_string(), "markdown");
    assert_eq!(ReportOutputFormat::Text.to_string(), "text");
    assert_eq!(ReportOutputFormat::Html.to_string(), "html");
    assert_eq!(ReportOutputFormat::Pdf.to_string(), "pdf");
    assert_eq!(ReportOutputFormat::Dashboard.to_string(), "dashboard");
}

// ============ RepoScoreOutputFormat Tests ============

#[test]
fn test_repo_score_output_format_display() {
    assert_eq!(RepoScoreOutputFormat::Text.to_string(), "text");
    assert_eq!(RepoScoreOutputFormat::Json.to_string(), "json");
    assert_eq!(RepoScoreOutputFormat::Markdown.to_string(), "markdown");
    assert_eq!(RepoScoreOutputFormat::Yaml.to_string(), "yaml");
}

// ============ QualityCheckType Tests ============

#[test]
fn test_quality_check_type_display() {
    assert_eq!(QualityCheckType::DeadCode.to_string(), "dead-code");
    assert_eq!(QualityCheckType::Complexity.to_string(), "complexity");
    assert_eq!(QualityCheckType::Coverage.to_string(), "coverage");
    assert_eq!(QualityCheckType::Sections.to_string(), "sections");
    assert_eq!(QualityCheckType::Provability.to_string(), "provability");
    assert_eq!(QualityCheckType::Satd.to_string(), "satd");
    assert_eq!(QualityCheckType::Entropy.to_string(), "entropy");
    assert_eq!(QualityCheckType::Security.to_string(), "security");
    assert_eq!(QualityCheckType::Duplicates.to_string(), "duplicates");
    assert_eq!(QualityCheckType::All.to_string(), "all");
}

#[test]
fn test_quality_check_type_default_checks() {
    let defaults = QualityCheckType::default_checks();
    assert!(defaults.contains(&QualityCheckType::Complexity));
    assert!(defaults.contains(&QualityCheckType::DeadCode));
    assert!(defaults.contains(&QualityCheckType::Satd));
    assert!(defaults.contains(&QualityCheckType::Entropy));
    assert!(defaults.contains(&QualityCheckType::Security));
    assert!(defaults.contains(&QualityCheckType::Duplicates));
    assert!(defaults.contains(&QualityCheckType::Coverage));
    assert!(defaults.contains(&QualityCheckType::Sections));
    assert!(defaults.contains(&QualityCheckType::Provability));
    assert!(!defaults.contains(&QualityCheckType::All));
}

// ============ EntropyOutputFormat Tests ============

#[test]
fn test_entropy_output_format_display() {
    assert_eq!(EntropyOutputFormat::Summary.to_string(), "summary");
    assert_eq!(EntropyOutputFormat::Detailed.to_string(), "detailed");
    assert_eq!(EntropyOutputFormat::Json.to_string(), "json");
    assert_eq!(EntropyOutputFormat::Markdown.to_string(), "markdown");
}

// ============ EntropySeverity Tests ============

#[test]
fn test_entropy_severity_display() {
    assert_eq!(EntropySeverity::Low.to_string(), "low");
    assert_eq!(EntropySeverity::Medium.to_string(), "medium");
    assert_eq!(EntropySeverity::High.to_string(), "high");
}

// ============ WasmOutputFormat Tests ============

#[test]
fn test_wasm_output_format_display() {
    assert_eq!(WasmOutputFormat::Summary.to_string(), "summary");
    assert_eq!(WasmOutputFormat::Detailed.to_string(), "detailed");
    assert_eq!(WasmOutputFormat::Json.to_string(), "json");
    assert_eq!(WasmOutputFormat::Sarif.to_string(), "sarif");
}

// ============ DeepWasmLanguage Tests ============

#[test]
fn test_deep_wasm_language_display() {
    assert_eq!(DeepWasmLanguage::Rust.to_string(), "rust");
    assert_eq!(DeepWasmLanguage::Ruchy.to_string(), "ruchy");
}

// ============ DeepWasmFocus Tests ============

#[test]
fn test_deep_wasm_focus_display() {
    assert_eq!(DeepWasmFocus::Full.to_string(), "full");
    assert_eq!(DeepWasmFocus::Source.to_string(), "source");
    assert_eq!(DeepWasmFocus::Compilation.to_string(), "compilation");
    assert_eq!(DeepWasmFocus::Runtime.to_string(), "runtime");
    assert_eq!(DeepWasmFocus::Interop.to_string(), "interop");
}

// ============ DeepWasmOutputFormat Tests ============

#[test]
fn test_deep_wasm_output_format_display() {
    assert_eq!(DeepWasmOutputFormat::Markdown.to_string(), "markdown");
    assert_eq!(DeepWasmOutputFormat::Json.to_string(), "json");
    assert_eq!(DeepWasmOutputFormat::Html.to_string(), "html");
}

// ============ DebugOutputFormat Tests ============

#[test]
fn test_debug_output_format_display() {
    assert_eq!(DebugOutputFormat::Text.to_string(), "text");
    assert_eq!(DebugOutputFormat::Json.to_string(), "json");
    assert_eq!(DebugOutputFormat::Markdown.to_string(), "markdown");
}

// ============ PromptOutputFormat Tests ============

#[test]
fn test_prompt_output_format_display() {
    assert_eq!(PromptOutputFormat::Yaml.to_string(), "yaml");
    assert_eq!(PromptOutputFormat::Json.to_string(), "json");
    assert_eq!(PromptOutputFormat::Text.to_string(), "text");
}

// ============ DefectsOutputFormat Tests ============

#[test]
fn test_defects_output_format_display() {
    assert_eq!(DefectsOutputFormat::Text.to_string(), "text");
    assert_eq!(DefectsOutputFormat::Json.to_string(), "json");
    assert_eq!(DefectsOutputFormat::Junit.to_string(), "junit");
}

// ============ Clone and Equality Tests ============

#[test]
fn test_enum_clone_and_equality() {
    let f1 = EnforceOutputFormat::Json;
    let f2 = f1;
    assert_eq!(f1, f2);

    let p1 = QualityProfile::Strict;
    let p2 = p1;
    assert_eq!(p1, p2);

    let d1 = DuplicateType::Semantic;
    let d2 = d1.clone();
    assert_eq!(d1, d2);
}

#[test]
fn test_enum_debug() {
    assert!(format!("{:?}", EnforceOutputFormat::Json).contains("Json"));
    assert!(format!("{:?}", QualityProfile::Extreme).contains("Extreme"));
    assert!(format!("{:?}", DuplicateType::Fuzzy).contains("Fuzzy"));
    assert!(format!("{:?}", GraphMetricType::PageRank).contains("PageRank"));
}
