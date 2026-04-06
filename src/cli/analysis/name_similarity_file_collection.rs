// Collect names from project based on scope
async fn collect_names(
    project_path: &Path,
    include: &Option<String>,
    exclude: &Option<String>,
    scope: crate::cli::SearchScope,
) -> Result<Vec<(String, String, usize, String)>> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let mut names = Vec::new();
    let files = collect_source_files(project_path, include, exclude).await?;

    for file in files {
        let content = tokio::fs::read_to_string(&file).await?;
        let file_str = file.to_string_lossy().to_string();

        // Extract names based on scope
        let file_names = extract_names(&content, &file_str, scope)?;
        names.extend(file_names);
    }

    Ok(names)
}

// Collect source files
async fn collect_source_files(
    project_path: &Path,
    include: &Option<String>,
    exclude: &Option<String>,
) -> Result<Vec<PathBuf>> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let mut files = Vec::new();

    collect_files_recursive(project_path, &mut files, include, exclude).await?;

    Ok(files)
}

// Recursively collect files
async fn collect_files_recursive(
    dir: &Path,
    files: &mut Vec<PathBuf>,
    include: &Option<String>,
    exclude: &Option<String>,
) -> Result<()> {
    debug_assert!(dir.exists(), "dir must exist: {}", dir.display());
    let mut entries = tokio::fs::read_dir(dir).await?;

    while let Some(entry) = entries.next_entry().await? {
        process_entry(entry, files, include, exclude).await?;
    }

    Ok(())
}

/// Process a single directory entry
async fn process_entry(
    entry: tokio::fs::DirEntry,
    files: &mut Vec<PathBuf>,
    include: &Option<String>,
    exclude: &Option<String>,
) -> Result<()> {
    let path = entry.path();

    if should_skip(&path, exclude) {
        return Ok(());
    }

    if path.is_dir() {
        handle_directory(&path, files, include, exclude).await
    } else {
        handle_file(path, files, include)
    }
}

/// Check if path should be skipped
fn should_skip(path: &Path, exclude: &Option<String>) -> bool {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    if let Some(excl) = exclude {
        let path_str = path.to_string_lossy();
        return path_str.contains(excl);
    }
    false
}

/// Handle directory traversal
async fn handle_directory(
    path: &Path,
    files: &mut Vec<PathBuf>,
    include: &Option<String>,
    exclude: &Option<String>,
) -> Result<()> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    if should_traverse_directory(path) {
        Box::pin(collect_files_recursive(path, files, include, exclude)).await?;
    }
    Ok(())
}

/// Check if directory should be traversed
fn should_traverse_directory(path: &Path) -> bool {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    !name.starts_with('.') && name != "node_modules" && name != "target"
}

/// Handle file inclusion
fn handle_file(path: PathBuf, files: &mut Vec<PathBuf>, include: &Option<String>) -> Result<()> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    if !is_code_file(&path) {
        return Ok(());
    }

    if should_include_file(&path, include) {
        files.push(path);
    }
    Ok(())
}

/// Check if file should be included
fn should_include_file(path: &Path, include: &Option<String>) -> bool {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    match include {
        Some(incl) => {
            let path_str = path.to_string_lossy();
            path_str.contains(incl)
        }
        None => true,
    }
}

// Check if file is a code file
fn is_code_file(path: &Path) -> bool {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    matches!(
        path.extension().and_then(|s| s.to_str()),
        Some("rs" | "js" | "ts" | "py" | "java" | "cpp" | "c" | "go")
    )
}
