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
