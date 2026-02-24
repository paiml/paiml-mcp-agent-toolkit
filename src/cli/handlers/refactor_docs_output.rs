// Included via include!() in refactor_docs_handlers.rs
// Output: formatting (summary, detailed, JSON), interactive mode, backup, file removal

/// Handle interactive mode
async fn handle_interactive_mode(mut result: RefactorDocsResult) -> Result<RefactorDocsResult> {
    let mut stdin = BufReader::new(io::stdin());
    let mut stdout = io::stdout();
    let mut to_remove = Vec::new();

    println!(
        "\n\u{1f50d} Found {} files for potential cleanup:\n",
        result.cruft_files.len()
    );

    for (idx, file) in result.cruft_files.iter().enumerate() {
        println!(
            "[{}] {} ({} bytes, {} days old)",
            idx + 1,
            file.path.display(),
            file.size_bytes,
            file.age_days
        );
        println!("    Category: {}", file.category);
        println!("    Reason: {}", file.reason);

        stdout
            .write_all(b"\n    Remove this file? [y/N/a/q] ")
            .await?;
        stdout.flush().await?;

        let mut response = String::new();
        stdin.read_line(&mut response).await?;

        match response.trim().to_lowercase().as_str() {
            "y" | "yes" => {
                to_remove.push(file.clone());
                println!("    \u{2713} Marked for removal");
            }
            "a" | "all" => {
                // Add all remaining files
                to_remove.extend(result.cruft_files[idx..].iter().cloned());
                println!("    \u{2713} Marked all remaining files for removal");
                break;
            }
            "q" | "quit" => {
                println!("    \u{2717} Cancelled");
                break;
            }
            _ => {
                println!("    \u{2717} Skipped");
            }
        }
    }

    result.cruft_files = to_remove;
    Ok(result)
}

/// Create backup of files
async fn create_backup(files: &[CruftFile], backup_dir: &Path) -> Result<()> {
    // Create backup directory with timestamp
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let backup_path = backup_dir.join(format!("refactor_docs_{timestamp}"));

    tokio::fs::create_dir_all(&backup_path).await?;

    println!("\u{1f4e6} Creating backup in: {}", backup_path.display());

    for file in files {
        let relative_path = file.path.strip_prefix("/").unwrap_or(&file.path);
        let backup_file_path = backup_path.join(relative_path);

        if let Some(parent) = backup_file_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        tokio::fs::copy(&file.path, &backup_file_path)
            .await
            .with_context(|| format!("Failed to backup {}", file.path.display()))?;
    }

    println!("\u{2705} Backup created successfully");
    Ok(())
}

/// Remove files
async fn remove_files(files: &[CruftFile]) -> Result<()> {
    let mut removed = 0;
    let mut errors = Vec::new();

    for file in files {
        match tokio::fs::remove_file(&file.path).await {
            Ok(()) => {
                removed += 1;
            }
            Err(e) => {
                errors.push(format!("Failed to remove {}: {}", file.path.display(), e));
            }
        }
    }

    if !errors.is_empty() {
        eprintln!("\u{26a0}\u{fe0f}  Errors during removal:");
        for error in errors {
            eprintln!("  - {error}");
        }
    }

    println!("\u{1f5d1}\u{fe0f}  Removed {removed} files");
    Ok(())
}

/// Format output based on format type
fn format_output(
    result: &RefactorDocsResult,
    format: RefactorDocsOutputFormat,
    dry_run: bool,
    perf: bool,
    elapsed: std::time::Duration,
) -> Result<String> {
    match format {
        RefactorDocsOutputFormat::Summary => format_summary(result, dry_run, perf, elapsed),
        RefactorDocsOutputFormat::Detailed => format_detailed(result, dry_run, perf, elapsed),
        RefactorDocsOutputFormat::Json => format_json(result),
        RefactorDocsOutputFormat::Interactive => format_summary(result, dry_run, perf, elapsed),
    }
}

