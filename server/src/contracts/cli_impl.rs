//! CLI implementation using uniform contracts
//! This provides a contract-based CLI executor that uses the uniform contract system

use super::*;
use crate::cli::{commands::AnalyzeCommands, Commands};
use anyhow::Result;
use std::sync::Arc;

/// CLI handler that uses contracts for consistent parameter processing
pub struct ContractCliHandler {
    service: Arc<crate::contracts::service::ContractService>,
}

impl ContractCliHandler {
    pub fn new() -> Result<Self> {
        Ok(Self {
            service: Arc::new(crate::contracts::service::ContractService::new()?),
        })
    }

    /// Process CLI commands using uniform contracts
    pub async fn handle_command(&self, cmd: Commands) -> Result<()> {
        match cmd {
            Commands::Analyze(analyze_cmd) => self.handle_analyze_command(analyze_cmd).await,
            _ => Err(anyhow::anyhow!(
                "Command not yet implemented with contracts"
            )),
        }
    }

    /// Handle analyze subcommands
    async fn handle_analyze_command(&self, cmd: AnalyzeCommands) -> Result<()> {
        // Print deprecation warnings if any
        let warnings = super::adapter::ContractAdapter::deprecation_warnings(&cmd);
        for warning in warnings {
            eprintln!("{}", warning);
        }

        // Process based on command type
        let result = match cmd {
            AnalyzeCommands::Complexity { .. } => self.handle_complexity_analysis(&cmd).await?,
            AnalyzeCommands::Satd { .. } => self.handle_satd_analysis(&cmd).await?,
            AnalyzeCommands::DeadCode { .. } => self.handle_dead_code_analysis(&cmd).await?,
            AnalyzeCommands::Tdg { .. } => self.handle_tdg_analysis(&cmd).await?,
            AnalyzeCommands::LintHotspot { .. } => self.handle_lint_hotspot_analysis(&cmd).await?,
            _ => {
                return Err(anyhow::anyhow!(
                    "Command not yet implemented with contracts"
                ))
            }
        };

        // Output result
        self.output_result(result, &cmd)?;
        Ok(())
    }

    async fn handle_complexity_analysis(&self, cmd: &AnalyzeCommands) -> Result<serde_json::Value> {
        let contract = super::adapter::ContractAdapter::from_cli(cmd)?;
        if let Ok(complexity_contract) = contract
            .as_any()
            .downcast_ref::<AnalyzeComplexityContract>()
        {
            self.service
                .analyze_complexity(complexity_contract.clone())
                .await
        } else {
            Err(anyhow::anyhow!(
                "Invalid contract type for complexity analysis"
            ))
        }
    }

    async fn handle_satd_analysis(&self, cmd: &AnalyzeCommands) -> Result<serde_json::Value> {
        let contract = super::adapter::ContractAdapter::from_cli(cmd)?;
        if let Ok(satd_contract) = contract.as_any().downcast_ref::<AnalyzeSatdContract>() {
            self.service.analyze_satd(satd_contract.clone()).await
        } else {
            Err(anyhow::anyhow!("Invalid contract type for SATD analysis"))
        }
    }

    async fn handle_dead_code_analysis(&self, cmd: &AnalyzeCommands) -> Result<serde_json::Value> {
        let contract = super::adapter::ContractAdapter::from_cli(cmd)?;
        if let Ok(dead_code_contract) = contract.as_any().downcast_ref::<AnalyzeDeadCodeContract>()
        {
            self.service
                .analyze_dead_code(dead_code_contract.clone())
                .await
        } else {
            Err(anyhow::anyhow!(
                "Invalid contract type for dead code analysis"
            ))
        }
    }

    async fn handle_tdg_analysis(&self, cmd: &AnalyzeCommands) -> Result<serde_json::Value> {
        let contract = super::adapter::ContractAdapter::from_cli(cmd)?;
        if let Ok(tdg_contract) = contract.as_any().downcast_ref::<AnalyzeTdgContract>() {
            self.service.analyze_tdg(tdg_contract.clone()).await
        } else {
            Err(anyhow::anyhow!("Invalid contract type for TDG analysis"))
        }
    }

    async fn handle_lint_hotspot_analysis(
        &self,
        cmd: &AnalyzeCommands,
    ) -> Result<serde_json::Value> {
        let contract = super::adapter::ContractAdapter::from_cli(cmd)?;
        if let Ok(lint_contract) = contract
            .as_any()
            .downcast_ref::<AnalyzeLintHotspotContract>()
        {
            self.service
                .analyze_lint_hotspot(lint_contract.clone())
                .await
        } else {
            Err(anyhow::anyhow!(
                "Invalid contract type for lint hotspot analysis"
            ))
        }
    }

    /// Output result based on command configuration
    fn output_result(&self, result: serde_json::Value, cmd: &AnalyzeCommands) -> Result<()> {
        // Get output path if specified
        let output_path = match cmd {
            AnalyzeCommands::Complexity { output, .. }
            | AnalyzeCommands::Satd { output, .. }
            | AnalyzeCommands::DeadCode { output, .. }
            | AnalyzeCommands::Tdg { output, .. }
            | AnalyzeCommands::LintHotspot { output, .. } => output,
            _ => &None,
        };

        // Format and output
        let output_str = match result {
            serde_json::Value::String(s) => s,
            other => serde_json::to_string_pretty(&other)?,
        };

        if let Some(path) = output_path {
            std::fs::write(path, output_str)?;
            println!("Results written to: {}", path.display());
        } else {
            println!("{}", output_str);
        }

        Ok(())
    }
}

/// Extension trait to enable downcasting (simplified for now)
trait AsAny {
    fn as_any(&self) -> &dyn std::any::Any;
}

// Note: This is a simplified implementation - proper downcasting would need more work
impl AsAny for Box<dyn ContractValidation> {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
