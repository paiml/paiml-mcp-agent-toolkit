// Command routing: main dispatch table for all CLI commands
//
// Routes each Commands variant to its appropriate handler.

impl CommandDispatcher {
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
                extract_candidates,
                max_module_lines,
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
                    extract_candidates,
                    max_module_lines,
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
            Commands::CiLocal {
                path,
                quick,
                matrix,
                fix,
                format: _,
                verbose,
            } => {
                crate::cli::handlers::ci_local_handler::handle_ci_local(
                    &path,
                    quick,
                    matrix.as_deref(),
                    fix,
                    verbose,
                )
                .await
            }
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
