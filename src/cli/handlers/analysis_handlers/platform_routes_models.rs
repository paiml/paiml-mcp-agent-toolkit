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
            let stdout = no_models_stdout(&project_path, &format)?;
            println!("{stdout}");
            if matches!(format, cli::OutputFormat::Json) {
                // The human sentence still exists, just not on the JSON stream.
                eprintln!("{}", no_models_sentence(&project_path));
            }
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

/// The human sentence emitted when a project contains no model files.
fn no_models_sentence(project_path: &std::path::Path) -> String {
    format!(
        "No model files found (*.gguf, *.apr, *.safetensors) in {}",
        project_path.display()
    )
}

/// What `analyze models` writes to **stdout** when no model files exist.
///
/// Issue #678: this used to be the prose sentence for every format, so
/// `analyze models --format json` exited 0 having printed
/// "No model files found (*.gguf, *.apr, *.safetensors) in <path>" — which
/// does not parse as JSON. Under `--format json` stdout now carries the empty
/// inventory document; the sentence moves to stderr.
fn no_models_stdout(project_path: &std::path::Path, format: &cli::OutputFormat) -> Result<String> {
    if matches!(format, cli::OutputFormat::Json) {
        render_model_inventory_json(&[], 0)
    } else {
        Ok(no_models_sentence(project_path))
    }
}

fn print_model_inventory_json(entries: &[ModelInventoryEntry], total_size: u64) -> Result<()> {
    println!("{}", render_model_inventory_json(entries, total_size)?);
    Ok(())
}

fn render_model_inventory_json(entries: &[ModelInventoryEntry], total_size: u64) -> Result<String> {
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

    Ok(serde_json::to_string_pretty(&output)?)
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
    all.extend(crate::cli::handlers::comply_cb_detect::detect_cb1008_apr_missing_crc(project_path));
    all
}

#[cfg(test)]
mod model_helper_tests {
    //! Wave 39 PR20 — pure-helper coverage for platform_routes_models.rs.
    //! `route_model_analysis` is async + filesystem-bound (disqualified);
    //! the pure helpers `format_size`, `is_lfs_tracked`, and the parser
    //! `detect_lfs_patterns` are testable.
    use super::*;

    // ── no_models_stdout (issue #678) ───────────────────────────────────────

    /// Issue #678 regression: `analyze models --format json` on a directory
    /// with no model files printed the prose sentence "No model files found
    /// (*.gguf, *.apr, *.safetensors) in <path>" to stdout and exited 0, so
    /// `python3 -m json.tool` / `jq .` failed.
    #[test]
    fn test_no_models_json_stdout_parses_as_json() {
        let path = std::path::Path::new("/tmp/some-fixture");
        let out = no_models_stdout(path, &cli::OutputFormat::Json).unwrap();

        assert!(
            !out.starts_with("No model files found"),
            "--format json must not emit the prose sentence on stdout, got {out:?}"
        );
        let doc: serde_json::Value =
            serde_json::from_str(&out).expect("--format json must emit parseable JSON");
        assert_eq!(doc["model_count"], 0);
        assert_eq!(doc["total_size_bytes"], 0);
        assert!(doc["models"].as_array().unwrap().is_empty());
    }

    /// The human formats keep the sentence, and it names the path scanned.
    #[test]
    fn test_no_models_table_stdout_is_the_sentence() {
        let path = std::path::Path::new("/tmp/some-fixture");
        let out = no_models_stdout(path, &cli::OutputFormat::Table).unwrap();
        assert!(out.starts_with("No model files found"));
        assert!(out.contains("/tmp/some-fixture"));
    }

    // ── format_size (delegates to batuta_common::fmt) ──────────────────────

    #[test]
    fn test_format_size_zero_bytes() {
        let s = format_size(0);
        assert!(!s.is_empty());
    }

    #[test]
    fn test_format_size_human_units() {
        // batuta_common::fmt::format_bytes returns "K"/"M"/"G" suffix forms.
        assert!(format_size(1).len() < 20);
        assert!(format_size(1024).len() < 20);
        assert!(format_size(1024 * 1024).len() < 20);
        assert!(format_size(u64::MAX).len() < 30);
    }

