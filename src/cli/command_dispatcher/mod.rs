//! Command Dispatcher - Reduces CLI complexity through handler pattern
//!
//! This module implements a dispatch table pattern to reduce cyclomatic complexity
//! in the CLI module by delegating command execution to specialized handlers.
#![cfg_attr(coverage_nightly, coverage(off))]

use super::commands::QddCommands;
use super::{AnalyzeCommands, Commands, RefactorCommands};
use crate::cli::handlers;
use crate::cli::handlers::cache::CacheCommand;
use crate::cli::handlers::memory::MemoryCommand;
use crate::stateless_server::StatelessTemplateServer;
use std::sync::Arc;

// Extracted modules for CB-040 file health compliance
mod config_commands;
#[cfg(feature = "demo")]
mod demo_commands;
mod metrics_commands;
mod quality_commands;
mod roadmap_commands;
mod scaffold_commands;
mod semantic_commands;
mod test_commands;

// Work and spec handlers extracted for file health compliance (CB-040)
#[path = "command_dispatcher_work.rs"]
mod command_dispatcher_work;

// Scoring and infrastructure handlers extracted for cognitive complexity reduction
#[path = "command_dispatcher_scoring.rs"]
mod command_dispatcher_scoring;

/// Trait for command handlers to reduce complexity through delegation
#[allow(dead_code)]
#[allow(async_fn_in_trait)]
pub trait CommandHandler: Send + Sync {
    async fn execute(&self, server: Arc<StatelessTemplateServer>) -> anyhow::Result<()>;
}

/// Trait for analyze command handlers
#[allow(dead_code)]
#[allow(async_fn_in_trait)]
pub trait AnalyzeCommandHandler: Send + Sync {
    async fn execute(&self) -> anyhow::Result<()>;
}

/// Command dispatcher that reduces complexity by delegating to handlers
pub struct CommandDispatcher;

impl CommandDispatcher {
    /// Execute a command using the handler pattern (reduces CC from dispatch match)
    pub async fn execute_command(
        command: Commands,
        server: Arc<StatelessTemplateServer>,
    ) -> anyhow::Result<()> {
        Self::route_command(command, server).await
    }

