//! Work and Spec Command Handlers for CommandDispatcher
//!
//! Extracted from command_dispatcher.rs for file health compliance (CB-040).
//! Contains execute_work_command and handle_spec_command implementations.
#![cfg_attr(coverage_nightly, coverage(off))]

use super::CommandDispatcher;

impl CommandDispatcher {
    /// Execute work command (Issue #75: Unified GitHub/YAML workflow)
    pub(crate) async fn execute_work_command(
        command: &crate::cli::commands::WorkCommands,
    ) -> anyhow::Result<()> {
        use crate::cli::commands::WorkCommands;
        use crate::cli::handlers::work_handlers;

        match command {
            WorkCommands::Init {
                github_repo,
                no_github,
                path,
            } => {
                work_handlers::handle_work_init(github_repo.clone(), *no_github, path.clone()).await
            }
            WorkCommands::Start {
                id,
                with_spec,
                epic,
                path,
                create_github,
                profile,
                without,
                iteration,
            } => {
                work_handlers::handle_work_start(
                    id.clone(),
                    *with_spec,
                    *epic,
                    path.clone(),
                    *create_github,
                    profile.clone(),
                    without.clone().unwrap_or_default(),
                    *iteration,
                )
                .await
            }
            WorkCommands::Continue { id, path } => {
                work_handlers::handle_work_continue(id.clone(), path.clone()).await
            }
            WorkCommands::Checkpoint { id, path } => {
                work_handlers::handle_work_checkpoint(id.clone(), path.clone()).await
            }
            WorkCommands::Complete {
                id,
                skip_quality,
                override_claims,
                ticket,
                path,
            } => {
                work_handlers::handle_work_complete(
                    id.clone(),
                    *skip_quality,
                    override_claims.clone(),
                    ticket.clone(),
                    path.clone(),
                )
                .await
            }
            WorkCommands::Falsify {
                id,
                override_claims,
                ticket,
                path,
            } => {
                work_handlers::handle_work_falsify(
                    id.clone(),
                    override_claims.clone(),
                    ticket.clone(),
                    path.clone(),
                )
                .await
            }
            WorkCommands::Status { id, path, active } => {
                work_handlers::handle_work_status(id.clone(), path.clone(), *active).await
            }
            WorkCommands::Sync {
                direction,
                path,
                dry_run,
            } => work_handlers::handle_work_sync(*direction, path.clone(), *dry_run).await,
            WorkCommands::Validate { path, verbose, fix } => {
                work_handlers::handle_work_validate(path.clone(), *verbose, *fix).await
            }
            WorkCommands::Migrate {
                path,
                dry_run,
                backup,
            } => work_handlers::handle_work_migrate(path.clone(), *dry_run, *backup).await,
            WorkCommands::ListStatuses => work_handlers::handle_work_list_statuses().await,
            WorkCommands::Add {
                title,
                description,
                priority,
                tags,
                path,
                github,
            } => {
                work_handlers::handle_work_add(
                    title.clone(),
                    description.clone(),
                    *priority,
                    tags.clone(),
                    path.clone(),
                    *github,
                )
                .await
            }
            WorkCommands::List {
                status,
                priority,
                count,
                path,
            } => {
                work_handlers::handle_work_list(status.clone(), *priority, *count, path.clone())
                    .await
            }
            WorkCommands::Edit {
                id,
                title,
                description,
                priority,
                status,
                tags,
                path,
            } => {
                work_handlers::handle_work_edit(
                    id.clone(),
                    title.clone(),
                    description.clone(),
                    *priority,
                    status.clone(),
                    tags.clone(),
                    path.clone(),
                )
                .await
            }
            WorkCommands::Delete { id, force, path } => {
                work_handlers::handle_work_delete(id.clone(), *force, path.clone()).await
            }
            WorkCommands::Annotate {
                id,
                path,
                format,
                with_churn,
                churn_days,
            } => {
                work_handlers::handle_work_annotate(
                    id.clone(),
                    path.clone(),
                    *format,
                    *with_churn,
                    *churn_days,
                )
                .await
            }
            WorkCommands::Score {
                id,
                min_score,
                path,
                format,
            } => {
                work_handlers::handle_work_score(
                    id.clone(),
                    *min_score,
                    path.clone(),
                    format.clone(),
                )
                .await
            }
        }
    }

    /// Execute spec command (master-plan-pmat-work-system.md S-001 to S-010)
    pub(crate) async fn handle_spec_command(
        command: crate::cli::commands::SpecCommands,
    ) -> anyhow::Result<()> {
        use crate::cli::commands::SpecCommands;
        use crate::cli::handlers::spec_handlers;

        match command {
            SpecCommands::Score {
                spec,
                format,
                output,
                verbose,
            } => spec_handlers::handle_spec_score(&spec, format, output.as_deref(), verbose).await,
            SpecCommands::Comply {
                spec,
                dry_run,
                format,
            } => spec_handlers::handle_spec_comply(&spec, dry_run, format).await,
            SpecCommands::Create {
                name,
                issue,
                epic,
                output,
            } => {
                spec_handlers::handle_spec_create(
                    &name,
                    issue.as_deref(),
                    epic.as_deref(),
                    output.as_deref(),
                )
                .await
            }
            SpecCommands::List {
                path,
                min_score,
                failing_only,
                format,
            } => spec_handlers::handle_spec_list(&path, min_score, failing_only, format).await,
            SpecCommands::Sync {
                spec_path,
                roadmap_path,
                dry_run,
                direction,
            } => {
                spec_handlers::handle_spec_sync(&spec_path, &roadmap_path, dry_run, direction).await
            }
            SpecCommands::Drift {
                spec_path,
                roadmap_path,
                format,
            } => spec_handlers::handle_spec_drift(&spec_path, &roadmap_path, format).await,
        }
    }

    /// Execute top-level falsify command — routes to spec or work item falsification
    pub(crate) async fn execute_falsify_command(
        target: String,
        override_claims: Option<Vec<String>>,
        ticket: Option<String>,
        path: Option<std::path::PathBuf>,
        format: Option<String>,
        failures_only: bool,
        dry_run: bool,
    ) -> anyhow::Result<()> {
        crate::cli::handlers::spec_falsify_handler::handle_falsify(
            target,
            override_claims,
            ticket,
            path,
            format,
            failures_only,
            dry_run,
        )
        .await
    }
}