    // ── is_lfs_tracked ──────────────────────────────────────────────────────

    #[test]
    fn test_is_lfs_tracked_glob_extension_match() {
        let patterns = vec!["*.gguf".to_string(), "*.bin".to_string()];
        assert!(is_lfs_tracked("model.gguf", &patterns));
        assert!(is_lfs_tracked("weights.bin", &patterns));
    }

    #[test]
    fn test_is_lfs_tracked_glob_extension_case_insensitive() {
        // PIN: extension comparison is case-insensitive (eq_ignore_ascii_case).
        let patterns = vec!["*.gguf".to_string()];
        assert!(is_lfs_tracked("model.GGUF", &patterns));
        assert!(is_lfs_tracked("MODEL.GgUf", &patterns));
    }

    #[test]
    fn test_is_lfs_tracked_exact_filename_match() {
        let patterns = vec!["secrets.env".to_string()];
        assert!(is_lfs_tracked("secrets.env", &patterns));
    }

    #[test]
    fn test_is_lfs_tracked_no_match_returns_false() {
        let patterns = vec!["*.gguf".to_string()];
        assert!(!is_lfs_tracked("model.txt", &patterns));
        assert!(!is_lfs_tracked("notes.md", &patterns));
    }

    #[test]
    fn test_is_lfs_tracked_empty_patterns() {
        assert!(!is_lfs_tracked("model.gguf", &[]));
    }

    #[test]
    fn test_is_lfs_tracked_glob_only_supports_star_dot() {
        // PIN: simple matcher only handles `*.ext` form. More complex globs
        // (e.g. `models/*.bin`) fall through to the exact-filename branch.
        let patterns = vec!["models/*.bin".to_string()];
        // Won't match because parsing only recognizes `*.` prefix exactly.
        assert!(!is_lfs_tracked("model.bin", &patterns));
        // Exact match still works.
        assert!(is_lfs_tracked("models/*.bin", &patterns));
    }

    // ── detect_lfs_patterns ─────────────────────────────────────────────────

    #[test]
    fn test_detect_lfs_patterns_missing_gitattributes() {
        let tmp = tempfile::tempdir().unwrap();
        let patterns = detect_lfs_patterns(tmp.path());
        assert!(patterns.is_empty());
    }

    #[test]
    fn test_detect_lfs_patterns_extracts_lfs_lines() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join(".gitattributes"),
            "*.gguf filter=lfs diff=lfs merge=lfs -text\n*.bin filter=lfs diff=lfs merge=lfs -text\n",
        )
        .unwrap();
        let patterns = detect_lfs_patterns(tmp.path());
        assert_eq!(patterns.len(), 2);
        assert!(patterns.contains(&"*.gguf".to_string()));
        assert!(patterns.contains(&"*.bin".to_string()));
    }

    #[test]
    fn test_detect_lfs_patterns_skips_comments_and_blank_lines() {
        // PIN: lines starting with `#` are skipped; blank lines are skipped.
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join(".gitattributes"),
            "# Top-level comment\n\n*.gguf filter=lfs diff=lfs merge=lfs -text\n# Another comment\n",
        )
        .unwrap();
        let patterns = detect_lfs_patterns(tmp.path());
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0], "*.gguf");
    }

    #[test]
    fn test_detect_lfs_patterns_skips_non_lfs_lines() {
        // PIN: only lines containing `filter=lfs` are extracted.
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join(".gitattributes"),
            "*.txt text\n*.gguf filter=lfs diff=lfs merge=lfs -text\n*.md text=auto\n",
        )
        .unwrap();
        let patterns = detect_lfs_patterns(tmp.path());
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0], "*.gguf");
    }

    #[test]
    fn test_detect_lfs_patterns_extracts_first_token() {
        // PIN: pattern = first whitespace-separated token.
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join(".gitattributes"),
            "models/*.safetensors filter=lfs diff=lfs merge=lfs -text\n",
        )
        .unwrap();
        let patterns = detect_lfs_patterns(tmp.path());
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0], "models/*.safetensors");
    }
}
