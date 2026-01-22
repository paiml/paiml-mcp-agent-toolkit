//! CLI Format Conversion Helper Functions
//!
//! Extracted from cli.rs for file health compliance (CB-040).
//! Contains format conversion helpers used by the CLI adapter.

use crate::cli::ContextFormat;
use crate::models::churn::ChurnOutputFormat;

// Format conversion helper functions

pub fn format_to_string(format: &ContextFormat) -> String {
    match format {
        ContextFormat::Markdown => "markdown".to_string(),
        ContextFormat::Json => "json".to_string(),
        ContextFormat::Sarif => "sarif".to_string(),
        ContextFormat::LlmOptimized => "llm-optimized".to_string(),
    }
}

pub fn churn_format_to_string(format: &ChurnOutputFormat) -> String {
    match format {
        ChurnOutputFormat::Summary => "summary".to_string(),
        ChurnOutputFormat::Markdown => "markdown".to_string(),
        ChurnOutputFormat::Json => "json".to_string(),
        ChurnOutputFormat::Csv => "csv".to_string(),
    }
}

pub fn complexity_format_to_string(format: &crate::cli::ComplexityOutputFormat) -> String {
    match format {
        crate::cli::ComplexityOutputFormat::Summary => "summary".to_string(),
        crate::cli::ComplexityOutputFormat::Full => "full".to_string(),
        crate::cli::ComplexityOutputFormat::Json => "json".to_string(),
        crate::cli::ComplexityOutputFormat::Sarif => "sarif".to_string(),
    }
}

pub fn dag_type_to_string(dag_type: &crate::cli::DagType) -> String {
    match dag_type {
        crate::cli::DagType::CallGraph => "call-graph".to_string(),
        crate::cli::DagType::ImportGraph => "import-graph".to_string(),
        crate::cli::DagType::Inheritance => "inheritance".to_string(),
        crate::cli::DagType::FullDependency => "full-dependency".to_string(),
    }
}

pub fn dead_code_format_to_string(format: &crate::cli::DeadCodeOutputFormat) -> String {
    match format {
        crate::cli::DeadCodeOutputFormat::Summary => "summary".to_string(),
        crate::cli::DeadCodeOutputFormat::Json => "json".to_string(),
        crate::cli::DeadCodeOutputFormat::Sarif => "sarif".to_string(),
        crate::cli::DeadCodeOutputFormat::Markdown => "markdown".to_string(),
    }
}

pub fn satd_format_to_string(format: &crate::cli::SatdOutputFormat) -> String {
    match format {
        crate::cli::SatdOutputFormat::Summary => "summary".to_string(),
        crate::cli::SatdOutputFormat::Json => "json".to_string(),
        crate::cli::SatdOutputFormat::Sarif => "sarif".to_string(),
        crate::cli::SatdOutputFormat::Markdown => "markdown".to_string(),
    }
}

pub fn satd_severity_to_string(severity: &crate::cli::SatdSeverity) -> String {
    match severity {
        crate::cli::SatdSeverity::Critical => "critical".to_string(),
        crate::cli::SatdSeverity::High => "high".to_string(),
        crate::cli::SatdSeverity::Medium => "medium".to_string(),
        crate::cli::SatdSeverity::Low => "low".to_string(),
    }
}

pub fn graph_metric_type_to_string(metric: &crate::cli::GraphMetricType) -> String {
    match metric {
        crate::cli::GraphMetricType::All => "all".to_string(),
        crate::cli::GraphMetricType::Centrality => "centrality".to_string(),
        crate::cli::GraphMetricType::Betweenness => "betweenness".to_string(),
        crate::cli::GraphMetricType::Closeness => "closeness".to_string(),
        crate::cli::GraphMetricType::PageRank => "pagerank".to_string(),
        crate::cli::GraphMetricType::Clustering => "clustering".to_string(),
        crate::cli::GraphMetricType::Components => "components".to_string(),
    }
}

