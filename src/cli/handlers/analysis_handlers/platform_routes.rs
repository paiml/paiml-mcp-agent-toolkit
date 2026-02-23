//! Platform-specific and specialized analysis route handlers
//!
//! Handles: GraphMetrics, NameSimilarity, ProofAnnotations, IncrementalCoverage,
//! SymbolTable, BigO, AssemblyScript, WebAssembly, Wasm, DeepWasm, Mutation,
//! Makefile, Models (MLOps)

use crate::cli::{self, AnalyzeCommands};
use anyhow::Result;

/// Route graph metrics analysis command
pub(super) async fn route_graph_metrics_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::GraphMetrics {
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
        crate::cli::handlers::advanced_analysis_handlers::handle_analyze_graph_metrics(
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
        )
        .await
    } else {
        unreachable!("Expected GraphMetrics command")
    }
}

/// Route name similarity analysis command
pub(super) async fn route_name_similarity_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::NameSimilarity {
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
        crate::cli::handlers::name_similarity_analysis::handle_analyze_name_similarity(
            project_path,
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
        project_path,
        format,
        high_confidence_only,
        include_evidence,
        property_type,
        verification_method,
        output,
        perf,
        clear_cache,
        top_files: _top_files,
    } = cmd
    {
        crate::cli::handlers::proof_annotations_handler::handle_analyze_proof_annotations(
            project_path,
            format,
            high_confidence_only,
            include_evidence,
            property_type,
            verification_method,
            output,
            perf,
            clear_cache,
        )
        .await
    } else {
        unreachable!("Expected ProofAnnotations command")
    }
}

/// Route incremental coverage analysis command
pub(super) async fn route_incremental_coverage_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::IncrementalCoverage {
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

        let config = IncrementalCoverageConfig {
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
        top_files: _top_files,
    } = cmd
    {
        crate::cli::handlers::advanced_analysis_handlers::handle_analyze_symbol_table(
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
        )
        .await
    } else {
        unreachable!("Expected SymbolTable command")
    }
}

/// Route Big O analysis command
pub(super) async fn route_big_o_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::BigO {
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
        crate::cli::handlers::big_o_handlers::handle_analyze_big_o(
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
        )
        .await
    } else {
        unreachable!("Expected BigO command")
    }
}

/// Route `AssemblyScript` analysis command
pub(super) async fn route_assemblyscript_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::AssemblyScript {
        project_path,
        format,
        wasm_complexity,
        memory_analysis,
        security,
        output,
        timeout,
        perf,
        top_files: _top_files,
    } = cmd
    {
        crate::cli::handlers::wasm_handlers::handle_analyze_assemblyscript(
            project_path,
            format,
            wasm_complexity,
            memory_analysis,
            security,
            output,
            timeout,
            perf,
        )
        .await
    } else {
        unreachable!("Expected AssemblyScript command")
    }
}