    /// Route commands to appropriate handlers (reduces complexity)
    async fn route_command(
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
            Commands::Query {
                query,
                limit,
                min_grade,
                max_complexity,
                language,
                path,
                project_path,
                format,
                include_source,
                rebuild_index,
                exclude_tests,
                rank_by,
                min_pagerank,
                include_project,
                churn,
                duplicates,
                entropy,
                faults,
                coverage,
                uncovered_only,
                coverage_diff,
                coverage_file,
                coverage_gaps,
                include_excluded,
                definition_type,
                summary,
                git_history,
                regex,
                literal,
                raw,
                case_sensitive,
                ignore_case,
                exclude,
                exclude_file,
                files_with_matches,
                count,
                after_context,
                before_context,
                context_lines,
                ptx_flow,
                ptx_diagnostics,
                suggest_rename,
                apply,
                no_docs,
                docs_only,
            } => {
                // Default is to show code; --summary disables it
                let show_code = !summary;
                let effective_docs = !no_docs;
                handlers::handle_query(
                    query,
                    limit,
                    min_grade,
                    max_complexity,
                    language,
                    path,
                    project_path,
                    format,
                    include_source,
                    rebuild_index,
                    exclude_tests,
                    rank_by,
                    min_pagerank,
                    include_project,
                    churn,
                    duplicates,
                    entropy,
                    faults,
                    coverage,
                    uncovered_only,
                    coverage_diff,
                    coverage_file,
                    coverage_gaps,
                    include_excluded,
                    definition_type,
                    show_code,
                    git_history,
                    regex,
                    literal,
                    raw,
                    case_sensitive,
                    ignore_case,
                    exclude,
                    exclude_file,
                    files_with_matches,
                    count,
                    after_context,
                    before_context,
                    context_lines,
                    ptx_flow,
                    ptx_diagnostics,
                    suggest_rename,
                    apply,
                    effective_docs,
                    docs_only,
                )
                .await
            }
            Commands::Sql {
                query,
                format,
                workspace,
                schema,
                examples,
                path,
            } => {
                use crate::cli::handlers::sql_handler;

                if examples {
                    sql_handler::handle_examples();
                    return Ok(());
                }

                let db_path = sql_handler::find_db_path(&path, workspace)?;

                if schema {
                    return sql_handler::handle_schema(&db_path);
                }

                let sql = query.as_deref().unwrap_or("grade-dist");
                let fmt = sql_handler::SqlOutputFormat::from_str_opt(&format);
                sql_handler::handle_sql(sql, fmt, &db_path)
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
                    anyhow::bail!("Organizational intelligence feature is not enabled. Rebuild with --features org-intelligence")
                }
            }
            Commands::Prompt(prompt_cmd) => handlers::handle_prompt_command(prompt_cmd).await,
            // Scoring and reporting commands delegated to reduce cognitive complexity
            cmd @ (Commands::QualityGate { .. }
            | Commands::Report { .. }
            | Commands::RepoScore { .. }
            | Commands::RustProjectScore { .. }
            | Commands::BrickScore { .. }
            | Commands::PopperScore { .. }
            | Commands::DemoScore { .. }
            | Commands::Serve { .. }) => Self::route_scoring_command(cmd).await,
            Commands::Diagnose(args) => super::diagnose::handle_diagnose(args).await,
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
            } => {
                Self::execute_test_command(
                    suite, iterations, memory, throughput, regression, timeout, output, perf,
                )
                .await
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
                handlers::comply_handlers::handle_comply_command(command).await
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
            } => {
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

            // Quality and analysis commands delegated to reduce cognitive complexity
            cmd @ (Commands::ProjectDiag { .. }
            | Commands::TestDiscovery { .. }
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

    /// Execute analyze commands using handler pattern (reduces CC)
    pub async fn execute_analyze_command(analyze_cmd: AnalyzeCommands) -> anyhow::Result<()> {
        // Delegate to the modular analysis handlers
        super::handlers::route_analyze_command(analyze_cmd).await
    }

    /// Execute QDD commands using handler pattern (reduces CC)
    pub async fn execute_qdd_command(qdd_cmd: QddCommands) -> anyhow::Result<()> {
        // Delegate to the QDD handlers
        super::handlers::qdd_handlers::handle_qdd_command(qdd_cmd).await
    }

    /// Execute refactor commands using handler pattern (reduces CC)
    pub async fn execute_refactor_command(refactor_cmd: RefactorCommands) -> anyhow::Result<()> {
        // Delegate to the refactor handlers
        super::handlers::route_refactor_command(refactor_cmd).await
    }

    /// Execute memory management commands using handler pattern (reduces CC)
    pub async fn execute_memory_command(memory_cmd: MemoryCommand) -> anyhow::Result<()> {
        // Delegate to the memory handler
        super::handlers::handle_memory_command(&memory_cmd).await
    }

    /// Execute cache management commands using handler pattern (reduces CC)
    pub async fn execute_cache_command(cache_cmd: CacheCommand) -> anyhow::Result<()> {
        // Delegate to the cache handler
        super::handlers::handle_cache_command(&cache_cmd).await
    }

    /// Route quality and analysis commands (extracted to reduce route_command cognitive complexity)
    async fn route_quality_command(command: Commands) -> anyhow::Result<()> {
        match command {
            Commands::ProjectDiag {
                path,
                format,
                category,
                failures_only,
                output,
                quiet,
            } => {
                let config = handlers::project_diag_handlers::ProjectDiagConfig {
                    path,
                    format,
                    category,
                    failures_only,
                    output,
                    quiet,
                };
                handlers::project_diag_handlers::handle_project_diag(config).await
            }
            Commands::TestDiscovery { command } => {
                handlers::test_discovery_handlers::handle_test_discovery_command(command).await
            }
            Commands::DebugFiveWhys {
                issue,
                depth,
                format,
                output,
                path,
                context,
                auto_analyze,
            } => {
                crate::cli::handlers::five_whys_handlers::handle_debug(
                    &issue,
                    depth,
                    format,
                    output.as_deref(),
                    &path,
                    context.as_deref(),
                    auto_analyze,
                )
                .await
            }
            Commands::Oracle { command } => {
                crate::cli::handlers::oracle_handlers::handle_oracle_command(command).await
            }
            Commands::PerfectionScore {
                path,
                breakdown,
                target,
                format,
                output,
                fast,
            } => {
                handlers::perfection_score_handlers::handle_perfection_score(
                    &path,
                    breakdown,
                    target,
                    format,
                    output.as_deref(),
                    fast,
                )
                .await
            }
            Commands::Spec { command } => Self::handle_spec_command(command).await,
            Commands::Localize {
                passed_coverage,
                failed_coverage,
                passed_count,
                failed_count,
                formula,
                top_n,
                output,
                format,
            } => {
                crate::cli::handlers::localize_handlers::handle_localize(
                    &passed_coverage,
                    &failed_coverage,
                    passed_count,
                    failed_count,
                    &formula,
                    top_n,
                    output.as_deref(),
                    &format,
                )
                .await
            }
            Commands::CudaTdg {
                path,
                command,
                format,
                min_score,
                fail_on_p0,
                simd,
                wgpu,
                output,
                quiet,
            } => {
                let config = handlers::CudaTdgCommandConfig {
                    path,
                    command,
                    format,
                    min_score,
                    fail_on_p0,
                    simd,
                    wgpu,
                    output,
                    quiet,
                };
                handlers::handle_cuda_tdg_command(config).await
            }
            Commands::DepsAudit {
                path,
                format,
                all,
                pareto,
                sort_by,
            } => handlers::deps_audit_handlers::handle_deps_audit(
                &path, &format, all, pareto, &sort_by,
            ),
            Commands::Kaizen {
                path,
                dry_run,
                no_commit,
                no_issues,
                push,
                agent,
                max_agents,
                format,
                output,
                skip_clippy,
                skip_fmt,
                skip_comply,
                skip_github,
                skip_defects,
            } => {
                let config = handlers::kaizen_handler::KaizenConfig {
                    path,
                    dry_run,
                    commit: !no_commit,
                    create_issues: !no_issues,
                    push,
                    auto_agent: agent,
                    max_agents,
                    format,
                    output,
                    skip_clippy,
                    skip_fmt,
                    skip_comply,
                    skip_github,
                    skip_defects,
                };
                handlers::kaizen_handler::handle_kaizen(config).await
            }
            _ => unreachable!("route_quality_command called with non-quality command"),
        }
    }
}

// Tests extracted for file health compliance (CB-040)
#[cfg(test)]
#[path = "tests.rs"]
mod tests;
