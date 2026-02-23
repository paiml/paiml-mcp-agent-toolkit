#![cfg_attr(coverage_nightly, coverage(off))]
//! Forward commands to command_dispatcher.rs

use super::super::CommandExecutor;
use crate::cli::Commands;
use anyhow::Result;

impl CommandExecutor {
    /// Forward commands that should be handled by command_dispatcher.rs
    pub(super) fn forward_to_dispatcher(command: Commands) -> Result<()> {
        let name = match command {
            Commands::ShowMetrics { .. } => "ShowMetrics",
            Commands::PredictQuality { .. } => "PredictQuality",
            Commands::RecordMetric { .. } => "RecordMetric",
            Commands::Work { .. } => "Work",
            Commands::Falsify { .. } => "Falsify",
            Commands::QaWork { .. } => "QaWork",
            Commands::Comply { .. } => "Comply",
            Commands::ProjectDiag { .. } => "ProjectDiag",
            Commands::TestDiscovery { .. } => "TestDiscovery",
            Commands::DebugFiveWhys { .. } => "DebugFiveWhys",
            Commands::Localize { .. } => "Localize",
            Commands::Oracle { .. } => "Oracle",
            Commands::PerfectionScore { .. } => "PerfectionScore",
            Commands::Spec { .. } => "Spec",
            Commands::CudaTdg { .. } => "CudaTdg",
            Commands::DepsAudit { .. } => "DepsAudit",
            Commands::Kaizen { .. } => "Kaizen",
            Commands::Extract { .. } => "Extract",
            _ => "Unknown",
        };
        anyhow::bail!(
            "{} command should be handled by command_dispatcher.rs",
            name
        )
    }
}