/// Format summary output
fn format_summary(
    result: &RefactorDocsResult,
    dry_run: bool,
    perf: bool,
    elapsed: std::time::Duration,
) -> Result<String> {
    let mut output = String::new();

    output.push_str("# Documentation Refactoring Report\n\n");

    if dry_run {
        output.push_str("**Mode**: Dry Run (no files will be removed)\n\n");
    }

    output.push_str("## Summary\n\n");
    append_summary_stats(&mut output, result);

    if !result.summary.files_by_category.is_empty() {
        append_category_breakdown(&mut output, result);
    }

    if !result.errors.is_empty() {
        append_errors(&mut output, result);
    }

    if perf {
        output.push_str(&format!(
            "\u{23f1}\u{fe0f}  Analysis completed in {:.2}s\n",
            elapsed.as_secs_f64()
        ));
    }

    Ok(output)
}

/// Append summary statistics to output string
fn append_summary_stats(output: &mut String, result: &RefactorDocsResult) {
    output.push_str(&format!(
        "- **Files Scanned**: {}\n",
        result.summary.total_files_scanned
    ));
    output.push_str(&format!(
        "- **Cruft Files Found**: {}\n",
        result.summary.cruft_files_found
    ));
    output.push_str(&format!(
        "- **Total Size**: {:.2} MB\n",
        result.summary.total_size_bytes as f64 / 1_048_576.0
    ));
    output.push_str(&format!(
        "- **Oldest File**: {} days\n",
        result.summary.oldest_file_days
    ));
    output.push_str(&format!(
        "- **Newest File**: {} days\n\n",
        result.summary.newest_file_days
    ));
}

/// Append category breakdown to output string
fn append_category_breakdown(output: &mut String, result: &RefactorDocsResult) {
    output.push_str("## Files by Category\n\n");
    for (category, count) in &result.summary.files_by_category {
        let size = result.summary.size_by_category.get(category).unwrap_or(&0);
        output.push_str(&format!(
            "- **{}**: {} files ({:.2} MB)\n",
            category,
            count,
            *size as f64 / 1_048_576.0
        ));
    }
    output.push('\n');
}

/// Append error section to output string
fn append_errors(output: &mut String, result: &RefactorDocsResult) {
    output.push_str("## \u{26a0}\u{fe0f} Errors\n\n");
    for error in &result.errors {
        output.push_str(&format!("- {error}\n"));
    }
    output.push('\n');
}

/// Format detailed output
fn format_detailed(
    result: &RefactorDocsResult,
    dry_run: bool,
    perf: bool,
    elapsed: std::time::Duration,
) -> Result<String> {
    let mut output = format_summary(result, dry_run, perf, elapsed)?;

    if !result.cruft_files.is_empty() {
        append_cruft_file_details(&mut output, result);
    }

    if !result.preserved_files.is_empty() && result.preserved_files.len() <= 20 {
        append_preserved_files(&mut output, result);
    }

    Ok(output)
}

/// Append detailed cruft file information to output string
fn append_cruft_file_details(output: &mut String, result: &RefactorDocsResult) {
    output.push_str("## Cruft Files Details\n\n");

    for file in &result.cruft_files {
        let modified_date = DateTime::<Utc>::from(file.modified);
        output.push_str(&format!("### {}\n", file.path.display()));
        output.push_str(&format!("- **Category**: {}\n", file.category));
        output.push_str(&format!("- **Size**: {} bytes\n", file.size_bytes));
        output.push_str(&format!("- **Age**: {} days\n", file.age_days));
        output.push_str(&format!(
            "- **Modified**: {}\n",
            modified_date.format("%Y-%m-%d %H:%M:%S")
        ));
        output.push_str(&format!("- **Pattern**: {}\n", file.pattern_matched));
        output.push_str(&format!("- **Reason**: {}\n\n", file.reason));
    }
}

/// Append preserved files list to output string
fn append_preserved_files(output: &mut String, result: &RefactorDocsResult) {
    output.push_str("## Preserved Files\n\n");
    for file in &result.preserved_files {
        output.push_str(&format!("- {}\n", file.display()));
    }
    output.push('\n');
}

/// Format JSON output
fn format_json(result: &RefactorDocsResult) -> Result<String> {
    serde_json::to_string_pretty(result).context("Failed to serialize to JSON")
}