/// Route WebAssembly analysis command
pub(super) async fn route_webassembly_analysis(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::WebAssembly {
        project_path,
        format,
        include_binary,
        include_text,
        memory_analysis,
        security,
        complexity,
        output,
        perf,
        top_files: _top_files,
    } = cmd
    {
        crate::cli::handlers::wasm_handlers::handle_analyze_webassembly(
            project_path,
            format,
            include_binary,
            include_text,
            memory_analysis,
            security,
            complexity,
            output,
            perf,
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

/// Route MLOps model analysis (PMAT-500)
pub(super) async fn route_model_analysis(cmd: AnalyzeCommands) -> Result<()> {
    use cli::AnalyzeCommands;

    if let AnalyzeCommands::Models {
        path,
        format,
        check,
    } = cmd
    {
        let project_path = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());

        let model_files =
            crate::cli::handlers::comply_cb_detect::walkdir_model_files(&project_path);

        if model_files.is_empty() {
            println!(
                "No model files found (*.gguf, *.apr, *.safetensors) in {}",
                project_path.display()
            );
            return Ok(());
        }

        // Detect Git LFS patterns
        let lfs_patterns = detect_lfs_patterns(&project_path);

        // Collect metadata for each model file
        let mut entries: Vec<ModelInventoryEntry> = Vec::new();
        let mut total_size: u64 = 0;

        for file_path in &model_files {
            let file_size = std::fs::metadata(file_path).map(|m| m.len()).unwrap_or(0);
            total_size += file_size;

            let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
            let format_name =
                crate::cli::handlers::comply_cb_detect::ModelFormat::from_extension(ext)
                    .map(|f| f.name())
                    .unwrap_or("Unknown");

            let rel = file_path
                .strip_prefix(&project_path)
                .unwrap_or(file_path)
                .display()
                .to_string();

            let filename = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            entries.push(ModelInventoryEntry {
                file: rel,
                format: format_name.to_string(),
                size_bytes: file_size,
                lfs_tracked: is_lfs_tracked(filename, &lfs_patterns),
            });
        }

        match format {
            cli::OutputFormat::Json => {
                print_model_inventory_json(&entries, total_size)?;
            }
            _ => {
                print_model_inventory_table(&entries, total_size);
            }
        }

        // Optionally run compliance checks
        if check {
            println!();
            let violations = collect_model_violations(&project_path);
            if violations.is_empty() {
                println!("\u{2705} All model files pass quality checks");
            } else {
                for v in &violations {
                    let icon = match v.severity {
                        crate::cli::handlers::comply_cb_detect::Severity::Error => "\u{274c}",
                        crate::cli::handlers::comply_cb_detect::Severity::Warning => {
                            "\u{26a0}\u{fe0f}"
                        }
                        _ => "\u{2139}\u{fe0f}",
                    };
                    println!("{} {}: {} ({})", icon, v.pattern_id, v.description, v.file);
                }
            }
        }

        Ok(())
    } else {
        unreachable!("Expected Models command")
    }
}

struct ModelInventoryEntry {
    file: String,
    format: String,
    size_bytes: u64,
    lfs_tracked: bool,
}

fn format_size(bytes: u64) -> String {
    batuta_common::fmt::format_bytes(bytes)
}

fn print_model_inventory_table(entries: &[ModelInventoryEntry], total_size: u64) {
    let has_lfs = entries.iter().any(|e| e.lfs_tracked);
    let width = if has_lfs { 78 } else { 72 };

    println!(
        "Model Inventory ({} files, {} total)",
        entries.len(),
        format_size(total_size)
    );
    println!("{}", "\u{2500}".repeat(width));
    if has_lfs {
        println!(
            "{:<40} {:<12} {:>12} {:>6}",
            "File", "Format", "Size", "LFS"
        );
    } else {
        println!("{:<40} {:<12} {:>12}", "File", "Format", "Size");
    }
    println!("{}", "\u{2500}".repeat(width));
    for entry in entries {
        let display_file = if entry.file.len() > 38 {
            format!("...{}", &entry.file[entry.file.len() - 35..])
        } else {
            entry.file.clone()
        };
        if has_lfs {
            println!(
                "{:<40} {:<12} {:>12} {:>6}",
                display_file,
                entry.format,
                format_size(entry.size_bytes),
                if entry.lfs_tracked { "Yes" } else { "-" }
            );
        } else {
            println!(
                "{:<40} {:<12} {:>12}",
                display_file,
                entry.format,
                format_size(entry.size_bytes)
            );
        }
    }
    println!("{}", "\u{2500}".repeat(width));
}

fn print_model_inventory_json(entries: &[ModelInventoryEntry], total_size: u64) -> Result<()> {
    let json_entries: Vec<serde_json::Value> = entries
        .iter()
        .map(|e| {
            serde_json::json!({
                "file": e.file,
                "format": e.format,
                "size_bytes": e.size_bytes,
                "size_human": format_size(e.size_bytes),
                "lfs_tracked": e.lfs_tracked,
            })
        })
        .collect();

    let output = serde_json::json!({
        "model_count": entries.len(),
        "total_size_bytes": total_size,
        "total_size_human": format_size(total_size),
        "models": json_entries,
    });

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

/// Parse .gitattributes files to find LFS-tracked patterns
fn detect_lfs_patterns(project_path: &std::path::Path) -> Vec<String> {
    let mut patterns = Vec::new();
    let gitattr_path = project_path.join(".gitattributes");
    if let Ok(content) = std::fs::read_to_string(&gitattr_path) {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if trimmed.contains("filter=lfs") {
                // Extract the pattern (first whitespace-separated token)
                if let Some(pattern) = trimmed.split_whitespace().next() {
                    patterns.push(pattern.to_string());
                }
            }
        }
    }
    patterns
}

/// Check if a filename matches any LFS glob pattern
fn is_lfs_tracked(filename: &str, lfs_patterns: &[String]) -> bool {
    for pattern in lfs_patterns {
        // Simple glob matching: *.ext
        if let Some(ext_pattern) = pattern.strip_prefix("*.") {
            if let Some(file_ext) = filename.rsplit('.').next() {
                if file_ext.eq_ignore_ascii_case(ext_pattern) {
                    return true;
                }
            }
        } else if pattern == filename {
            return true;
        }
    }
    false
}

fn collect_model_violations(
    project_path: &std::path::Path,
) -> Vec<crate::cli::handlers::comply_cb_detect::CbPatternViolation> {
    let mut all = Vec::new();
    all.extend(
        crate::cli::handlers::comply_cb_detect::detect_cb1000_missing_model_card(project_path),
    );
    all.extend(
        crate::cli::handlers::comply_cb_detect::detect_cb1001_oversized_tensor_count(project_path),
    );
    all.extend(
        crate::cli::handlers::comply_cb_detect::detect_cb1002_missing_tokenizer(project_path),
    );
    all.extend(
        crate::cli::handlers::comply_cb_detect::detect_cb1004_missing_architecture(project_path),
    );
    all.extend(
        crate::cli::handlers::comply_cb_detect::detect_cb1005_quantization_mismatch(project_path),
    );
    all.extend(
        crate::cli::handlers::comply_cb_detect::detect_cb1006_sharded_without_index(project_path),
    );
    all.extend(
        crate::cli::handlers::comply_cb_detect::detect_cb1007_excessive_file_size(project_path),
    );
    all.extend(
        crate::cli::handlers::comply_cb_detect::detect_cb1008_apr_missing_crc(project_path),
    );
    all
}
