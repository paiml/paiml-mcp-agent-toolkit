#![cfg_attr(coverage_nightly, coverage(off))]
//! Main command dispatch - routes Commands variants to their handlers

use super::super::CommandExecutor;
use crate::cli::Commands;
use anyhow::Result;

impl CommandExecutor {
    /// Execute a command using the modular dispatch architecture
    pub async fn execute(&self, command: Commands) -> Result<()> {
        match command {
            // Generation commands
            Commands::Generate {
                category,
                template,
                params,
                output,
                create_dirs,
            } => {
                self.registry
                    .generate_handlers
                    .handle_generate(
                        self.server.clone(),
                        category,
                        template,
                        params,
                        output,
                        create_dirs,
                    )
                    .await
            }
            Commands::Scaffold { command } => self.execute_scaffold(command).await,
            Commands::Validate { uri, params } => {
                self.registry
                    .generate_handlers
                    .handle_validate(self.server.clone(), uri, params)
                    .await
            }

            // Analysis commands
            Commands::Analyze(analyze_cmd) => {
                self.registry.analyze_handlers.execute(analyze_cmd).await
            }

            Commands::Qdd(qdd_cmd) => {
                use crate::cli::handlers::qdd_handlers;
                qdd_handlers::handle_qdd_command(qdd_cmd).await
            }

            // Utility commands
            Commands::List {
                toolchain,
                category,
                format,
            } => {
                self.registry
                    .utility_handlers
                    .handle_list(self.server.clone(), toolchain, category, format)
                    .await
            }
            Commands::Search {
                query,
                toolchain,
                limit,
            } => {
                self.registry
                    .utility_handlers
                    .handle_search(self.server.clone(), query, toolchain, limit)
                    .await
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
                self.registry
                    .utility_handlers
                    .handle_context(
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
            } => {
                if contracts {
                    return crate::cli::command_dispatcher::handle_pv_query_delegation(&query, limit, &format);
                }
                // Default is to show code; --summary disables it
                let show_code = !summary;
                let effective_docs = !no_docs;
                crate::cli::handlers::handle_query(
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
            Commands::Serve {
                port,
                host,
                cors,
                transport,
            } => {
                self.registry
                    .utility_handlers
                    .handle_serve(host, port, cors, transport)
                    .await
            }

            // Remaining commands delegated to extended dispatch
            other => self.execute_extended(other).await,
        }
    }
}