pub fn graph_metrics_format_to_string(format: &crate::cli::GraphMetricsOutputFormat) -> String {
    match format {
        crate::cli::GraphMetricsOutputFormat::Summary => "summary".to_string(),
        crate::cli::GraphMetricsOutputFormat::Detailed => "detailed".to_string(),
        crate::cli::GraphMetricsOutputFormat::Human => "human".to_string(),
        crate::cli::GraphMetricsOutputFormat::Json => "json".to_string(),
        crate::cli::GraphMetricsOutputFormat::Csv => "csv".to_string(),
        crate::cli::GraphMetricsOutputFormat::GraphML => "graphml".to_string(),
        crate::cli::GraphMetricsOutputFormat::Markdown => "markdown".to_string(),
    }
}

pub fn name_similarity_format_to_string(format: &crate::cli::NameSimilarityOutputFormat) -> String {
    match format {
        crate::cli::NameSimilarityOutputFormat::Summary => "summary".to_string(),
        crate::cli::NameSimilarityOutputFormat::Detailed => "detailed".to_string(),
        crate::cli::NameSimilarityOutputFormat::Human => "human".to_string(),
        crate::cli::NameSimilarityOutputFormat::Json => "json".to_string(),
        crate::cli::NameSimilarityOutputFormat::Csv => "csv".to_string(),
        crate::cli::NameSimilarityOutputFormat::Markdown => "markdown".to_string(),
    }
}

pub fn property_type_filter_to_string(filter: &crate::cli::PropertyTypeFilter) -> String {
    match filter {
        crate::cli::PropertyTypeFilter::All => "all".to_string(),
        crate::cli::PropertyTypeFilter::MemorySafety => "memory-safety".to_string(),
        crate::cli::PropertyTypeFilter::ThreadSafety => "thread-safety".to_string(),
        crate::cli::PropertyTypeFilter::DataRaceFreeze => "data-race-freeze".to_string(),
        crate::cli::PropertyTypeFilter::Termination => "termination".to_string(),
        crate::cli::PropertyTypeFilter::FunctionalCorrectness => {
            "functional-correctness".to_string()
        }
        crate::cli::PropertyTypeFilter::ResourceBounds => "resource-bounds".to_string(),
    }
}

pub fn verification_method_filter_to_string(method: &crate::cli::VerificationMethodFilter) -> String {
    match method {
        crate::cli::VerificationMethodFilter::All => "all".to_string(),
        crate::cli::VerificationMethodFilter::FormalProof => "formal-proof".to_string(),
        crate::cli::VerificationMethodFilter::ModelChecking => "model-checking".to_string(),
        crate::cli::VerificationMethodFilter::StaticAnalysis => "static-analysis".to_string(),
        crate::cli::VerificationMethodFilter::AbstractInterpretation => {
            "abstract-interpretation".to_string()
        }
        crate::cli::VerificationMethodFilter::BorrowChecker => "borrow-checker".to_string(),
    }
}

pub fn proof_annotation_format_to_string(format: &crate::cli::ProofAnnotationOutputFormat) -> String {
    match format {
        crate::cli::ProofAnnotationOutputFormat::Summary => "summary".to_string(),
        crate::cli::ProofAnnotationOutputFormat::Full => "full".to_string(),
        crate::cli::ProofAnnotationOutputFormat::Json => "json".to_string(),
        crate::cli::ProofAnnotationOutputFormat::Markdown => "markdown".to_string(),
        crate::cli::ProofAnnotationOutputFormat::Sarif => "sarif".to_string(),
    }
}

pub fn incremental_coverage_format_to_string(
    format: &crate::cli::IncrementalCoverageOutputFormat,
) -> String {
    match format {
        crate::cli::IncrementalCoverageOutputFormat::Summary => "summary".to_string(),
        crate::cli::IncrementalCoverageOutputFormat::Detailed => "detailed".to_string(),
        crate::cli::IncrementalCoverageOutputFormat::Json => "json".to_string(),
        crate::cli::IncrementalCoverageOutputFormat::Markdown => "markdown".to_string(),
        crate::cli::IncrementalCoverageOutputFormat::Lcov => "lcov".to_string(),
        crate::cli::IncrementalCoverageOutputFormat::Delta => "delta".to_string(),
        crate::cli::IncrementalCoverageOutputFormat::Sarif => "sarif".to_string(),
    }
}

