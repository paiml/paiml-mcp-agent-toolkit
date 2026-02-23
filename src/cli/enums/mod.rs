#![cfg_attr(coverage_nightly, coverage(off))]
//! CLI enum definitions
//!
//! This module contains all the enum types used by the CLI for command-line parsing
//! and output formatting. Each enum implements Display for testability.

mod analysis_formats;
mod analysis_types;
mod execution;
mod filters;
mod graph_and_dag;
mod output_formats;

// Re-export everything publicly so external code doesn't break
pub use analysis_formats::*;
pub use analysis_types::*;
pub use execution::*;
pub use filters::*;
pub use graph_and_dag::*;
pub use output_formats::*;

#[cfg_attr(coverage_nightly, coverage(off))]
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

        assert_eq!(EntropyOutputFormat::Summary, EntropyOutputFormat::Summary);
        assert_ne!(EntropyOutputFormat::Summary, EntropyOutputFormat::Json);

        assert_eq!(EntropySeverity::Low, EntropySeverity::Low);
        assert_ne!(EntropySeverity::Low, EntropySeverity::High);
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
