#![cfg_attr(coverage_nightly, coverage(off))]
//! Maintain subcommand execution

use super::super::CommandExecutor;
use anyhow::Result;

impl CommandExecutor {
    /// Execute maintain subcommands
    pub(super) async fn execute_maintain(
        command: crate::cli::commands::MaintainCommands,
    ) -> Result<()> {
        use crate::cli::commands::MaintainCommands;
        match command {
            MaintainCommands::Roadmap {
                roadmap,
                tickets_dir,
                validate,
                health,
                fix,
                generate_tickets,
                dry_run,
                format,
            } => {
                let config = crate::cli::handlers::roadmap_handler::RoadmapMaintenanceConfig::new(
                    validate,
                    health,
                    fix,
                    generate_tickets,
                    dry_run,
                );
                crate::cli::handlers::handle_maintain_roadmap(roadmap, tickets_dir, config, format)
                    .await
            }
            MaintainCommands::Health {
                project_dir,
                format,
                quick,
                all,
                check_build,
                check_tests,
                check_coverage,
                check_complexity,
                check_satd,
            } => {
                let config = crate::cli::handlers::health_handler::HealthCheckConfig::new(
                    quick,
                    all,
                    check_build,
                    check_tests,
                    check_coverage,
                    check_complexity,
                    check_satd,
                );
                crate::cli::handlers::handle_maintain_health(project_dir, format, config).await
            }
            MaintainCommands::BugReport {
                title,
                dry_run,
                interactive,
                clear,
            } => {
                crate::cli::handlers::bug_report_handler::handle_bug_report(
                    title.as_deref(),
                    dry_run,
                    interactive,
                    clear,
                )
                .await
            }
            MaintainCommands::CleanupResources {
                project_dir,
                targets,
                execute,
                exclude,
                min_age_days,
                format,
            } => {
                crate::cli::handlers::cleanup_resources_handler::handle_cleanup_resources(
                    &project_dir,
                    &targets,
                    execute,
                    &exclude,
                    min_age_days,
                    format,
                )
                .await
            }
        }
    }
}