pub fn symbol_type_filter_to_string(filter: &crate::cli::SymbolTypeFilter) -> String {
    match filter {
        crate::cli::SymbolTypeFilter::All => "all".to_string(),
        crate::cli::SymbolTypeFilter::Functions => "functions".to_string(),
        crate::cli::SymbolTypeFilter::Classes => "classes".to_string(),
        crate::cli::SymbolTypeFilter::Types => "types".to_string(),
        crate::cli::SymbolTypeFilter::Variables => "variables".to_string(),
        crate::cli::SymbolTypeFilter::Modules => "modules".to_string(),
    }
}

pub fn symbol_table_format_to_string(format: &crate::cli::SymbolTableOutputFormat) -> String {
    match format {
        crate::cli::SymbolTableOutputFormat::Summary => "summary".to_string(),
        crate::cli::SymbolTableOutputFormat::Detailed => "detailed".to_string(),
        crate::cli::SymbolTableOutputFormat::Human => "human".to_string(),
        crate::cli::SymbolTableOutputFormat::Json => "json".to_string(),
        crate::cli::SymbolTableOutputFormat::Csv => "csv".to_string(),
    }
}

pub fn big_o_format_to_string(format: &crate::cli::BigOOutputFormat) -> String {
    match format {
        crate::cli::BigOOutputFormat::Summary => "summary".to_string(),
        crate::cli::BigOOutputFormat::Json => "json".to_string(),
        crate::cli::BigOOutputFormat::Markdown => "markdown".to_string(),
        crate::cli::BigOOutputFormat::Detailed => "detailed".to_string(),
    }
}

pub fn deep_context_format_to_string(format: &crate::cli::DeepContextOutputFormat) -> String {
    match format {
        crate::cli::DeepContextOutputFormat::Markdown => "markdown".to_string(),
        crate::cli::DeepContextOutputFormat::Json => "json".to_string(),
        crate::cli::DeepContextOutputFormat::Sarif => "sarif".to_string(),
    }
}

pub fn deep_context_dag_type_to_string(dag_type: &crate::cli::DeepContextDagType) -> String {
    match dag_type {
        crate::cli::DeepContextDagType::CallGraph => "call-graph".to_string(),
        crate::cli::DeepContextDagType::ImportGraph => "import-graph".to_string(),
        crate::cli::DeepContextDagType::Inheritance => "inheritance".to_string(),
        crate::cli::DeepContextDagType::FullDependency => "full-dependency".to_string(),
    }
}

pub fn deep_context_cache_strategy_to_string(
    strategy: &crate::cli::DeepContextCacheStrategy,
) -> String {
    match strategy {
        crate::cli::DeepContextCacheStrategy::Normal => "normal".to_string(),
        crate::cli::DeepContextCacheStrategy::ForceRefresh => "force-refresh".to_string(),
        crate::cli::DeepContextCacheStrategy::Offline => "offline".to_string(),
    }
}

pub fn tdg_format_to_string(format: &crate::cli::TdgOutputFormat) -> String {
    match format {
        crate::cli::TdgOutputFormat::Table => "table".to_string(),
        crate::cli::TdgOutputFormat::Json => "json".to_string(),
        crate::cli::TdgOutputFormat::Markdown => "markdown".to_string(),
        crate::cli::TdgOutputFormat::Sarif => "sarif".to_string(),
    }
}

pub fn provability_format_to_string(format: &crate::cli::ProvabilityOutputFormat) -> String {
    match format {
        crate::cli::ProvabilityOutputFormat::Summary => "summary".to_string(),
        crate::cli::ProvabilityOutputFormat::Full => "full".to_string(),
        crate::cli::ProvabilityOutputFormat::Json => "json".to_string(),
        crate::cli::ProvabilityOutputFormat::Sarif => "sarif".to_string(),
        crate::cli::ProvabilityOutputFormat::Markdown => "markdown".to_string(),
    }
}
