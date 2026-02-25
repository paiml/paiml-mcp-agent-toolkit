//! Coverage tests for tools handlers
//! Extracted for file health compliance (CB-040)

use super::*;
use crate::models::churn::{ChurnSummary, CodeChurnAnalysis, FileChurnMetrics};
use crate::models::dead_code::{
    ConfidenceLevel, DeadCodeItem, DeadCodeRankingResult, DeadCodeSummary, DeadCodeType,
    FileDeadCodeMetrics,
};
use crate::models::defect_prediction::{DefectPredictionResult, DefectProbability, DefectRiskLevel};
use crate::models::tdg::{TDGComponents, TDGScore, TDGSeverity};
use serde_json::json;
use std::path::PathBuf;

// Part 1: Tool identification tests (is_template_tool, is_analysis_tool)
// and churn formatting tests (format_churn_summary, format_churn_as_markdown, format_churn_as_csv)
include!("tools_coverage_tests_tool_identification.rs");

// Part 2: Template variant, relevance scoring, path resolution,
// toolchain detection, file matching, complexity thresholds,
// DAG type parsing, deep context parsing, cache strategy, and analysis type parsing
include!("tools_coverage_tests_path_and_parsing.rs");

// Part 3: Cyclomatic/cognitive complexity calculations, duplicate ratio,
// coupling metrics, utility functions, defaults, TDG formatting,
// and dead code formatting
include!("tools_coverage_tests_calculations.rs");

// Part 4: Parameter validation, SATD detector, lint hotspot extraction,
// lint output formatting, tool call parsing, argument parsing,
// churn parameter extraction, and churn output/response building
include!("tools_coverage_tests_validation_and_args.rs");
}
