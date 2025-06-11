//! CLI enum definitions
//!
//! This module contains all the enum types used by the CLI for command-line parsing
//! and output formatting. Each enum implements Display for testability.

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Execution mode for the CLI
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecutionMode {
    Cli,
    Mcp,
}

impl fmt::Display for ExecutionMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecutionMode::Cli => write!(f, "cli"),
            ExecutionMode::Mcp => write!(f, "mcp"),
        }
    }
}

/// Output format for general commands
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum OutputFormat {
    Table,
    Json,
    Yaml,
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutputFormat::Table => write!(f, "table"),
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Yaml => write!(f, "yaml"),
        }
    }
}

/// Explain level for code explanations
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum ExplainLevel {
    Brief,
    Detailed,
    Verbose,
}

impl fmt::Display for ExplainLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExplainLevel::Brief => write!(f, "brief"),
            ExplainLevel::Detailed => write!(f, "detailed"),
            ExplainLevel::Verbose => write!(f, "verbose"),
        }
    }
}

/// Refactor output format
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum RefactorOutputFormat {
    Json,
    Table,
    Summary,
}

impl fmt::Display for RefactorOutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RefactorOutputFormat::Json => write!(f, "json"),
            RefactorOutputFormat::Table => write!(f, "table"),
            RefactorOutputFormat::Summary => write!(f, "summary"),
        }
    }
}

/// Refactor mode
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum RefactorMode {
    Batch,
    Interactive,
}

impl fmt::Display for RefactorMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RefactorMode::Batch => write!(f, "batch"),
            RefactorMode::Interactive => write!(f, "interactive"),
        }
    }
}

/// Context format
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum ContextFormat {
    Markdown,
    Json,
    Sarif,
    #[value(name = "llm-optimized")]
    LlmOptimized,
}

impl fmt::Display for ContextFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContextFormat::Markdown => write!(f, "markdown"),
            ContextFormat::Json => write!(f, "json"),
            ContextFormat::Sarif => write!(f, "sarif"),
            ContextFormat::LlmOptimized => write!(f, "llm-optimized"),
        }
    }
}

/// TDG output format
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum TdgOutputFormat {
    Table,
    Json,
    Markdown,
    Sarif,
}

impl fmt::Display for TdgOutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TdgOutputFormat::Table => write!(f, "table"),
            TdgOutputFormat::Json => write!(f, "json"),
            TdgOutputFormat::Markdown => write!(f, "markdown"),
            TdgOutputFormat::Sarif => write!(f, "sarif"),
        }
    }
}

/// Makefile output format
#[derive(Clone, Debug, ValueEnum, PartialEq, Serialize, Deserialize)]
pub enum MakefileOutputFormat {
    /// Human-readable output
    Human,
    /// JSON output
    Json,
    /// GCC-style output for editor integration
    Gcc,
    /// SARIF format for CI/CD integration
    Sarif,
}

impl fmt::Display for MakefileOutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MakefileOutputFormat::Human => write!(f, "human"),
            MakefileOutputFormat::Json => write!(f, "json"),
            MakefileOutputFormat::Gcc => write!(f, "gcc"),
            MakefileOutputFormat::Sarif => write!(f, "sarif"),
        }
    }
}

/// Provability output format
#[derive(Clone, Debug, ValueEnum, PartialEq, Serialize, Deserialize)]
pub enum ProvabilityOutputFormat {
    /// Summary statistics only
    Summary,
    /// Full detailed report
    Full,
    /// JSON format for tools
    Json,
    /// SARIF format for CI/CD integration
    Sarif,
    /// Markdown report format
    Markdown,
}

impl fmt::Display for ProvabilityOutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProvabilityOutputFormat::Summary => write!(f, "summary"),
            ProvabilityOutputFormat::Full => write!(f, "full"),
            ProvabilityOutputFormat::Json => write!(f, "json"),
            ProvabilityOutputFormat::Sarif => write!(f, "sarif"),
            ProvabilityOutputFormat::Markdown => write!(f, "markdown"),
        }
    }
}

