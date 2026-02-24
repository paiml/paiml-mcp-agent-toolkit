// Included via include!() in refactor_docs_handlers.rs
// Scanning: directory traversal, file processing, pattern matching, metadata helpers

/// Scan directories for cruft files
async fn scan_for_cruft(
    scan_dirs: &[PathBuf],
    patterns: &[(String, FileCategory)],
    preserve_patterns: &[String],
    min_age_days: u32,
    max_size_bytes: u64,
    recursive: bool,
) -> Result<RefactorDocsResult> {
    let mut cruft_files = Vec::new();
    let mut preserved_files = Vec::new();
    let mut errors = Vec::new();
    let mut total_files_scanned = 0;
    let mut summary = CleanupSummary::default();
    let now = SystemTime::now();

    for dir in scan_dirs {
        let dir_result = process_directory(
            dir,
            patterns,
            preserve_patterns,
            min_age_days,
            max_size_bytes,
            recursive,
            &now,
        )
        .await?;

        cruft_files.extend(dir_result.cruft_files);
        preserved_files.extend(dir_result.preserved_files);
        errors.extend(dir_result.errors);
        total_files_scanned += dir_result.files_scanned;
        merge_summary(&mut summary, &dir_result.summary);
    }

    finalize_summary(&mut summary, total_files_scanned, &cruft_files);

    Ok(RefactorDocsResult {
        cruft_files,
        summary,
        preserved_files,
        errors,
    })
}

/// Process a single directory for cruft files
async fn process_directory(
    dir: &Path,
    patterns: &[(String, FileCategory)],
    preserve_patterns: &[String],
    min_age_days: u32,
    max_size_bytes: u64,
    recursive: bool,
    now: &SystemTime,
) -> Result<DirectoryResult> {
    if !dir.exists() {
        return Ok(DirectoryResult {
            cruft_files: Vec::new(),
            preserved_files: Vec::new(),
            errors: vec![format!("Directory does not exist: {}", dir.display())],
            files_scanned: 0,
            summary: CleanupSummary::default(),
        });
    }

    let files = collect_directory_files(dir, recursive).await?;
    let mut result = DirectoryResult {
        cruft_files: Vec::new(),
        preserved_files: Vec::new(),
        errors: Vec::new(),
        files_scanned: files.len(),
        summary: CleanupSummary::default(),
    };

    for file_path in files {
        process_file(
            &file_path,
            patterns,
            preserve_patterns,
            min_age_days,
            max_size_bytes,
            now,
            &mut result,
        )
        .await;
    }

    Ok(result)
}

/// Collect files from directory based on recursive setting
async fn collect_directory_files(dir: &Path, recursive: bool) -> Result<Vec<PathBuf>> {
    if recursive {
        collect_files_recursive(dir).await
    } else {
        collect_files_flat(dir).await
    }
}

/// Process a single file for cruft classification
async fn process_file(
    file_path: &Path,
    patterns: &[(String, FileCategory)],
    preserve_patterns: &[String],
    min_age_days: u32,
    max_size_bytes: u64,
    now: &SystemTime,
    result: &mut DirectoryResult,
) {
    if should_preserve(file_path, preserve_patterns) {
        result.preserved_files.push(file_path.to_path_buf());
        return;
    }

    let metadata = match get_file_metadata(file_path) {
        Ok(m) => m,
        Err(error) => {
            result.errors.push(error);
            return;
        }
    };

    if !passes_file_filters(&metadata, min_age_days, max_size_bytes, now) {
        return;
    }

    if let Some((pattern, category)) = matches_pattern(file_path, patterns) {
        let cruft = create_cruft_file(file_path, &metadata, category, &pattern, now);
        update_summary_for_cruft(&mut result.summary, &cruft);
        result.cruft_files.push(cruft);
    }
}

/// Get file metadata with error handling
fn get_file_metadata(file_path: &Path) -> Result<fs::Metadata, String> {
    fs::metadata(file_path)
        .map_err(|e| format!("Failed to read metadata for {}: {}", file_path.display(), e))
}

