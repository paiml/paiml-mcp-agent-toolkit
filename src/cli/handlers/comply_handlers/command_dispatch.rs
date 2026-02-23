// Command dispatch for comply subcommands
//
// This file is include!()'d into comply_handlers/mod.rs scope,
// where it has access to both check_handlers (re-exported) and migrate_handlers items.

/// Handle all comply subcommands
pub async fn handle_comply_command(command: ComplyCommands) -> Result<()> {
    match command {
        ComplyCommands::Check {
            path,
            strict,
            failures_only,
            format,
            include_project,
        } => {
            let result = handle_check(&path, strict, failures_only, format).await;
            if !include_project.is_empty() {
                if let Err(e) = check_file_health_multi(&path, &include_project) {
                    eprintln!("Cross-stack health check warning: {}", e);
                }
            }
            result
        }

        ComplyCommands::Migrate {
            path,
            version,
            dry_run,
            no_backup,
            force,
        } => handle_migrate(&path, version.as_deref(), dry_run, no_backup, force).await,

        ComplyCommands::Upgrade { path, target, dry_run } => {
            handle_upgrade(&path, &target, dry_run).await
        }

        ComplyCommands::Diff {
            path,
            from,
            to,
            breaking_only,
        } => handle_diff(&path, from.as_deref(), to.as_deref(), breaking_only).await,

        ComplyCommands::Update {
            path,
            hooks,
            config,
            dry_run,
        } => handle_update(&path, hooks, config, dry_run).await,

        ComplyCommands::Init { path, force } => handle_init(&path, force).await,

        ComplyCommands::Enforce {
            path,
            yes,
            disable,
            format,
        } => handle_enforce(&path, yes, disable, format).await,

        ComplyCommands::Report {
            path,
            include_history,
            format,
            output,
        } => handle_report(&path, include_history, format, output.as_deref()).await,

        ComplyCommands::Review {
            path,
            format,
            output,
        } => handle_review(&path, format, output.as_deref()).await,

        ComplyCommands::Audit {
            path,
            format,
            output,
        } => handle_audit(&path, format, output.as_deref()).await,

        ComplyCommands::Baseline { path } => {
            generate_file_health_baseline(&path)
        }

        ComplyCommands::CrossCrate {
            path,
            crates,
            similarity_threshold,
            churn_window_days,
            rules,
            format,
            output,
            strict,
            save_baseline,
        } => {
            cross_crate_handlers::handle_cross_crate(
                &path,
                crates.as_deref(),
                similarity_threshold,
                churn_window_days,
                rules.as_deref(),
                format,
                output.as_deref(),
                strict,
                save_baseline,
            )
            .await
        }
    }
}
