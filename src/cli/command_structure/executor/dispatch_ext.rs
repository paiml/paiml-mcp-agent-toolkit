#![cfg_attr(coverage_nightly, coverage(off))]
//! Extended command dispatch - routes to scoring, tools, and advanced sub-dispatchers

use super::super::CommandExecutor;
use crate::cli::Commands;
use anyhow::Result;

impl CommandExecutor {
    /// Execute extended commands (demo, quality, scoring, maintain, debug, etc.)
    pub(super) async fn execute_extended(&self, command: Commands) -> Result<()> {
        match command {
            // Demo, quality gate, reporting, and scoring commands
            cmd @ (Commands::Demo { .. }
            | Commands::QualityGate { .. }
            | Commands::Report { .. }
            | Commands::RepoScore { .. }
            | Commands::RustProjectScore { .. }
            | Commands::BrickScore { .. }
            | Commands::PopperScore { .. }
            | Commands::DemoScore { .. }) => self.execute_ext_scoring(cmd).await,

            // Infrastructure, tools, and configuration commands
            cmd @ (Commands::Diagnose(..)
            | Commands::Refactor(..)
            | Commands::Enforce(..)
            | Commands::Roadmap(..)
            | Commands::Test { .. }
            | Commands::Memory { .. }
            | Commands::Cache { .. }
            | Commands::Telemetry { .. }
            | Commands::Config { .. }
            | Commands::Agent { .. }
            | Commands::QualityGates { .. }
            | Commands::Maintain { .. }) => self.execute_ext_tools(cmd).await,

            // TDG, validation, semantic, and feature-gated commands
            other => self.execute_ext_advanced(other).await,
        }
    }
}