/// Duplicate type
#[derive(Clone, Debug, ValueEnum, PartialEq, Serialize)]
pub enum DuplicateType {
    /// Exact duplicates (Type 1 clones)
    Exact,
    /// Renamed duplicates (Type 2 clones)
    Renamed,
    /// Gapped duplicates (Type 3 clones)
    Gapped,
    /// Semantic duplicates using AST similarity
    Semantic,
    /// All types of duplicates
    All,
}

impl fmt::Display for DuplicateType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DuplicateType::Exact => write!(f, "exact"),
            DuplicateType::Renamed => write!(f, "renamed"),
            DuplicateType::Gapped => write!(f, "gapped"),
            DuplicateType::Semantic => write!(f, "semantic"),
            DuplicateType::All => write!(f, "all"),
        }
    }
}

/// Defect prediction output format
#[derive(Clone, Debug, ValueEnum, PartialEq, Serialize)]
pub enum DefectPredictionOutputFormat {
    /// Summary statistics only
    Summary,
    /// Detailed analysis with recommendations
    Detailed,
    /// JSON format for tooling
    Json,
    /// CSV format for spreadsheet import
    Csv,
    /// SARIF format for IDE integration
    Sarif,
}

impl fmt::Display for DefectPredictionOutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DefectPredictionOutputFormat::Summary => write!(f, "summary"),
            DefectPredictionOutputFormat::Detailed => write!(f, "detailed"),
            DefectPredictionOutputFormat::Json => write!(f, "json"),
            DefectPredictionOutputFormat::Csv => write!(f, "csv"),
            DefectPredictionOutputFormat::Sarif => write!(f, "sarif"),
        }
    }
}

/// Comprehensive output format
#[derive(Clone, Debug, ValueEnum, PartialEq, Serialize)]
pub enum ComprehensiveOutputFormat {
    /// Executive summary report
    Summary,
    /// Detailed unified analysis report
    Detailed,
    /// JSON format for tooling integration
    Json,
    /// Markdown report format
    Markdown,
    /// SARIF format for IDE integration
    Sarif,
}

impl fmt::Display for ComprehensiveOutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComprehensiveOutputFormat::Summary => write!(f, "summary"),
            ComprehensiveOutputFormat::Detailed => write!(f, "detailed"),
            ComprehensiveOutputFormat::Json => write!(f, "json"),
            ComprehensiveOutputFormat::Markdown => write!(f, "markdown"),
            ComprehensiveOutputFormat::Sarif => write!(f, "sarif"),
        }
    }
}

/// Graph metric type
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum GraphMetricType {
    /// Betweenness centrality
    Centrality,
    /// PageRank scores
    PageRank,
    /// Clustering coefficient
    Clustering,
    /// Connected components analysis
    Components,
    /// All available metrics
    All,
}

impl fmt::Display for GraphMetricType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GraphMetricType::Centrality => write!(f, "centrality"),
            GraphMetricType::PageRank => write!(f, "pagerank"),
            GraphMetricType::Clustering => write!(f, "clustering"),
            GraphMetricType::Components => write!(f, "components"),
            GraphMetricType::All => write!(f, "all"),
        }
    }
}

/// Graph metrics output format
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum GraphMetricsOutputFormat {
    /// Summary statistics only
    Summary,
    /// Detailed metrics with rankings
    Detailed,
    /// JSON format for tooling integration
    Json,
    /// CSV format for spreadsheet import
    Csv,
    /// GraphML export format
    GraphML,
    /// Markdown report format
    Markdown,
}

impl fmt::Display for GraphMetricsOutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GraphMetricsOutputFormat::Summary => write!(f, "summary"),
            GraphMetricsOutputFormat::Detailed => write!(f, "detailed"),
            GraphMetricsOutputFormat::Json => write!(f, "json"),
            GraphMetricsOutputFormat::Csv => write!(f, "csv"),
            GraphMetricsOutputFormat::GraphML => write!(f, "graphml"),
            GraphMetricsOutputFormat::Markdown => write!(f, "markdown"),
        }
    }
}

/// Search scope
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum SearchScope {
    /// Search function names
    Functions,
    /// Search type/class names
    Types,
    /// Search variable names
    Variables,
    /// Search all identifiers
    All,
}

