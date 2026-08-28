//! Command routing: main dispatch table for all CLI commands.
//!
//! Routes each `Commands` variant to its appropriate handler.
#![cfg_attr(coverage_nightly, coverage(off))]

use super::CommandDispatcher;
use crate::cli::commands::Commands;
use crate::cli::handlers;
use crate::stateless_server::StatelessTemplateServer;
use std::sync::Arc;

impl CommandDispatcher {
    /// Route commands to appropriate handlers (reduces complexity)
    pub(super) async fn route_command(
        command: Commands,
        server: Arc<StatelessTemplateServer>,
    ) -> anyhow::Result<()> {
        match command {
            Commands::Generate {
                category,
                template,
                params,
                output,
                create_dirs,
            } => {
                handlers::handle_generate(server, category, template, params, output, create_dirs)
                    .await
            }
            Commands::Scaffold { command } => Self::execute_scaffold_command(command, server).await,
            Commands::List {
                toolchain,
                category,
                format,
            } => handlers::handle_list(server, toolchain, category, format).await,
            Commands::Search {
                query,
                toolchain,
                limit,
            } => handlers::handle_search(server, query, toolchain, limit).await,
            Commands::Validate { uri, params } => {
                handlers::handle_validate(server, uri, params).await
            }
            Commands::Context {
                toolchain,
                project_path,
                output,
                format,
                include_large_files,
                skip_expensive_metrics,
                language,
                languages,
            } => {
                handlers::handle_context(
                    toolchain,
                    project_path,
                    output,
                    format,
                    include_large_files,
                    skip_expensive_metrics,
                    language,
                    languages,
                )
                .await
            }
            Commands::Mcp(cmd) => handlers::handle_mcp_command(cmd, std::path::Path::new("")).await,
            Commands::Agy(cmd) => {
                handlers::handle_agy_command(&cmd, std::path::Path::new("")).await
            }
            Commands::Init {
                target,
                path,
                force,
                format,
            } => handlers::init_handler::handle_init(target, &path, force, format),
            cmd @ Commands::Query { .. } => Self::route_query_command(cmd).await,
            Commands::Sql {
                query,
                format,
                workspace,
                schema,
                examples,
                path,
            } => {
                use crate::cli::handlers::sql_handler::{self, SqlAction};

                // The five-way decision lives in `sql_handler::plan`, which is
                // pure and tested; this arm only supplies the database.
                // `--examples` is dispatched first because it must not need
                // one — `find_db_path` fails in a project with no index.
                match sql_handler::plan(query.as_deref(), &format, schema, examples) {
                    SqlAction::Examples => {
                        sql_handler::handle_examples();
                        Ok(())
                    }
                    action => {
                        sql_handler::run(action, &sql_handler::find_db_path(&path, workspace)?)
                    }
                }
            }
            Commands::Analyze(analyze_cmd) => Self::execute_analyze_command(analyze_cmd).await,
            Commands::Qdd(qdd_cmd) => Self::execute_qdd_command(qdd_cmd).await,
            Commands::Embed(embed_cmd) => Self::execute_embed_command(embed_cmd).await,
            Commands::Semantic(semantic_cmd) => Self::execute_semantic_command(semantic_cmd).await,
            #[cfg(feature = "demo")]
            Commands::Demo {
                path,
                url,
                repo,
                format,
                protocol,
                show_api,
                no_browser,
                port,
                cli,
                target_nodes,
                centrality_threshold,
                merge_threshold,
                debug,
                debug_output,
                skip_vendor,
                no_skip_vendor,
                max_line_length,
            } => {
                Self::execute_demo_command(
                    path,
                    url,
                    repo,
                    Some(format),
                    protocol,
                    show_api,
                    no_browser,
                    port.unwrap_or(8080),
                    cli,
                    Some(target_nodes),
                    Some(centrality_threshold),
                    Some(merge_threshold as f64),
                    debug,
                    debug_output,
                    skip_vendor,
                    no_skip_vendor,
                    max_line_length,
                    server,
                )
                .await
            }
            #[cfg(not(feature = "demo"))]
            Commands::Demo { .. } => {
                anyhow::bail!("Demo feature not enabled. Build with --features demo")
            }
            Commands::ValidateDocs(cmd) => {
                let exit_code = cmd.execute().await?;
                if exit_code != std::process::ExitCode::SUCCESS {
                    std::process::exit(1);
                }
                Ok(())
            }
            Commands::ValidateReadme(cmd) => {
                // Sprint 38: Hallucination detection (synchronous)
                let exit_code = cmd.execute()?;
                if exit_code != std::process::ExitCode::SUCCESS {
                    std::process::exit(1);
                }
                Ok(())
            }
            Commands::RedTeam(cmd) => {
                // Red Team Mode: Commit hallucination detection
                let exit_code = cmd.execute()?;
                if exit_code != std::process::ExitCode::SUCCESS {
                    std::process::exit(1);
                }
                Ok(())
            }
            Commands::Org(_org_cmd) => {
                #[cfg(feature = "org-intelligence")]
                {
                    handlers::handle_org_command(_org_cmd).await
                }
                #[cfg(not(feature = "org-intelligence"))]
                {
                    // `org analyze` is gone in EVERY build — the feature-enabled
                    // arm bails with the same removal notice. Sending a user off
                    // on a multi-minute rebuild to reach a second, different
                    // error is worse than telling them now. Only `org localize`
                    // is genuinely gated by the feature.
                    crate::cli::handlers::org_handlers::org_not_enabled_error(&_org_cmd)
                }
            }
            Commands::Prompt(prompt_cmd) => handlers::handle_prompt_command(prompt_cmd).await,
            Commands::Explain { pattern } => {
                handle_explain(pattern.as_deref());
                Ok(())
            }
            // Scoring and reporting commands delegated to reduce cognitive complexity
            cmd @ (Commands::QualityGate { .. }
            | Commands::Report { .. }
            | Commands::Score { .. }
            | Commands::RepoScore { .. }
            | Commands::RustProjectScore { .. }
            | Commands::BrickScore { .. }
            | Commands::PopperScore { .. }
            | Commands::DemoScore { .. }
            | Commands::InfraScore { .. }
            | Commands::Serve { .. }) => Self::route_scoring_command(cmd).await,
            Commands::CiLocal {
                path,
                quick,
                matrix,
                fix,
                format,
                verbose,
            } => {
                // `format` was bound as `_` here, so every advertised value of
                // --format printed the same human banner: `-f json` was not JSON.
                crate::cli::handlers::ci_local_handler::handle_ci_local(
                    &path,
                    quick,
                    matrix.as_deref(),
                    fix,
                    verbose,
                    format,
                )
                .await
            }
            Commands::Diagnose(args) => crate::cli::diagnose::handle_diagnose(args).await,
            Commands::Verify(args) => crate::cli::verify::handle_verify(args).await,
            Commands::Enforce(enforce_cmd) => handlers::route_enforce_command(enforce_cmd).await,
            Commands::Refactor(refactor_cmd) => Self::execute_refactor_command(refactor_cmd).await,
            Commands::Roadmap(roadmap_cmd) => Self::execute_roadmap_command(roadmap_cmd).await,
            Commands::Test {
                suite,
                iterations,
                memory,
                throughput,
                regression,
                timeout,
                output,
                perf,
                record,
                from_stdin,
                dry_run,
            } => {
                if record {
                    crate::cli::command_dispatcher::test_record::execute_test_record(
                        from_stdin, dry_run,
                    )
                    .await
                } else {
                    Self::execute_test_command(
                        suite, iterations, memory, throughput, regression, timeout, output, perf,
                    )
                    .await
                }
            }
            Commands::Memory { command } => Self::execute_memory_command(command).await,
            Commands::Cache { command } => Self::execute_cache_command(command).await,
            // Infrastructure and config commands delegated to reduce cognitive complexity
            cmd @ (Commands::Telemetry { .. }
            | Commands::Config { .. }
            | Commands::ShowMetrics { .. }
            | Commands::PredictQuality { .. }
            | Commands::RecordMetric { .. }
            | Commands::Agent { .. }
            | Commands::Tdg { .. }
            | Commands::QualityGates { .. }
            | Commands::Maintain { .. }
            | Commands::Hooks(..)
            | Commands::Debug { .. }) => Self::route_infra_command(cmd, server).await,

            #[cfg(feature = "mutation-testing")]
            Commands::Mutate(args) => handlers::mutate::handle(args, server).await,
            Commands::Work { command } => {
                // Issue #75: Unified GitHub/YAML workflow
                Self::execute_work_command(&command).await
            }
            Commands::Falsify {
                target,
                override_claims,
                ticket,
                path,
                format,
                failures_only,
                dry_run,
            } => {
                Self::execute_falsify_command(
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
            Commands::QaWork { command } => {
                // GH-102: Toyota Way quality validation
                handlers::qa_work_handler::handle_qa_work_command(command).await
            }
            Commands::Comply { command } => {
                // GH-96: PMAT compliance and migration system
                // Default to Check when no subcommand given (bare `pmat comply`)
                let cmd = command.unwrap_or(crate::cli::commands::ComplyCommands::Check {
                    path: std::path::PathBuf::from("."),
                    strict: false,
                    failures_only: false,
                    format: crate::cli::commands::ComplyOutputFormat::Text,
                    include_project: vec![],
                });
                handlers::comply_handlers::handle_comply_command(cmd).await
            }
            Commands::Extract { list } => {
                // GH-215: Extract function boundaries from a single file
                handlers::handle_extract_list(&list).await
            }
            Commands::Split {
                file,
                path,
                execute,
                format,
                output,
                min_cluster_lines,
                resolution,
                auto,
                max_lines,
                dry_run,
                commit,
            } => {
                if auto {
                    // --auto mode: scan project for oversized files
                    handlers::split_auto_handler::handle_split_auto(
                        &path,
                        max_lines,
                        file.as_deref(),
                        dry_run,
                        commit,
                    )
                    .await
                } else {
                    // Original single-file split mode
                    let file = file.ok_or_else(|| {
                        anyhow::anyhow!(
                            "FILE argument is required unless --auto is used.\n\
                             Usage: pmat split <FILE> or pmat split --auto"
                        )
                    })?;
                    let fmt = match format.as_str() {
                        "json" => handlers::split_handler::SplitOutputFormat::Json,
                        _ => handlers::split_handler::SplitOutputFormat::Text,
                    };
                    handlers::split_handler::handle_split(handlers::split_handler::SplitConfig {
                        file,
                        project_path: path,
                        execute,
                        format: fmt,
                        output,
                        min_cluster_lines,
                        resolution,
                    })
                    .await
                }
            }

            Commands::Stack { command } => {
                use crate::cli::commands::misc_commands::StackCommands;
                match command {
                    StackCommands::Status { format } => {
                        handlers::stack_sync_handler::handle_stack_status(&format).await
                    }
                    StackCommands::Sync { apply, dry_run } => {
                        handlers::stack_sync_handler::handle_stack_sync(apply, dry_run).await
                    }
                    StackCommands::Scaffold {
                        all,
                        template,
                        diff,
                        force,
                    } => {
                        handlers::stack_scaffold_handler::handle_stack_scaffold(
                            all, &template, diff, force,
                        )
                        .await
                    }
                }
            }

            // Quality and analysis commands delegated to reduce cognitive complexity
            cmd @ (Commands::ProjectDiag { .. }
            | Commands::TestDiscovery { .. }
            | Commands::TestStability { .. }
            | Commands::DebugFiveWhys { .. }
            | Commands::Oracle { .. }
            | Commands::PerfectionScore { .. }
            | Commands::Spec { .. }
            | Commands::Localize { .. }
            | Commands::CudaTdg { .. }
            | Commands::DepsAudit { .. }
            | Commands::Kaizen { .. }) => Self::route_quality_command(cmd).await,
        }
    }
}

/// Handle `pmat explain <PATTERN>` — look up check/metric explanations.
fn handle_explain(pattern: Option<&str>) {
    match pattern {
        Some(pat) => {
            let results = crate::explain::lookup(pat);
            if results.is_empty() {
                // A registered-ID miss and a typo are different failures; the
                // one message used to claim both were "no such check".
                eprintln!("{}", crate::explain::miss_message(pat));
                std::process::exit(1);
            }
            for (i, e) in results.iter().enumerate() {
                if i > 0 {
                    println!();
                }
                print!("{}", crate::explain::format_explanation(e));
            }
        }
        None => {
            println!("Available checks and metrics:\n");
            for (domain, checks) in crate::explain::list_all() {
                println!("{domain}:");
                for e in checks {
                    println!("  {:<10} {}", e.id, e.name);
                }
                println!();
            }
            println!("Usage: pmat explain <ID>  (e.g., pmat explain CB-1210)");
        }
    }
}

// All contract query handlers (handle_pv_query_delegation, handle_contract_gaps,
// handle_asset_contracts) extracted to contract_query_handlers.rs for CB-040.
