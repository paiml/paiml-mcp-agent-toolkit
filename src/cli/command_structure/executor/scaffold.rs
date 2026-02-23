#![cfg_attr(coverage_nightly, coverage(off))]
//! Scaffold subcommand execution

use super::super::CommandExecutor;
use crate::cli::commands::ScaffoldCommands;
use anyhow::Result;

impl CommandExecutor {
    /// Execute scaffold subcommands
    pub(super) async fn execute_scaffold(&self, command: ScaffoldCommands) -> Result<()> {
        match command {
            ScaffoldCommands::Project {
                toolchain,
                templates,
                params,
                parallel,
            } => {
                self.registry
                    .generate_handlers
                    .handle_scaffold(self.server.clone(), toolchain, templates, params, parallel)
                    .await
            }
            ScaffoldCommands::Agent {
                name,
                template,
                features,
                quality,
                output,
                force,
                dry_run,
                interactive,
                deterministic_core,
                probabilistic_wrapper,
            } => {
                let params = crate::cli::handlers::ScaffoldAgentParams {
                    name,
                    template,
                    features,
                    quality,
                    output,
                    force,
                    dry_run,
                    interactive,
                    deterministic_core,
                    probabilistic_wrapper,
                };
                crate::cli::handlers::handle_scaffold_agent(params).await
            }
            ScaffoldCommands::Wasm {
                name,
                framework,
                features,
                quality,
                output,
                force,
                dry_run,
            } => {
                let params = crate::cli::handlers::ScaffoldWasmParams {
                    name,
                    framework,
                    features,
                    quality,
                    output,
                    force,
                    dry_run,
                };
                crate::cli::handlers::handle_scaffold_wasm(params).await
            }
            ScaffoldCommands::ListTemplates => {
                crate::cli::handlers::handle_list_agent_templates().await
            }
            ScaffoldCommands::ValidateTemplate { path } => {
                crate::cli::handlers::handle_validate_agent_template(path).await
            }
            ScaffoldCommands::ListSubagents { all } => {
                crate::cli::handlers::subagent_handlers::list_subagents(all)
            }
            ScaffoldCommands::CreateSubagent { agent_name, output } => {
                crate::cli::handlers::subagent_handlers::create_subagent(&agent_name, output)
            }
            ScaffoldCommands::CreateAllSubagents { output } => {
                crate::cli::handlers::subagent_handlers::create_all_mvp_subagents(output)
            }
            ScaffoldCommands::ValidateSubagent { file_path } => {
                crate::cli::handlers::subagent_handlers::validate_subagent(&file_path)
            }
            ScaffoldCommands::ShowToolMapping { agent } => {
                crate::cli::handlers::subagent_handlers::show_tool_mapping(agent)
            }
            ScaffoldCommands::ExportToolMapping { output } => {
                crate::cli::handlers::subagent_handlers::export_tool_mapping_json(&output)
            }
        }
    }
}
