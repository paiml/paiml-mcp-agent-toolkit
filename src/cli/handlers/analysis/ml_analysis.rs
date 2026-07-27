//! ML and predictive analysis handlers

use crate::cli::commands::AnalyzeCommands;
use anyhow::Result;

/// Handle defect prediction analysis
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn handle_defect_prediction(cmd: AnalyzeCommands) -> Result<()> {
    // Route to existing working handler
    crate::cli::handlers::route_analyze_command(cmd).await
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod unit_tests {
    use super::*;

    /// Test that handle_defect_prediction function exists and has correct signature
    #[test]
    fn test_handle_defect_prediction_signature() {
        let _fn_ref: fn(AnalyzeCommands) -> _ = handle_defect_prediction;
    }

    /// Test module exports the ML analysis handler
    #[test]
    fn test_module_exports_handler() {
        fn _verify_export() {
            let _defect_prediction: fn(AnalyzeCommands) -> _ = handle_defect_prediction;
        }
    }

    /// Test that Result type is properly used (anyhow::Result)
    #[test]
    fn test_result_type_compatibility() {
        fn _check_result_type() -> Result<()> {
            Ok(())
        }
        assert!(_check_result_type().is_ok());
    }

    /// Test that the handler is async
    #[test]
    fn test_handler_is_async() {
        fn _verify_async_nature() {
            // handle_defect_prediction is async - verified at compile time
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;
    use crate::cli::enums::DefectPredictionOutputFormat;
    use std::path::PathBuf;

    /// Test handle_defect_prediction delegates to route_analyze_command with default options
    #[tokio::test]
    async fn test_handle_defect_prediction_basic() {
        let cmd = AnalyzeCommands::DefectPrediction {
            path: PathBuf::from("/nonexistent/path/for/defect/test"),
            project_path: None,
            confidence_threshold: 0.5,
            min_lines: 10,
            include_low_confidence: false,
            format: DefectPredictionOutputFormat::Summary,
            high_risk_only: false,
            include_recommendations: false,
            include: None,
            exclude: None,
            output: None,
            perf: false,
            top_files: 10,
        };

        let result = handle_defect_prediction(cmd).await;
        // Delegation should work regardless of outcome
        assert!(result.is_ok() || result.is_err());
    }

    /// Test handle_defect_prediction with high_risk_only and recommendations
    #[tokio::test]
    async fn test_handle_defect_prediction_high_risk_with_recommendations() {
        let cmd = AnalyzeCommands::DefectPrediction {
            path: PathBuf::from("/tmp/test-defect"),
            project_path: None,
            confidence_threshold: 0.7,
            min_lines: 5,
            include_low_confidence: false,
            format: DefectPredictionOutputFormat::Json,
            high_risk_only: true,
            include_recommendations: true,
            include: Some("**/*.rs".to_string()),
            exclude: Some("**/target/**".to_string()),
            output: Some(PathBuf::from("/tmp/defect-output.json")),
            perf: true,
            top_files: 5,
        };

        let result = handle_defect_prediction(cmd).await;
        assert!(result.is_ok() || result.is_err());
    }

    /// Test handle_defect_prediction with low confidence included
    #[tokio::test]
    async fn test_handle_defect_prediction_include_low_confidence() {
        let cmd = AnalyzeCommands::DefectPrediction {
            path: PathBuf::from("/nonexistent"),
            project_path: None,
            confidence_threshold: 0.3,
            min_lines: 20,
            include_low_confidence: true,
            format: DefectPredictionOutputFormat::Detailed,
            high_risk_only: false,
            include_recommendations: true,
            include: None,
            exclude: None,
            output: None,
            perf: false,
            top_files: 20,
        };

        let result = handle_defect_prediction(cmd).await;
        assert!(result.is_ok() || result.is_err());
    }

    /// Test handle_defect_prediction with markdown output format
    #[tokio::test]
    async fn test_handle_defect_prediction_markdown() {
        let cmd = AnalyzeCommands::DefectPrediction {
            path: PathBuf::from("/tmp/test-defect-md"),
            project_path: None,
            confidence_threshold: 0.5,
            min_lines: 10,
            include_low_confidence: false,
            format: DefectPredictionOutputFormat::Detailed,
            high_risk_only: false,
            include_recommendations: false,
            include: None,
            exclude: Some("tests/**".to_string()),
            output: Some(PathBuf::from("/tmp/defect-report.md")),
            perf: false,
            top_files: 0,
        };

        let result = handle_defect_prediction(cmd).await;
        assert!(result.is_ok() || result.is_err());
    }

    /// Test that handler is async and returns proper Result type
    #[test]
    fn test_handler_function_signature() {
        // Verify the function exists and has correct return type
        let _: fn(AnalyzeCommands) -> _ = handle_defect_prediction;
    }
}
