/// Route graph metrics analysis command
pub(super) async fn route_graph_metrics_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::GraphMetrics {
        path,
        project_path,
        metrics,
        pagerank_seeds,
        damping_factor,
        max_iterations,
        convergence_threshold,
        export_graphml,
        format,
        include,
        exclude,
        output,
        perf,
        top_k,
        min_centrality,
    } = cmd
    {
        let path = project_path.unwrap_or(path);
        crate::cli::handlers::advanced_analysis_handlers::handle_analyze_graph_metrics(
            path,
            metrics,
            pagerank_seeds,
            damping_factor,
            max_iterations,
            convergence_threshold,
            export_graphml,
            format,
            include,
            exclude,
            output,
            perf,
            top_k,
            min_centrality,
        )
        .await
    } else {
        unreachable!("Expected GraphMetrics command")
    }
}

/// Route name similarity analysis command
pub(super) async fn route_name_similarity_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::NameSimilarity {
        path,
        project_path,
        query,
        top_k,
        phonetic,
        scope,
        format,
        output,
        threshold,
        include,
        exclude,
        perf,
        fuzzy,
        case_sensitive,
    } = cmd
    {
        let path = project_path.unwrap_or(path);
        crate::cli::handlers::name_similarity_analysis::handle_analyze_name_similarity(
            path,
            query,
            top_k,
            phonetic,
            scope,
            f64::from(threshold),
            format,
            include,
            exclude,
            output,
            perf,
            fuzzy,
            case_sensitive,
        )
        .await
    } else {
        unreachable!("Expected NameSimilarity command")
    }
}

/// Route proof annotations analysis command
pub(super) async fn route_proof_annotations_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::ProofAnnotations {
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
    } = cmd
    {
        let path = project_path.unwrap_or(path);
        // `--top-files` was bound to `_top_files` and dropped here, and the
        // summary renderer hardcoded `.take(10)`, so `--top-files 1` and
        // `--top-files 50` both printed ten rows over an 84-file corpus.
        crate::cli::handlers::proof_annotations_handler::handle_analyze_proof_annotations(
            path,
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
        .await
    } else {
        unreachable!("Expected ProofAnnotations command")
    }
}

/// Route incremental coverage analysis command
pub(super) async fn route_incremental_coverage_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::IncrementalCoverage {
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
    } = cmd
    {
        use crate::cli::handlers::incremental_coverage_handler::IncrementalCoverageConfig;

        let path = project_path.unwrap_or(path);
        let config = IncrementalCoverageConfig {
            project_path: path,
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
        };

        crate::cli::handlers::incremental_coverage_handler::handle_analyze_incremental_coverage(
            config,
        )
        .await
    } else {
        unreachable!("Expected IncrementalCoverage command")
    }
}

/// Route symbol table analysis command
pub(super) async fn route_symbol_table_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::SymbolTable {
        path,
        project_path,
        format,
        filter,
        query,
        include,
        exclude,
        show_unreferenced,
        show_references,
        output,
        perf,
        top_files,
    } = cmd
    {
        let path = project_path.unwrap_or(path);
        // `--top-files` was bound to `_top_files` and thrown away, so the flag
        // documented as "Number of top files to show (0 = all)" changed nothing:
        // `-n 3`, `-n 10` and `-n 25` all produced the same 10-row tables.
        crate::cli::handlers::advanced_analysis_handlers::handle_analyze_symbol_table(
            path,
            format,
            filter,
            query,
            include,
            exclude,
            show_unreferenced,
            show_references,
            output,
            perf,
            top_files,
        )
        .await
    } else {
        unreachable!("Expected SymbolTable command")
    }
}

/// Route Big O analysis command
pub(super) async fn route_big_o_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::BigO {
        path,
        project_path,
        format,
        confidence_threshold,
        analyze_space,
        include,
        exclude,
        high_complexity_only,
        output,
        perf,
        top_files,
    } = cmd
    {
        let path = project_path.unwrap_or(path);
        crate::cli::handlers::big_o_handlers::handle_analyze_big_o(
            path,
            format,
            confidence_threshold,
            analyze_space,
            include,
            exclude,
            high_complexity_only,
            output,
            perf,
            top_files,
        )
        .await
    } else {
        unreachable!("Expected BigO command")
    }
}

