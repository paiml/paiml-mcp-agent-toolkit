//! Scaffold Command Handlers for CommandDispatcher
//!
//! Extracted from command_dispatcher mod.rs for file health compliance (CB-040).
//! Contains scaffold command execution and helper functions.
#![cfg_attr(coverage_nightly, coverage(off))]

use super::CommandDispatcher;
use crate::cli::commands::ScaffoldCommands;
use crate::cli::handlers;
use crate::stateless_server::StatelessTemplateServer;
use std::path::PathBuf;
use std::sync::Arc;

impl CommandDispatcher {
    /// Execute scaffold commands using handler pattern (reduces complexity)
    pub(crate) async fn execute_scaffold_command(
        command: ScaffoldCommands,
        server: Arc<StatelessTemplateServer>,
    ) -> anyhow::Result<()> {
        match command {
            ScaffoldCommands::Project {
                toolchain,
                templates,
                params,
                parallel,
            } => handlers::handle_scaffold(server, toolchain, templates, params, parallel).await,
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
                Self::execute_scaffold_agent_command(
                    name,
                    template,
                    features,
                    quality,
                    output,
                    force,
                    dry_run,
                    interactive,
                    deterministic_core.is_some(),
                    probabilistic_wrapper.is_some(),
                )
                .await
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
                // TICKET-PMAT-5031: WASM scaffolding
                let params = handlers::ScaffoldWasmParams {
                    name,
                    framework,
                    features,
                    quality,
                    output,
                    force,
                    dry_run,
                };
                handlers::handle_scaffold_wasm(params).await
            }
            ScaffoldCommands::ListTemplates => handlers::handle_list_agent_templates().await,
            ScaffoldCommands::ValidateTemplate { path } => {
                handlers::handle_validate_agent_template(path).await
            }
            ScaffoldCommands::ListSubagents { all } => {
                handlers::subagent_handlers::list_subagents(all)
            }
            ScaffoldCommands::CreateSubagent { agent_name, output } => {
                handlers::subagent_handlers::create_subagent(&agent_name, output)
            }
            ScaffoldCommands::CreateAllSubagents { output } => {
                handlers::subagent_handlers::create_all_mvp_subagents(output)
            }
            ScaffoldCommands::ValidateSubagent { file_path } => {
                handlers::subagent_handlers::validate_subagent(&file_path)
            }
            ScaffoldCommands::ShowToolMapping { agent } => {
                handlers::subagent_handlers::show_tool_mapping(agent)
            }
            ScaffoldCommands::ExportToolMapping { output } => {
                handlers::subagent_handlers::export_tool_mapping_json(&output)
            }
        }
    }

    /// Execute scaffold agent command (extracted for complexity reduction)
    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn execute_scaffold_agent_command(
        name: String,
        template: String,
        features: Vec<String>,
        quality: String,
        output: Option<PathBuf>,
        force: bool,
        dry_run: bool,
        interactive: bool,
        deterministic_core: bool,
        probabilistic_wrapper: bool,
    ) -> anyhow::Result<()> {
        let params = handlers::generation_handlers::ScaffoldAgentParams {
            name,
            template,
            features,
            quality,
            output,
            force,
            dry_run,
            interactive,
            deterministic_core: if deterministic_core {
                Some("true".to_string())
            } else {
                None
            },
            probabilistic_wrapper: if probabilistic_wrapper {
                Some("true".to_string())
            } else {
                None
            },
        };
        handlers::handle_scaffold_agent(params).await
    }
}