impl fmt::Display for SearchScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SearchScope::Functions => write!(f, "functions"),
            SearchScope::Types => write!(f, "types"),
            SearchScope::Variables => write!(f, "variables"),
            SearchScope::All => write!(f, "all"),
        }
    }
}

/// Name similarity output format
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum NameSimilarityOutputFormat {
    /// Summary of matches only
    Summary,
    /// Detailed match analysis
    Detailed,
    /// JSON format for tooling integration
    Json,
    /// CSV format for spreadsheet import
    Csv,
    /// Markdown report format
    Markdown,
}

impl fmt::Display for NameSimilarityOutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NameSimilarityOutputFormat::Summary => write!(f, "summary"),
            NameSimilarityOutputFormat::Detailed => write!(f, "detailed"),
            NameSimilarityOutputFormat::Json => write!(f, "json"),
            NameSimilarityOutputFormat::Csv => write!(f, "csv"),
            NameSimilarityOutputFormat::Markdown => write!(f, "markdown"),
        }
    }
}

/// Duplicate output format
#[derive(Clone, Debug, ValueEnum, PartialEq, Serialize)]
pub enum DuplicateOutputFormat {
    /// Summary statistics only
    Summary,
    /// Detailed duplicate listing
    Detailed,
    /// JSON format for tooling
    Json,
    /// CSV format for spreadsheet import
    Csv,
    /// SARIF format for IDE integration
    Sarif,
}

impl fmt::Display for DuplicateOutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DuplicateOutputFormat::Summary => write!(f, "summary"),
            DuplicateOutputFormat::Detailed => write!(f, "detailed"),
            DuplicateOutputFormat::Json => write!(f, "json"),
            DuplicateOutputFormat::Csv => write!(f, "csv"),
            DuplicateOutputFormat::Sarif => write!(f, "sarif"),
        }
    }
}

/// Complexity output format
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum ComplexityOutputFormat {
    /// Summary statistics only
    Summary,
    /// Full report with violations
    Full,
    /// JSON format for tools
    Json,
    /// SARIF format for IDE integration
    Sarif,
}

impl fmt::Display for ComplexityOutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComplexityOutputFormat::Summary => write!(f, "summary"),
            ComplexityOutputFormat::Full => write!(f, "full"),
            ComplexityOutputFormat::Json => write!(f, "json"),
            ComplexityOutputFormat::Sarif => write!(f, "sarif"),
        }
    }
}

/// Dead code output format
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum DeadCodeOutputFormat {
    Summary,
    Json,
    Sarif,
    Markdown,
}

impl fmt::Display for DeadCodeOutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeadCodeOutputFormat::Summary => write!(f, "summary"),
            DeadCodeOutputFormat::Json => write!(f, "json"),
            DeadCodeOutputFormat::Sarif => write!(f, "sarif"),
            DeadCodeOutputFormat::Markdown => write!(f, "markdown"),
        }
    }
}

/// SATD output format
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum SatdOutputFormat {
    Summary,
    Json,
    Sarif,
    Markdown,
}

impl fmt::Display for SatdOutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SatdOutputFormat::Summary => write!(f, "summary"),
            SatdOutputFormat::Json => write!(f, "json"),
            SatdOutputFormat::Sarif => write!(f, "sarif"),
            SatdOutputFormat::Markdown => write!(f, "markdown"),
        }
    }
}

/// SATD severity levels
#[derive(Clone, Debug, ValueEnum, PartialEq, PartialOrd, Ord, Eq)]
pub enum SatdSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl fmt::Display for SatdSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SatdSeverity::Low => write!(f, "low"),
            SatdSeverity::Medium => write!(f, "medium"),
            SatdSeverity::High => write!(f, "high"),
            SatdSeverity::Critical => write!(f, "critical"),
        }
    }
}

/// Symbol table output format
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum SymbolTableOutputFormat {
    /// Summary with statistics
    Summary,
    /// Detailed output with all symbols
    Detailed,
    /// JSON format for tools
    Json,
    /// CSV format for spreadsheets
    Csv,
}

impl fmt::Display for SymbolTableOutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SymbolTableOutputFormat::Summary => write!(f, "summary"),
            SymbolTableOutputFormat::Detailed => write!(f, "detailed"),
            SymbolTableOutputFormat::Json => write!(f, "json"),
            SymbolTableOutputFormat::Csv => write!(f, "csv"),
        }
    }
}

