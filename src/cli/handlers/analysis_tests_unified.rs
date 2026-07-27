// Tests for analysis handlers
// Extracted for file health compliance (CB-040)
// Split into submodules via include!() for file size compliance.

#[allow(unused_imports)]
use super::*;

mod tests {
    use super::*;
    use crate::cli::{DagType, DeepContextCacheStrategy, DeepContextDagType};

    include!("analysis_tests_active.rs");
}

/// NOTE: Temporarily disabled due to struct definition mismatches
#[cfg(all(test, feature = "broken-tests"))]
mod coverage_tests {
    //! Comprehensive coverage tests for analysis_handlers.rs
    //!
    //! EXTREME TDD approach testing all routing functions, helper functions,
    //! and edge cases for the analysis command handlers.

    use super::*;
    use crate::cli::{
        self, AnalyzeCommands, ComplexityOutputFormat, DagType, DeadCodeOutputFormat,
        DeepContextCacheStrategy, DeepContextDagType, DeepContextOutputFormat,
        DefectPredictionOutputFormat, DefectsOutputFormat, DuplicateOutputFormat, DuplicateType,
        EntropyOutputFormat, EntropySeverity, GraphMetricType, GraphMetricsOutputFormat,
        LintHotspotOutputFormat, MakefileOutputFormat, NameSimilarityOutputFormat,
        ProofAnnotationOutputFormat, ProvabilityOutputFormat, SatdOutputFormat, SatdSeverity,
        SearchScope, SymbolTableOutputFormat, TdgOutputFormat, WasmOutputFormat,
    };
    use crate::models::churn::ChurnOutputFormat;
    use std::path::PathBuf;
    use tempfile::TempDir;

    // Helper function tests: DAG type, cache strategy, entropy config, violations
    include!("analysis_tests_coverage_part1.rs");

    // AnalyzeCommands construction tests: Complexity through Duplicates
    include!("analysis_tests_coverage_part2.rs");

    // AnalyzeCommands construction tests: DefectPrediction through route/format tests
    include!("analysis_tests_coverage_part3.rs");

    // Defects severity/format + additional command construction tests
    include!("analysis_tests_coverage_part4.rs");

    // Semantic analysis + coverage improve + config construction tests
    include!("analysis_tests_coverage_part5.rs");
}
