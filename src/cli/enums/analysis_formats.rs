#![cfg_attr(coverage_nightly, coverage(off))]
//! Analysis-specific output format enums

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::fmt;

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

/// Graph metrics output format
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum GraphMetricsOutputFormat {
    /// Summary statistics only
    Summary,
    /// Detailed metrics with rankings
    Detailed,
    /// Human-readable format
    Human,
    /// JSON format for tooling integration
    Json,
    /// CSV format for spreadsheet import
    Csv,
    /// `GraphML` export format
    GraphML,
    /// Markdown report format
    Markdown,
}

impl GraphMetricsOutputFormat {
    /// Get the string representation of the output format
    fn as_str(&self) -> &'static str {
        debug_assert!(true, "contract: as_str");
        match self {
            GraphMetricsOutputFormat::Summary => "summary",
            GraphMetricsOutputFormat::Detailed => "detailed",
            GraphMetricsOutputFormat::Human => "human",
            GraphMetricsOutputFormat::Json => "json",
            GraphMetricsOutputFormat::Csv => "csv",
            GraphMetricsOutputFormat::GraphML => "graphml",
            GraphMetricsOutputFormat::Markdown => "markdown",
        }
    }
}

impl fmt::Display for GraphMetricsOutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Name similarity output format
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum NameSimilarityOutputFormat {
    /// Summary of matches only
    Summary,
    /// Detailed match analysis
    Detailed,
    /// Human-readable format
    Human,
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
            NameSimilarityOutputFormat::Human => write!(f, "human"),
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
    /// Human-readable format
    Human,
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
            DuplicateOutputFormat::Human => write!(f, "human"),
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

/// Known Defects output format
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum DefectsOutputFormat {
    Text,
    Json,
    Junit,
}

impl fmt::Display for DefectsOutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DefectsOutputFormat::Text => write!(f, "text"),
            DefectsOutputFormat::Json => write!(f, "json"),
            DefectsOutputFormat::Junit => write!(f, "junit"),
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

/// Symbol table output format
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum SymbolTableOutputFormat {
    /// Summary with statistics
    Summary,
    /// Detailed output with all symbols
    Detailed,
    /// Human-readable format
    Human,
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
            SymbolTableOutputFormat::Human => write!(f, "human"),
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

impl IncrementalCoverageOutputFormat {
    /// Get the string representation of the output format
    fn as_str(&self) -> &'static str {
        debug_assert!(true, "contract: as_str");
        match self {
            IncrementalCoverageOutputFormat::Summary => "summary",
            IncrementalCoverageOutputFormat::Detailed => "detailed",
            IncrementalCoverageOutputFormat::Json => "json",
            IncrementalCoverageOutputFormat::Markdown => "markdown",
            IncrementalCoverageOutputFormat::Lcov => "lcov",
            IncrementalCoverageOutputFormat::Delta => "delta",
            IncrementalCoverageOutputFormat::Sarif => "sarif",
        }
    }
}

impl fmt::Display for IncrementalCoverageOutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Quality gate output format
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum QualityGateOutputFormat {
    Summary,
    Detailed,
    Human,
    Json,
    Junit,
    Markdown,
}

impl fmt::Display for QualityGateOutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QualityGateOutputFormat::Summary => write!(f, "summary"),
            QualityGateOutputFormat::Detailed => write!(f, "detailed"),
            QualityGateOutputFormat::Human => write!(f, "human"),
            QualityGateOutputFormat::Json => write!(f, "json"),
            QualityGateOutputFormat::Junit => write!(f, "junit"),
            QualityGateOutputFormat::Markdown => write!(f, "markdown"),
        }
    }
}

/// Entropy analysis output format
#[derive(Clone, Debug, ValueEnum, PartialEq, Eq, Deserialize, Serialize)]
pub enum EntropyOutputFormat {
    /// Summary with top violations
    Summary,
    /// Detailed violation report
    Detailed,
    /// JSON format for tooling
    Json,
    /// Markdown report
    Markdown,
}

impl fmt::Display for EntropyOutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EntropyOutputFormat::Summary => write!(f, "summary"),
            EntropyOutputFormat::Detailed => write!(f, "detailed"),
            EntropyOutputFormat::Json => write!(f, "json"),
            EntropyOutputFormat::Markdown => write!(f, "markdown"),
        }
    }
}