/// Big-O output format
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum BigOOutputFormat {
    /// Summary with complexity distribution
    Summary,
    /// JSON format for tools
    Json,
    /// Markdown report
    Markdown,
    /// Detailed analysis with all functions
    Detailed,
}

impl fmt::Display for BigOOutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BigOOutputFormat::Summary => write!(f, "summary"),
            BigOOutputFormat::Json => write!(f, "json"),
            BigOOutputFormat::Markdown => write!(f, "markdown"),
            BigOOutputFormat::Detailed => write!(f, "detailed"),
        }
    }
}

/// Symbol type filter
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum SymbolTypeFilter {
    /// Functions and methods
    Functions,
    /// Types, structs, and classes
    Types,
    /// Variables and constants
    Variables,
    /// Modules and namespaces
    Modules,
    /// All symbols
    All,
}

impl fmt::Display for SymbolTypeFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SymbolTypeFilter::Functions => write!(f, "functions"),
            SymbolTypeFilter::Types => write!(f, "types"),
            SymbolTypeFilter::Variables => write!(f, "variables"),
            SymbolTypeFilter::Modules => write!(f, "modules"),
            SymbolTypeFilter::All => write!(f, "all"),
        }
    }
}

/// DAG generation type
#[derive(Clone, Debug, ValueEnum, PartialEq, Eq, Hash)]
pub enum DagType {
    /// Function call graph
    #[value(name = "call-graph")]
    CallGraph,

    /// Import/dependency graph
    #[value(name = "import-graph")]
    ImportGraph,

    /// Class inheritance hierarchy
    #[value(name = "inheritance")]
    Inheritance,

    /// Complete dependency graph
    #[value(name = "full-dependency")]
    FullDependency,
}

impl fmt::Display for DagType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DagType::CallGraph => write!(f, "call-graph"),
            DagType::ImportGraph => write!(f, "import-graph"),
            DagType::Inheritance => write!(f, "inheritance"),
            DagType::FullDependency => write!(f, "full-dependency"),
        }
    }
}

/// Deep context output format
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum DeepContextOutputFormat {
    Markdown,
    Json,
    Sarif,
}

impl fmt::Display for DeepContextOutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeepContextOutputFormat::Markdown => write!(f, "markdown"),
            DeepContextOutputFormat::Json => write!(f, "json"),
            DeepContextOutputFormat::Sarif => write!(f, "sarif"),
        }
    }
}

/// Deep context DAG type
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum DeepContextDagType {
    #[value(name = "call-graph")]
    CallGraph,
    #[value(name = "import-graph")]
    ImportGraph,
    #[value(name = "inheritance")]
    Inheritance,
    #[value(name = "full-dependency")]
    FullDependency,
}

impl fmt::Display for DeepContextDagType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeepContextDagType::CallGraph => write!(f, "call-graph"),
            DeepContextDagType::ImportGraph => write!(f, "import-graph"),
            DeepContextDagType::Inheritance => write!(f, "inheritance"),
            DeepContextDagType::FullDependency => write!(f, "full-dependency"),
        }
    }
}

/// Deep context cache strategy
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum DeepContextCacheStrategy {
    Normal,
    #[value(name = "force-refresh")]
    ForceRefresh,
    Offline,
}

impl fmt::Display for DeepContextCacheStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeepContextCacheStrategy::Normal => write!(f, "normal"),
            DeepContextCacheStrategy::ForceRefresh => write!(f, "force-refresh"),
            DeepContextCacheStrategy::Offline => write!(f, "offline"),
        }
    }
}

/// Demo protocol selection
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum DemoProtocol {
    Cli,
    Http,
    Mcp,
    #[cfg(feature = "tui")]
    Tui,
    All,
}

impl fmt::Display for DemoProtocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DemoProtocol::Cli => write!(f, "cli"),
            DemoProtocol::Http => write!(f, "http"),
            DemoProtocol::Mcp => write!(f, "mcp"),
            #[cfg(feature = "tui")]
            DemoProtocol::Tui => write!(f, "tui"),
            DemoProtocol::All => write!(f, "all"),
        }
    }
}