/// Route `AssemblyScript` analysis command
pub(super) async fn route_assemblyscript_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::AssemblyScript {
        path,
        project_path,
        format,
        wasm_complexity,
        memory_analysis,
        security,
        output,
        timeout,
        perf,
        top_files,
    } = cmd
    {
        let path = project_path.unwrap_or(path);
        // Same one-line omission as the web-assembly route below, already fixed
        // for symbol-table above: `top_files: _top_files` threw the flag away.
        crate::cli::handlers::wasm_handlers::handle_analyze_assemblyscript(
            path,
            format,
            wasm_complexity,
            memory_analysis,
            security,
            output,
            timeout,
            perf,
            top_files,
        )
        .await
    } else {
        unreachable!("Expected AssemblyScript command")
    }
}

/// Route WebAssembly analysis command
pub(super) async fn route_webassembly_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::WebAssembly {
        path,
        project_path,
        format,
        include_binary,
        include_text,
        memory_analysis,
        security,
        complexity,
        output,
        perf,
        top_files,
    } = cmd
    {
        let path = project_path.unwrap_or(path);
        crate::cli::handlers::wasm_handlers::handle_analyze_webassembly(
            path,
            format,
            include_binary,
            include_text,
            memory_analysis,
            security,
            complexity,
            output,
            perf,
            top_files,
        )
        .await
    } else {
        unreachable!("Expected WebAssembly command")
    }
}

#[cfg(feature = "wasm-ast")]
pub(super) async fn route_wasm_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::Wasm {
        wasm_file,
        format,
        verify,
        security,
        profile,
        baseline,
        output,
        verbose,
    } = cmd
    {
        crate::cli::handlers::wasm_handler::handle_analyze_wasm(
            wasm_file, format, verify, security, profile, baseline, output, verbose,
        )
        .await
    } else {
        unreachable!("Expected Wasm command")
    }
}

/// Route Deep WASM analysis command
#[cfg(feature = "deep-wasm")]
pub(super) async fn route_deep_wasm_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::DeepWasm {
        source_path,
        wasm_file,
        dwarf_file,
        source_map,
        language,
        focus,
        format,
        output,
        strict,
        include_mir,
        include_llvm_ir,
        track_memory,
        detect_deadlocks,
    } = cmd
    {
        crate::cli::handlers::deep_wasm_handlers::handle_deep_wasm(
            crate::cli::handlers::deep_wasm_handlers::DeepWasmOptions {
                source_path,
                wasm_file,
                dwarf_file,
                source_map,
                language,
                focus,
                format,
                output,
                strict,
                _include_mir: include_mir,
                _include_llvm_ir: include_llvm_ir,
                _track_memory: track_memory,
                _detect_deadlocks: detect_deadlocks,
            },
        )
        .await
    } else {
        unreachable!("Expected DeepWasm command")
    }
}

/// Route Mutation Testing command (feature-gated)
#[cfg(feature = "mutation-testing")]
pub(super) async fn route_mutation_testing(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::Mutate {
        path,
        operators,
        ml_predict,
        distributed,
        workers,
        progress,
        min_score,
        ci_learning,
        ci_provider,
        auto_train_threshold,
        format,
        output,
    } = cmd
    {
        let config = crate::cli::handlers::mutation_handlers::MutationTestConfig::new(
            operators,
            ml_predict,
            distributed,
            workers,
            progress,
            min_score,
            ci_learning,
            ci_provider,
            auto_train_threshold,
        );
        crate::cli::handlers::mutation_handlers::handle_mutate(path, config, format, output).await
    } else {
        unreachable!("Expected Mutate command")
    }
}

/// Route Makefile analysis command
pub(super) async fn route_makefile_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::Makefile {
        path,
        rules,
        format,
        fix,
        gnu_version,
        top_files,
    } = cmd
    {
        crate::cli::handlers::advanced_analysis_handlers::handle_analyze_makefile(
            path,
            rules,
            format,
            fix,
            Some(gnu_version),
            top_files,
        )
        .await
    } else {
        unreachable!("Expected Makefile command")
    }
}
