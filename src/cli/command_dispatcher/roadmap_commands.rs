//! Roadmap Command Handlers for CommandDispatcher
//!
//! Extracted from command_dispatcher mod.rs for file health compliance (CB-040).
//! Contains roadmap command execution.

#![cfg_attr(coverage_nightly, coverage(off))]

use super::CommandDispatcher;
use crate::cli::commands::RoadmapCommands;

impl CommandDispatcher {
    /// Execute roadmap commands using handler pattern (reduces CC)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn execute_roadmap_command(roadmap_cmd: RoadmapCommands) -> anyhow::Result<()> {
        use crate::roadmap::{self, RoadmapConfig};
        use std::path::PathBuf;

        // MACS-013: `pmat roadmap sync` renders ROADMAP.yaml directly and
        // returns early — it does not use the sprint-config pipeline below.
        if let RoadmapCommands::Sync {
            gh_snapshot,
            dry_run,
            path,
        } = &roadmap_cmd
        {
            let generated_at = chrono::Utc::now().to_rfc3339();
            return crate::roadmap::sync::handle_roadmap_sync(
                path,
                gh_snapshot.clone(),
                *dry_run,
                &generated_at,
            );
        }

        // Load configuration (with defaults)
        let config = RoadmapConfig {
            path: PathBuf::from("docs/execution/roadmap.md"),
            quality_gates: roadmap::QualityGateConfig {
                complexity_max: 20,
                coverage_min: 80,
                satd_tolerance: 0,
                documentation_required: true,
                lint_compliance: true,
            },
            enforce_quality_gates: true,
            git: roadmap::GitConfig {
                create_branches: false, // DISABLED: per CLAUDE.md zero-branching policy
                branch_pattern: "feature/{task_id}".to_string(),
                commit_pattern: "{task_id}: {message}".to_string(),
                require_quality_check: true,
            },
            enabled: true,
            auto_generate_todos: true,
            require_task_ids: true,
            task_id_pattern: "PMAT-[0-9]{4}".to_string(),
            tracking: roadmap::TrackingConfig {
                velocity_tracking: true,
                burndown_charts: true,
                quality_metrics: true,
                export_format: "json".to_string(),
            },
        };

        // Create command struct and execute
        let cmd = roadmap::commands::RoadmapCommand {
            command: match roadmap_cmd {
                RoadmapCommands::Init {
                    version,
                    title,
                    duration_days,
                    priority,
                } => roadmap::commands::RoadmapSubcommand::Init {
                    version,
                    title,
                    duration_days,
                    priority,
                },
                RoadmapCommands::Todos {
                    sprint,
                    output,
                    include_quality_gates,
                } => roadmap::commands::RoadmapSubcommand::Todos {
                    sprint,
                    output,
                    include_quality_gates,
                },
                RoadmapCommands::Start {
                    task_id,
                    create_branch,
                } => roadmap::commands::RoadmapSubcommand::Start {
                    task_id,
                    create_branch,
                },
                RoadmapCommands::Complete {
                    task_id,
                    skip_quality_check,
                } => roadmap::commands::RoadmapSubcommand::Complete {
                    task_id,
                    skip_quality_check,
                },
                RoadmapCommands::Status {
                    sprint,
                    task,
                    format,
                } => {
                    let output_format = match format {
                        crate::cli::OutputFormat::Json => crate::cli::OutputFormat::Json,
                        _ => crate::cli::OutputFormat::Table,
                    };
                    roadmap::commands::RoadmapSubcommand::Status {
                        sprint,
                        task,
                        format: output_format,
                    }
                }
                RoadmapCommands::Validate { sprint, strict } => {
                    roadmap::commands::RoadmapSubcommand::Validate { sprint, strict }
                }
                RoadmapCommands::QualityCheck { task_id } => {
                    roadmap::commands::RoadmapSubcommand::QualityCheck { task_id }
                }
                // MACS-013: handled by the early return above; unreachable here.
                RoadmapCommands::Sync { .. } => unreachable!("Sync handled before config pipeline"),
            },
        };

        roadmap::commands::execute(cmd, config).await
    }
}
