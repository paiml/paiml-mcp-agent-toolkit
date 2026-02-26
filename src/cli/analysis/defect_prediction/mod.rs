//! Defect prediction analysis implementation using real ML-based service

mod handler;
mod summary_format;
mod detailed_format;
mod output_formats;

// Re-export the main handler for external use
pub use handler::handle_analyze_defect_prediction;

// Re-export internal items for tests (used by coverage_tests and tests modules via super::*)
#[cfg(test)]
pub(crate) use crate::cli::DefectPredictionOutputFormat;
#[cfg(test)]
pub(crate) use crate::services::defect_probability::DefectScore;
#[cfg(test)]
pub(crate) use handler::{create_defect_prediction_config, filter_and_sort_predictions};
#[cfg(test)]
pub(crate) use summary_format::{
    calculate_risk_statistics, format_defect_summary, get_risk_icon, write_risk_distribution,
    write_summary_footer, write_summary_header, write_top_risk_files, RiskStatistics,
};
#[cfg(test)]
pub(crate) use detailed_format::{
    format_defect_detailed, format_defect_json, format_risk_level_display, write_analysis_footer,
    write_confidence_level, write_contributing_factors, write_detailed_header, write_file_details,
    write_recommendations, write_risk_level,
};
#[cfg(test)]
pub(crate) use output_formats::{
    format_defect_csv, format_defect_output, format_defect_sarif,
};

#[cfg(test)]
mod coverage_tests;

/// Active unit tests for defect prediction module
// Tests extracted to defect_prediction_tests.rs for file health compliance (CB-040)
#[cfg(test)]
#[path = "defect_prediction_tests.rs"]
mod tests;
