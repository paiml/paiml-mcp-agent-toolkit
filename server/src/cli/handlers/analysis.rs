//! Analysis command handlers
//!
//! This module contains all analysis-related command handlers,
//! further subdivided into logical groups to reduce complexity.

use std::path::PathBuf;
use anyhow::Result;

pub mod code_quality;
pub mod complexity;
pub mod dependencies;
pub mod duplication;
pub mod ml_analysis;
pub mod technical_debt;

/// Main router for analysis commands
pub async fn analyze_router(cmd: super::super::AnalyzeCommands) -> Result<()> {
    use super::super::AnalyzeCommands;
    
    match cmd {
        // Code quality analysis
        AnalyzeCommands::Complexity { .. } => complexity::handle_complexity(cmd).await,
        AnalyzeCommands::DeadCode { .. } => code_quality::handle_dead_code(cmd).await,
        AnalyzeCommands::Satd { .. } => code_quality::handle_satd(cmd).await,
        AnalyzeCommands::Makefile { .. } => code_quality::handle_makefile(cmd).await,
        
        // Dependency and structure analysis  
        AnalyzeCommands::Dag { .. } => dependencies::handle_dag(cmd).await,
        AnalyzeCommands::GraphMetrics { .. } => dependencies::handle_graph_metrics(cmd).await,
        AnalyzeCommands::SymbolTable { .. } => dependencies::handle_symbol_table(cmd).await,
        
        // Duplication and similarity
        AnalyzeCommands::Duplicates { .. } => duplication::handle_duplicates(cmd).await,
        AnalyzeCommands::NameSimilarity { .. } => duplication::handle_name_similarity(cmd).await,
        
        // ML and predictive analysis
        AnalyzeCommands::DefectPrediction { .. } => ml_analysis::handle_defect_prediction(cmd).await,
        AnalyzeCommands::Provability { .. } => ml_analysis::handle_provability(cmd).await,
        AnalyzeCommands::ProofAnnotations { .. } => ml_analysis::handle_proof_annotations(cmd).await,
        
        // Technical debt and comprehensive analysis
        AnalyzeCommands::Tdg { .. } => technical_debt::handle_tdg(cmd).await,
        AnalyzeCommands::DeepContext { .. } => technical_debt::handle_deep_context(cmd).await,
        AnalyzeCommands::Comprehensive { .. } => technical_debt::handle_comprehensive(cmd).await,
        
        // Other analyses
        AnalyzeCommands::Churn { .. } => code_quality::handle_churn(cmd).await,
        AnalyzeCommands::IncrementalCoverage { .. } => code_quality::handle_incremental_coverage(cmd).await,
        AnalyzeCommands::QualityGate { .. } => technical_debt::handle_quality_gate(cmd).await,
    }
}

pub struct AnalysisHandlers;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analysis_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }
}
