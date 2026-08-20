impl CliAdapter {
    /// Toyota Way Extract Method: Structural analysis commands dispatch
    /// Handles graph and structural analysis: DAG, graph metrics, symbol table, name similarity
    fn dispatch_structural_analysis(
        analyze_cmd: &AnalyzeCommands,
    ) -> Result<(Method, String, Value, Option<OutputFormat>), ProtocolError> {
        match analyze_cmd {
            AnalyzeCommands::Dag {
                dag_type,
                path,
                project_path,
                output,
                max_depth,
                target_nodes,
                filter_external,
                show_complexity,
                include_duplicates,
                include_dead_code,
                enhanced,
            } => {
                let resolved = project_path.as_deref().unwrap_or(path);
                Self::decode_analyze_dag(
                dag_type,
                resolved,
                output,
                max_depth,
                target_nodes,
                *filter_external,
                *show_complexity,
                *include_duplicates,
                *include_dead_code,
                *enhanced,
            )
            }
            AnalyzeCommands::GraphMetrics {
                path,
                project_path,
                metrics,
                pagerank_seeds,
                damping_factor,
                max_iterations,
                convergence_threshold,
                format,
                include,
                exclude,
                output,
                export_graphml,
                perf,
                top_k,
                min_centrality,
            } => {
                let resolved = project_path.as_deref().unwrap_or(path);
                Self::decode_analyze_graph_metrics(
                resolved,
                metrics,
                pagerank_seeds,
                damping_factor,
                max_iterations,
                convergence_threshold,
                format,
                include,
                exclude,
                output,
                export_graphml,
                perf,
                top_k,
                min_centrality,
            )
            }
            AnalyzeCommands::SymbolTable {
                path,
                project_path,
                format,
                query,
                filter,
                include,
                exclude,
                show_unreferenced,
                show_references,
                output,
                perf,
                top_files,
            } => {
                let resolved = project_path.as_deref().unwrap_or(path);
                Self::decode_analyze_symbol_table(
                resolved,
                format,
                query,
                filter,
                include,
                exclude,
                show_unreferenced,
                show_references,
                output,
                perf,
                top_files,
            )
            }
            AnalyzeCommands::NameSimilarity {
                path,
                project_path,
                query,
                top_k,
                phonetic,
                scope,
                threshold,
                format,
                include,
                exclude,
                output,
                perf,
                fuzzy,
                case_sensitive,
            } => {
                let resolved = project_path.as_deref().unwrap_or(path);
                Self::decode_analyze_name_similarity(
                resolved,
                query,
                top_k,
                phonetic,
                scope,
                threshold,
                format,
                include,
                exclude,
                output,
                perf,
                fuzzy,
                case_sensitive,
            )
            }
            _ => Err(ProtocolError::UnsupportedProtocol(
                "Command not supported in structural analysis dispatch".to_string(),
            )),
        }
    }

    /// Toyota Way Extract Method: Specialized analysis commands dispatch
    /// Handles specialized analysis: makefile, provability, proof annotations, coverage, WebAssembly
    fn dispatch_specialized_analysis(
        analyze_cmd: &AnalyzeCommands,
    ) -> Result<(Method, String, Value, Option<OutputFormat>), ProtocolError> {
        match analyze_cmd {
            AnalyzeCommands::Makefile {
                path,
                rules,
                format,
                fix,
                gnu_version,
                top_files,
            } => Self::decode_analyze_makefile(path, rules, format, fix, gnu_version, top_files),
            AnalyzeCommands::Provability {
                path,
                project_path,
                functions,
                analysis_depth,
                format,
                high_confidence_only,
                include_evidence,
                output,
                top_files,
            } => {
                let resolved = project_path.as_deref().unwrap_or(path);
                Self::decode_analyze_provability(
                resolved,
                functions,
                *analysis_depth,
                format,
                *high_confidence_only,
                *include_evidence,
                output,
                *top_files,
            )
            }
            AnalyzeCommands::ProofAnnotations {
                path,
                project_path,
                format,
                high_confidence_only,
                include_evidence,
                property_type,
                verification_method,
                output,
                perf,
                clear_cache,
                top_files,
            } => {
                let resolved = project_path.as_deref().unwrap_or(path);
                Self::decode_analyze_proof_annotations(
                resolved,
                format,
                high_confidence_only,
                include_evidence,
                property_type,
                verification_method,
                output,
                perf,
                clear_cache,
                top_files,
            )
            }
            AnalyzeCommands::IncrementalCoverage {
                path,
                project_path,
                base_branch,
                target_branch,
                format,
                coverage_threshold,
                changed_files_only,
                detailed,
                output,
                perf,
                cache_dir,
                force_refresh,
                top_files,
            } => {
                let resolved = project_path.as_deref().unwrap_or(path);
                Self::decode_analyze_incremental_coverage(
                resolved,
                base_branch,
                target_branch,
                format,
                coverage_threshold,
                changed_files_only,
                detailed,
                output,
                perf,
                cache_dir,
                force_refresh,
                top_files,
            )
            }
            AnalyzeCommands::AssemblyScript { .. } => {
                Self::decode_analyze_assemblyscript()
            }
            AnalyzeCommands::WebAssembly { .. } => Self::decode_analyze_webassembly(),
            _ => Err(ProtocolError::UnsupportedProtocol(
                "Command not supported in specialized analysis dispatch".to_string(),
            )),
        }
    }
}