/// Proof annotation output format
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum ProofAnnotationOutputFormat {
    Summary,
    Full,
    Json,
    Markdown,
    Sarif,
}

impl fmt::Display for ProofAnnotationOutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProofAnnotationOutputFormat::Summary => write!(f, "summary"),
            ProofAnnotationOutputFormat::Full => write!(f, "full"),
            ProofAnnotationOutputFormat::Json => write!(f, "json"),
            ProofAnnotationOutputFormat::Markdown => write!(f, "markdown"),
            ProofAnnotationOutputFormat::Sarif => write!(f, "sarif"),
        }
    }
}

/// Property type filter
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum PropertyTypeFilter {
    MemorySafety,
    ThreadSafety,
    DataRaceFreeze,
    Termination,
    FunctionalCorrectness,
    ResourceBounds,
    All,
}

impl fmt::Display for PropertyTypeFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PropertyTypeFilter::MemorySafety => write!(f, "memory-safety"),
            PropertyTypeFilter::ThreadSafety => write!(f, "thread-safety"),
            PropertyTypeFilter::DataRaceFreeze => write!(f, "data-race-freeze"),
            PropertyTypeFilter::Termination => write!(f, "termination"),
            PropertyTypeFilter::FunctionalCorrectness => write!(f, "functional-correctness"),
            PropertyTypeFilter::ResourceBounds => write!(f, "resource-bounds"),
            PropertyTypeFilter::All => write!(f, "all"),
        }
    }
}

/// Verification method filter
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum VerificationMethodFilter {
    FormalProof,
    ModelChecking,
    StaticAnalysis,
    AbstractInterpretation,
    BorrowChecker,
    All,
}

impl fmt::Display for VerificationMethodFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VerificationMethodFilter::FormalProof => write!(f, "formal-proof"),
            VerificationMethodFilter::ModelChecking => write!(f, "model-checking"),
            VerificationMethodFilter::StaticAnalysis => write!(f, "static-analysis"),
            VerificationMethodFilter::AbstractInterpretation => {
                write!(f, "abstract-interpretation")
            }
            VerificationMethodFilter::BorrowChecker => write!(f, "borrow-checker"),
            VerificationMethodFilter::All => write!(f, "all"),
        }
    }
}

/// Incremental coverage output format
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum IncrementalCoverageOutputFormat {
    Summary,
    Detailed,
    Json,
    Markdown,
    Lcov,
    Delta,
    Sarif,
}

impl fmt::Display for IncrementalCoverageOutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IncrementalCoverageOutputFormat::Summary => write!(f, "summary"),
            IncrementalCoverageOutputFormat::Detailed => write!(f, "detailed"),
            IncrementalCoverageOutputFormat::Json => write!(f, "json"),
            IncrementalCoverageOutputFormat::Markdown => write!(f, "markdown"),
            IncrementalCoverageOutputFormat::Lcov => write!(f, "lcov"),
            IncrementalCoverageOutputFormat::Delta => write!(f, "delta"),
            IncrementalCoverageOutputFormat::Sarif => write!(f, "sarif"),
        }
    }
}

/// Quality gate output format
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum QualityGateOutputFormat {
    Summary,
    Detailed,
    Json,
    Junit,
    Markdown,
}

impl fmt::Display for QualityGateOutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QualityGateOutputFormat::Summary => write!(f, "summary"),
            QualityGateOutputFormat::Detailed => write!(f, "detailed"),
            QualityGateOutputFormat::Json => write!(f, "json"),
            QualityGateOutputFormat::Junit => write!(f, "junit"),
            QualityGateOutputFormat::Markdown => write!(f, "markdown"),
        }
    }
}

/// Report output format
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum ReportOutputFormat {
    Html,
    Markdown,
    Json,
    Pdf,
    Dashboard,
}

impl fmt::Display for ReportOutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReportOutputFormat::Html => write!(f, "html"),
            ReportOutputFormat::Markdown => write!(f, "markdown"),
            ReportOutputFormat::Json => write!(f, "json"),
            ReportOutputFormat::Pdf => write!(f, "pdf"),
            ReportOutputFormat::Dashboard => write!(f, "dashboard"),
        }
    }
}

