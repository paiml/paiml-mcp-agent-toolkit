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
                contracts,
                contract_gaps,
            } => {
                // Delegate to pv query for contract searches
                if contracts {
                    return handle_pv_query_delegation(&query, limit, &format);
                }
                // Contract gaps: show functions without bindings
                if contract_gaps {
                    return handle_contract_gaps(&project_path, limit, &format);
                }
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

                // Support `.schema` and `.tables` dot-commands
                if sql.eq_ignore_ascii_case(".schema") {
                    return sql_handler::handle_schema(&db_path);
                }
                if sql.eq_ignore_ascii_case(".tables") {
                    return sql_handler::handle_sql(
                        "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name",
                        sql_handler::SqlOutputFormat::Table,
                        &db_path,
                    );
                }

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
                record,
                from_stdin,
                dry_run,
            } => {
                if record {
                    crate::cli::command_dispatcher::test_record::execute_test_record(from_stdin, dry_run).await
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
                eprintln!("No checks matching '{pat}'. Run `pmat explain` to list all.");
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

/// Delegate to `pv query` for cross-project contract search.
/// pv-compatibility spec §2.6: pv query integration.
pub(crate) fn handle_pv_query_delegation(
    query: &str,
    limit: usize,
    format: &crate::cli::QueryOutputFormat,
) -> anyhow::Result<()> {
    let format_arg = match format {
        crate::cli::QueryOutputFormat::Json => "json",
        _ => "text",
    };
    // Find provable-contracts sibling directory
    let pv_dir = std::fs::canonicalize(".")
        .ok()
        .and_then(|p| p.parent().map(|pp| pp.join("provable-contracts")))
        .filter(|p| p.exists());

    if pv_dir.is_none() {
        eprintln!("error: ../provable-contracts/ directory not found.");
        eprintln!("  pmat query --contracts requires a provable-contracts sibling repo.");
        eprintln!("  Clone it: git clone https://github.com/paiml/provable-contracts ../provable-contracts");
        std::process::exit(1);
    }

    let mut cmd = std::process::Command::new("pv");
    cmd.args(["query", query, "--limit", &limit.to_string(), "-f", format_arg]);
    cmd.current_dir(pv_dir.as_ref().expect("checked above"));
    let output = cmd
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status();

    match output {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => {
            std::process::exit(status.code().unwrap_or(1));
        }
        Err(_) => {
            eprintln!("error: `pv` CLI not found. Install with:");
            eprintln!("  cargo install --path ../provable-contracts/crates/provable-contracts-cli");
            std::process::exit(1);
        }
    }
}

/// Show functions without contract bindings, ranked by importance.
/// Uses ContractIndex from .pmat/binding-index.json + function index.
pub(crate) fn handle_contract_gaps(
    project_path: &std::path::Path,
    limit: usize,
    format: &crate::cli::QueryOutputFormat,
) -> anyhow::Result<()> {
    use crate::services::contract_index::ContractIndex;

    let idx = ContractIndex::load(project_path);
    if idx.is_none() {
        eprintln!("No .pmat/binding-index.json found. Run: pmat comply refresh-bindings");
        std::process::exit(1);
    }
    let idx = idx.unwrap();

    // Load function index to get all source files
    let pmat_idx_path = project_path.join(".pmat/context.db");
    let legacy_idx = project_path.join(".pmat/context.idx");

    // Collect source files from the project
    let src_dir = project_path.join("src");
    let mut all_files: Vec<String> = Vec::new();
    if src_dir.exists() {
        collect_rs_files(&src_dir, project_path, &mut all_files);
    }

    let gaps = idx.find_gaps(&all_files);
    let bound_count = all_files.len() - gaps.len();

    if matches!(format, crate::cli::QueryOutputFormat::Json) {
        let json = serde_json::json!({
            "total_files": all_files.len(),
            "bound_files": bound_count,
            "gap_files": gaps.len(),
            "gaps": gaps.iter().take(limit).collect::<Vec<_>>(),
        });
        println!("{}", serde_json::to_string_pretty(&json)?);
    } else {
        println!(
            "Contract gaps: {}/{} source file(s) lack bindings\n",
            gaps.len(),
            all_files.len()
        );
        if idx.total_bindings > 0 {
            let pct = bound_count as f64 / all_files.len().max(1) as f64 * 100.0;
            println!(
                "Coverage: {:.1}% ({} bound, {} total bindings)\n",
                pct, bound_count, idx.total_bindings
            );
        }
        for (i, gap) in gaps.iter().enumerate().take(limit) {
            println!("  {}. {}", i + 1, gap);
        }
        if gaps.len() > limit {
            println!("  ... and {} more", gaps.len() - limit);
        }
        let _ = (pmat_idx_path, legacy_idx); // suppress unused warnings
    }

    Ok(())
}

/// Recursively collect .rs files relative to project root.
fn collect_rs_files(dir: &std::path::Path, root: &std::path::Path, out: &mut Vec<String>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            // Skip test directories and target
            let name = path.file_name().unwrap_or_default().to_str().unwrap_or("");
            if name == "target" || name == ".git" {
                continue;
            }
            collect_rs_files(&path, root, out);
        } else if path.extension().map_or(false, |e| e == "rs") {
            if let Ok(rel) = path.strip_prefix(root) {
                out.push(rel.to_string_lossy().to_string());
            }
        }
    }
}
