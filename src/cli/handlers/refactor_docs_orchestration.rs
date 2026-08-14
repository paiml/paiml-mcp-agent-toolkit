// Included via include!() in refactor_docs_handlers.rs
// Orchestration: command handler, directory collection, pattern combining, processing modes

/// Handle refactor docs command
#[allow(clippy::too_many_arguments)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_refactor_docs(
    project_path: PathBuf,
    include_docs: bool,
    include_root: bool,
    additional_dirs: Vec<PathBuf>,
    format: RefactorDocsOutputFormat,
    dry_run: bool,
    temp_patterns: Vec<String>,
    status_patterns: Vec<String>,
    artifact_patterns: Vec<String>,
    custom_patterns: Vec<String>,
    min_age_days: u32,
    max_size_mb: u64,
    recursive: bool,
    preserve_patterns: Vec<String>,
    output: Option<PathBuf>,
    auto_remove: bool,
    backup: bool,
    backup_dir: PathBuf,
    perf: bool,
) -> Result<()> {
    let start_time = std::time::Instant::now();

    let scan_dirs =
        collect_scan_directories(&project_path, include_root, include_docs, additional_dirs);

    let all_patterns = combine_patterns(
        temp_patterns,
        status_patterns,
        artifact_patterns,
        custom_patterns,
    );

    let mut result = perform_cruft_scan(
        &scan_dirs,
        &all_patterns,
        &preserve_patterns,
        min_age_days,
        max_size_mb,
        recursive,
    )
    .await?;

    result =
        handle_processing_modes(result, format, dry_run, auto_remove, backup, &backup_dir).await?;

    output_results(&result, format, dry_run, perf, start_time.elapsed(), output).await?;

    // `--perf` reached only the summary/detailed renderers, which fold the
    // timing into their own markdown document, so `refactor docs --format json
    // --perf` was byte-identical to `refactor docs --format json` — the same
    // format-conditioned defect `analyze web-assembly --perf` had. The readout
    // now comes from the one `--perf` implementation and goes to stderr, so it
    // reaches every format without corrupting the JSON document on stdout.
    crate::cli::handlers::analysis_handlers::perf_report::emit_readout(
        "refactor docs",
        start_time.elapsed(),
        perf,
        &[],
    );

    handle_exit_code(&result, auto_remove, dry_run);
    Ok(())
}

/// Collect directories to scan based on configuration
fn collect_scan_directories(
    project_path: &Path,
    include_root: bool,
    include_docs: bool,
    additional_dirs: Vec<PathBuf>,
) -> Vec<PathBuf> {
    let mut scan_dirs = Vec::new();

    if include_root {
        scan_dirs.push(project_path.to_path_buf());
    }

    if include_docs {
        let docs_dir = project_path.join("docs");
        if docs_dir.exists() {
            scan_dirs.push(docs_dir);
        }
    }

    scan_dirs.extend(additional_dirs);
    scan_dirs
}

/// Combine all pattern types into a single collection
fn combine_patterns(
    temp_patterns: Vec<String>,
    status_patterns: Vec<String>,
    artifact_patterns: Vec<String>,
    custom_patterns: Vec<String>,
) -> Vec<(String, FileCategory)> {
    let mut all_patterns = Vec::new();
    all_patterns.extend(
        temp_patterns
            .into_iter()
            .map(|p| (p, FileCategory::TemporaryScript)),
    );
    all_patterns.extend(
        status_patterns
            .into_iter()
            .map(|p| (p, FileCategory::StatusReport)),
    );
    all_patterns.extend(
        artifact_patterns
            .into_iter()
            .map(|p| (p, FileCategory::BuildArtifact)),
    );
    all_patterns.extend(
        custom_patterns
            .into_iter()
            .map(|p| (p, FileCategory::CustomPattern)),
    );
    all_patterns
}