/// Analysis type
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum AnalysisType {
    Complexity,
    DeadCode,
    Duplication,
    TechnicalDebt,
    BigO,
    All,
}

impl fmt::Display for AnalysisType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnalysisType::Complexity => write!(f, "complexity"),
            AnalysisType::DeadCode => write!(f, "dead-code"),
            AnalysisType::Duplication => write!(f, "duplication"),
            AnalysisType::TechnicalDebt => write!(f, "technical-debt"),
            AnalysisType::BigO => write!(f, "big-o"),
            AnalysisType::All => write!(f, "all"),
        }
    }
}

/// Quality check type
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum QualityCheckType {
    DeadCode,
    Complexity,
    Coverage,
    Sections,
    Provability,
    All,
}

impl fmt::Display for QualityCheckType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QualityCheckType::DeadCode => write!(f, "dead-code"),
            QualityCheckType::Complexity => write!(f, "complexity"),
            QualityCheckType::Coverage => write!(f, "coverage"),
            QualityCheckType::Sections => write!(f, "sections"),
            QualityCheckType::Provability => write!(f, "provability"),
            QualityCheckType::All => write!(f, "all"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_mode_display() {
        assert_eq!(ExecutionMode::Cli.to_string(), "cli");
        assert_eq!(ExecutionMode::Mcp.to_string(), "mcp");
    }

    #[test]
    fn test_output_format_display() {
        assert_eq!(OutputFormat::Table.to_string(), "table");
        assert_eq!(OutputFormat::Json.to_string(), "json");
        assert_eq!(OutputFormat::Yaml.to_string(), "yaml");
    }

    #[test]
    fn test_satd_severity_ordering() {
        assert!(SatdSeverity::Low < SatdSeverity::Medium);
        assert!(SatdSeverity::Medium < SatdSeverity::High);
        assert!(SatdSeverity::High < SatdSeverity::Critical);
    }

    #[test]
    fn test_all_enum_displays() {
        // Test a sample from each enum to ensure Display is implemented
        assert_eq!(ExplainLevel::Brief.to_string(), "brief");
        assert_eq!(RefactorOutputFormat::Json.to_string(), "json");
        assert_eq!(RefactorMode::Batch.to_string(), "batch");
        assert_eq!(ContextFormat::Markdown.to_string(), "markdown");
        assert_eq!(TdgOutputFormat::Table.to_string(), "table");
        assert_eq!(MakefileOutputFormat::Human.to_string(), "human");
        assert_eq!(DuplicateType::Exact.to_string(), "exact");
        assert_eq!(GraphMetricType::PageRank.to_string(), "pagerank");
        assert_eq!(SearchScope::Functions.to_string(), "functions");
        assert_eq!(ComplexityOutputFormat::Summary.to_string(), "summary");
        assert_eq!(DeadCodeOutputFormat::Json.to_string(), "json");
        assert_eq!(SatdOutputFormat::Markdown.to_string(), "markdown");
        assert_eq!(SymbolTableOutputFormat::Csv.to_string(), "csv");
        assert_eq!(BigOOutputFormat::Detailed.to_string(), "detailed");
        assert_eq!(SymbolTypeFilter::All.to_string(), "all");
        assert_eq!(DagType::CallGraph.to_string(), "call-graph");
        assert_eq!(DeepContextOutputFormat::Sarif.to_string(), "sarif");
        assert_eq!(DemoProtocol::Http.to_string(), "http");
        assert_eq!(AnalysisType::BigO.to_string(), "big-o");
        assert_eq!(QualityCheckType::Coverage.to_string(), "coverage");
    }

    #[test]
    fn test_enum_equality() {
        assert_eq!(ExecutionMode::Cli, ExecutionMode::Cli);
        assert_ne!(ExecutionMode::Cli, ExecutionMode::Mcp);

        assert_eq!(OutputFormat::Json, OutputFormat::Json);
        assert_ne!(OutputFormat::Json, OutputFormat::Table);

        assert_eq!(SatdSeverity::Low, SatdSeverity::Low);
        assert_ne!(SatdSeverity::Low, SatdSeverity::High);
    }
}
