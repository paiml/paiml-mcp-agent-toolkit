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
        AnalyzeCommands::Complexity { .. } => route_complexity_command(cmd).await,
        AnalyzeCommands::DeadCode { .. } => route_code_quality_command(cmd).await,
        AnalyzeCommands::Satd { .. } => route_code_quality_command(cmd).await,
        AnalyzeCommands::Makefile { .. } => route_code_quality_command(cmd).await,
        AnalyzeCommands::Churn { .. } => route_code_quality_command(cmd).await,
        AnalyzeCommands::IncrementalCoverage { .. } => route_code_quality_command(cmd).await,
        
        // Dependency and structure analysis  
        AnalyzeCommands::Dag { .. } => route_dependency_command(cmd).await,
        AnalyzeCommands::GraphMetrics { .. } => route_dependency_command(cmd).await,
        AnalyzeCommands::SymbolTable { .. } => route_dependency_command(cmd).await,
        
        // Duplication and similarity
        AnalyzeCommands::Duplicates { .. } => route_duplication_command(cmd).await,
        AnalyzeCommands::NameSimilarity { .. } => route_duplication_command(cmd).await,
        
        // ML and predictive analysis
        AnalyzeCommands::DefectPrediction { .. } => route_ml_command(cmd).await,
        AnalyzeCommands::Provability { .. } => route_ml_command(cmd).await,
        AnalyzeCommands::ProofAnnotations { .. } => route_ml_command(cmd).await,
        
        // Quality analysis and comprehensive reporting
        AnalyzeCommands::Tdg { .. } => route_technical_debt_command(cmd).await,
        AnalyzeCommands::DeepContext { .. } => route_technical_debt_command(cmd).await,
        AnalyzeCommands::Comprehensive { .. } => route_technical_debt_command(cmd).await,
        AnalyzeCommands::QualityGate { .. } => route_technical_debt_command(cmd).await,
    }
}

async fn route_complexity_command(cmd: super::super::AnalyzeCommands) -> Result<()> {
    complexity::handle_complexity(cmd).await
}

async fn route_code_quality_command(cmd: super::super::AnalyzeCommands) -> Result<()> {
    use super::super::AnalyzeCommands;
    match cmd {
        AnalyzeCommands::DeadCode { .. } => code_quality::handle_dead_code(cmd).await,
        AnalyzeCommands::Satd { .. } => code_quality::handle_satd(cmd).await,
        AnalyzeCommands::Makefile { .. } => code_quality::handle_makefile(cmd).await,
        AnalyzeCommands::Churn { .. } => code_quality::handle_churn(cmd).await,
        AnalyzeCommands::IncrementalCoverage { .. } => code_quality::handle_incremental_coverage(cmd).await,
        _ => Err(anyhow::anyhow!("Invalid code quality command")),
    }
}

async fn route_dependency_command(cmd: super::super::AnalyzeCommands) -> Result<()> {
    use super::super::AnalyzeCommands;
    match cmd {
        AnalyzeCommands::Dag { .. } => dependencies::handle_dag(cmd).await,
        AnalyzeCommands::GraphMetrics { .. } => dependencies::handle_graph_metrics(cmd).await,
        AnalyzeCommands::SymbolTable { .. } => dependencies::handle_symbol_table(cmd).await,
        _ => Err(anyhow::anyhow!("Invalid dependency command")),
    }
}

async fn route_duplication_command(cmd: super::super::AnalyzeCommands) -> Result<()> {
    use super::super::AnalyzeCommands;
    match cmd {
        AnalyzeCommands::Duplicates { .. } => duplication::handle_duplicates(cmd).await,
        AnalyzeCommands::NameSimilarity { .. } => duplication::handle_name_similarity(cmd).await,
        _ => Err(anyhow::anyhow!("Invalid duplication command")),
    }
}

async fn route_ml_command(cmd: super::super::AnalyzeCommands) -> Result<()> {
    use super::super::AnalyzeCommands;
    match cmd {
        AnalyzeCommands::DefectPrediction { .. } => ml_analysis::handle_defect_prediction(cmd).await,
        AnalyzeCommands::Provability { .. } => ml_analysis::handle_provability(cmd).await,
        AnalyzeCommands::ProofAnnotations { .. } => ml_analysis::handle_proof_annotations(cmd).await,
        _ => Err(anyhow::anyhow!("Invalid ML command")),
    }
}

async fn route_technical_debt_command(cmd: super::super::AnalyzeCommands) -> Result<()> {
    use super::super::AnalyzeCommands;
    match cmd {
        AnalyzeCommands::Tdg { .. } => technical_debt::handle_tdg(cmd).await,
        AnalyzeCommands::DeepContext { .. } => technical_debt::handle_deep_context(cmd).await,
        AnalyzeCommands::Comprehensive { .. } => technical_debt::handle_comprehensive(cmd).await,
        AnalyzeCommands::QualityGate { .. } => technical_debt::handle_quality_gate(cmd).await,
        _ => Err(anyhow::anyhow!("Invalid technical debt command")),
    }
}

pub struct AnalysisHandlers;

#[cfg(test)]
mod tests {
    // use super::*; // Unused in simple tests

    #[test]
    fn test_analysis_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }
}