/// Check if file passes size and age filters
fn passes_file_filters(
    metadata: &fs::Metadata,
    min_age_days: u32,
    max_size_bytes: u64,
    now: &SystemTime,
) -> bool {
    if metadata.len() > max_size_bytes {
        return false;
    }

    let age_days = calculate_age_days(metadata, now);
    age_days >= min_age_days
}

/// Calculate file age in days
fn calculate_age_days(metadata: &fs::Metadata, now: &SystemTime) -> u32 {
    match metadata.modified() {
        Ok(modified) => {
            let duration = now.duration_since(modified).unwrap_or_default();
            (duration.as_secs() / 86400) as u32
        }
        Err(_) => 0,
    }
}

/// Create a `CruftFile` from metadata and classification
fn create_cruft_file(
    file_path: &Path,
    metadata: &fs::Metadata,
    category: FileCategory,
    pattern: &str,
    now: &SystemTime,
) -> CruftFile {
    let age_days = calculate_age_days(metadata, now);
    CruftFile {
        path: file_path.to_path_buf(),
        category,
        size_bytes: metadata.len(),
        modified: metadata.modified().unwrap_or(*now),
        age_days,
        reason: format!("Matches pattern: {pattern}"),
        pattern_matched: pattern.to_string(),
    }
}

/// Update summary statistics for a cruft file
fn update_summary_for_cruft(summary: &mut CleanupSummary, cruft: &CruftFile) {
    let category_str = cruft.category.to_string();
    *summary
        .files_by_category
        .entry(category_str.clone())
        .or_default() += 1;
    *summary.size_by_category.entry(category_str).or_default() += cruft.size_bytes;
    summary.oldest_file_days = summary.oldest_file_days.max(cruft.age_days);
    summary.newest_file_days = if summary.newest_file_days == 0 {
        cruft.age_days
    } else {
        summary.newest_file_days.min(cruft.age_days)
    };
}

/// Merge directory summary into main summary
fn merge_summary(main: &mut CleanupSummary, dir: &CleanupSummary) {
    for (category, count) in &dir.files_by_category {
        *main.files_by_category.entry(category.clone()).or_default() += count;
    }
    for (category, size) in &dir.size_by_category {
        *main.size_by_category.entry(category.clone()).or_default() += size;
    }
    main.oldest_file_days = main.oldest_file_days.max(dir.oldest_file_days);
    main.newest_file_days = if main.newest_file_days == 0 {
        dir.newest_file_days
    } else {
        main.newest_file_days.min(dir.newest_file_days)
    };
}

/// Finalize summary with total counts
fn finalize_summary(
    summary: &mut CleanupSummary,
    total_files_scanned: usize,
    cruft_files: &[CruftFile],
) {
    summary.total_files_scanned = total_files_scanned;
    summary.cruft_files_found = cruft_files.len();
    summary.total_size_bytes = cruft_files.iter().map(|f| f.size_bytes).sum();
}

/// Collect files recursively
async fn collect_files_recursive(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut dirs_to_process = vec![dir.to_path_buf()];

    while let Some(current_dir) = dirs_to_process.pop() {
        let mut entries = tokio::fs::read_dir(&current_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            if path.is_dir() {
                dirs_to_process.push(path);
            } else if path.is_file() {
                files.push(path);
            }
        }
    }

    Ok(files)
}

/// Collect files in a single directory (non-recursive)
async fn collect_files_flat(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut entries = tokio::fs::read_dir(dir).await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_file() {
            files.push(path);
        }
    }

    Ok(files)
}

/// Check if a file should be preserved
fn should_preserve(path: &Path, preserve_patterns: &[String]) -> bool {
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    for pattern in preserve_patterns {
        if let Ok(pattern_glob) = glob::Pattern::new(pattern) {
            if pattern_glob.matches(file_name) {
                return true;
            }
        }
    }

    false
}

/// Check if a file matches any pattern
fn matches_pattern(
    path: &Path,
    patterns: &[(String, FileCategory)],
) -> Option<(String, FileCategory)> {
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    for (pattern, category) in patterns {
        if let Ok(pattern_glob) = glob::Pattern::new(pattern) {
            if pattern_glob.matches(file_name) {
                return Some((pattern.clone(), *category));
            }
        }
    }

    None
}