/// Perform the cruft scanning with proper result sorting
async fn perform_cruft_scan(
    scan_dirs: &[PathBuf],
    all_patterns: &[(String, FileCategory)],
    preserve_patterns: &[String],
    min_age_days: u32,
    max_size_mb: u64,
    recursive: bool,
) -> Result<RefactorDocsResult> {
    let mut result = scan_for_cruft(
        scan_dirs,
        all_patterns,
        preserve_patterns,
        min_age_days,
        max_size_mb * 1024 * 1024, // Convert MB to bytes
        recursive,
    )
    .await?;

    // Sort cruft files by size (largest first)
    result
        .cruft_files
        .sort_by_key(|b| std::cmp::Reverse(b.size_bytes));

    Ok(result)
}

/// Handle processing modes (interactive, backup, removal)
async fn handle_processing_modes(
    mut result: RefactorDocsResult,
    format: RefactorDocsOutputFormat,
    dry_run: bool,
    auto_remove: bool,
    backup: bool,
    backup_dir: &Path,
) -> Result<RefactorDocsResult> {
    result = handle_interactive_processing(result, format, dry_run, auto_remove).await?;
    handle_backup_processing(&result, backup, dry_run, auto_remove, backup_dir).await?;
    handle_file_removal_processing(&result, dry_run, auto_remove, format).await?;
    Ok(result)
}

/// Handle interactive mode processing
async fn handle_interactive_processing(
    result: RefactorDocsResult,
    format: RefactorDocsOutputFormat,
    dry_run: bool,
    auto_remove: bool,
) -> Result<RefactorDocsResult> {
    if should_use_interactive_mode(format, dry_run, auto_remove) {
        handle_interactive_mode(result).await
    } else {
        Ok(result)
    }
}

/// Check if interactive mode should be used
fn should_use_interactive_mode(
    format: RefactorDocsOutputFormat,
    dry_run: bool,
    auto_remove: bool,
) -> bool {
    format == RefactorDocsOutputFormat::Interactive && !dry_run && !auto_remove
}

/// Handle backup processing
async fn handle_backup_processing(
    result: &RefactorDocsResult,
    backup: bool,
    dry_run: bool,
    auto_remove: bool,
    backup_dir: &Path,
) -> Result<()> {
    if should_create_backup(backup, dry_run, &result.cruft_files, auto_remove) {
        create_backup(&result.cruft_files, backup_dir).await?;
    }
    Ok(())
}

/// Check if backup should be created
fn should_create_backup(
    backup: bool,
    dry_run: bool,
    cruft_files: &[CruftFile],
    auto_remove: bool,
) -> bool {
    backup && !dry_run && (!cruft_files.is_empty() || auto_remove)
}

/// Handle file removal processing
async fn handle_file_removal_processing(
    result: &RefactorDocsResult,
    dry_run: bool,
    auto_remove: bool,
    format: RefactorDocsOutputFormat,
) -> Result<()> {
    if should_remove_files(dry_run, auto_remove, format) {
        remove_files(&result.cruft_files).await?;
    }
    Ok(())
}

/// Check if files should be removed
fn should_remove_files(dry_run: bool, auto_remove: bool, format: RefactorDocsOutputFormat) -> bool {
    !dry_run && (auto_remove || format == RefactorDocsOutputFormat::Interactive)
}

/// Output results based on format and configuration
async fn output_results(
    result: &RefactorDocsResult,
    format: RefactorDocsOutputFormat,
    dry_run: bool,
    perf: bool,
    elapsed: std::time::Duration,
    output: Option<PathBuf>,
) -> Result<()> {
    let output_content = format_output(result, format, dry_run, perf, elapsed)?;

    if let Some(output_path) = output {
        tokio::fs::write(output_path, &output_content).await?;
    } else {
        println!("{output_content}");
    }
    Ok(())
}

/// Handle appropriate exit code based on results
fn handle_exit_code(result: &RefactorDocsResult, auto_remove: bool, dry_run: bool) {
    if !result.cruft_files.is_empty() && !auto_remove && !dry_run {
        std::process::exit(1); // Files found but not removed
    }
}
