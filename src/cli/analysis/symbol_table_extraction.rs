// Symbol extraction and file collection logic

#[allow(clippy::too_many_arguments)]
pub async fn handle_analyze_symbol_table(
    project_path: PathBuf,
    format: crate::cli::SymbolTableOutputFormat,
    filter: Option<crate::cli::SymbolTypeFilter>,
    query: Option<String>,
    include: Option<String>,
    exclude: Option<String>,
    show_unreferenced: bool,
    show_references: bool,
    output: Option<PathBuf>,
    _perf: bool,
) -> Result<()> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    eprintln!("🔍 Building symbol table for project...");

    // Build the symbol table
    let table = build_symbol_table(&project_path, &include, &exclude).await?;

    // Apply filters
    let filtered = apply_filters(table, filter, query)?;

    // Format output
    let content = format_output(filtered, format, show_unreferenced, show_references)?;

    // Write output
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &content).await?;
        eprintln!("✅ Symbol table written to: {}", output_path.display());
    } else {
        println!("{content}");
    }

    Ok(())
}

// Build symbol table from project files
async fn build_symbol_table(
    project_path: &Path,
    include: &Option<String>,
    exclude: &Option<String>,
) -> Result<SymbolTable> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let mut symbols = Vec::new();

    // Get all relevant files
    let files = collect_files(project_path, include, exclude).await?;

    // Extract symbols from each file
    for file in files {
        let file_symbols = extract_symbols_from_file(&file).await?;
        symbols.extend(file_symbols);
    }

    // Find unreferenced symbols
    let unreferenced = find_unreferenced_symbols(&symbols);

    // Find most referenced symbols
    let most_referenced = find_most_referenced(&symbols);

    Ok(SymbolTable {
        total_symbols: symbols.len(),
        symbols,
        unreferenced_symbols: unreferenced,
        most_referenced,
    })
}

// Collect files based on include/exclude patterns
async fn collect_files(
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
        process_directory_entry(entry, files, include, exclude).await?;
    }

    Ok(())
}

/// Process a single directory entry
async fn process_directory_entry(
    entry: tokio::fs::DirEntry,
    files: &mut Vec<PathBuf>,
    include: &Option<String>,
    exclude: &Option<String>,
) -> Result<()> {
    let path = entry.path();

    if should_skip_path(&path, exclude) {
        return Ok(());
    }

    if path.is_dir() {
        process_directory(&path, files, include, exclude).await
    } else {
        process_file(path, files, include)
    }
}

/// Check if path should be skipped
fn should_skip_path(path: &Path, exclude: &Option<String>) -> bool {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    if let Some(excl) = exclude {
        let path_str = path.to_string_lossy();
        return path_str.contains(excl);
    }
    false
}

/// Process a directory
async fn process_directory(
    path: &Path,
    files: &mut Vec<PathBuf>,
    include: &Option<String>,
    exclude: &Option<String>,
) -> Result<()> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    if should_process_directory(path) {
        Box::pin(collect_files_recursive(path, files, include, exclude)).await?;
    }
    Ok(())
}

/// Check if directory should be processed
fn should_process_directory(path: &Path) -> bool {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    !name.starts_with('.') && name != "node_modules" && name != "target"
}

/// Process a file
fn process_file(path: PathBuf, files: &mut Vec<PathBuf>, include: &Option<String>) -> Result<()> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    if !is_source_file(&path) {
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

// Check if file is a source file
fn is_source_file(path: &Path) -> bool {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    matches!(
        path.extension().and_then(|s| s.to_str()),
        Some("rs" | "js" | "ts" | "py" | "java" | "cpp" | "c" | "h" | "hpp" | "go" | "rb")
    )
}

// Extract symbols from a single file
async fn extract_symbols_from_file(file_path: &Path) -> Result<Vec<Symbol>> {
    debug_assert!(file_path.exists(), "file_path must exist: {}", file_path.display());
    let content = tokio::fs::read_to_string(file_path).await?;
    let file_str = file_path.to_string_lossy().to_string();

    // Use simple regex-based extraction for now
    extract_symbols_simple(&content, &file_str)
}

// Simple symbol extraction using regex
fn extract_symbols_simple(content: &str, file: &str) -> Result<Vec<Symbol>> {
    use regex::Regex;

    let mut symbols = Vec::new();

    // Function patterns for different languages
    let patterns = vec![
        (
            Regex::new(r"(?m)^(?:pub\s+)?(?:async\s+)?fn\s+(\w+)")?,
            SymbolKind::Function,
        ),
        (Regex::new(r"(?m)^class\s+(\w+)")?, SymbolKind::Class),
        (
            Regex::new(r"(?m)^(?:export\s+)?(?:async\s+)?function\s+(\w+)")?,
            SymbolKind::Function,
        ),
        (Regex::new(r"(?m)^def\s+(\w+)")?, SymbolKind::Function),
        (Regex::new(r"(?m)^const\s+(\w+)\s*=")?, SymbolKind::Constant),
        (
            Regex::new(r"(?m)^(?:pub\s+)?struct\s+(\w+)")?,
            SymbolKind::Type,
        ),
        (
            Regex::new(r"(?m)^(?:pub\s+)?enum\s+(\w+)")?,
            SymbolKind::Enum,
        ),
        (
            Regex::new(r"(?m)^interface\s+(\w+)")?,
            SymbolKind::Interface,
        ),
    ];

    for (line_no, line) in content.lines().enumerate() {
        for (pattern, kind) in &patterns {
            if let Some(captures) = pattern.captures(line) {
                if let Some(name) = captures.get(1) {
                    symbols.push(Symbol {
                        name: name.as_str().to_string(),
                        kind: kind.clone(),
                        file: file.to_string(),
                        line: line_no + 1,
                        column: name.start(),
                        visibility: detect_visibility(line),
                        references: vec![Reference {
                            file: file.to_string(),
                            line: line_no + 1,
                            column: name.start(),
                            kind: ReferenceKind::Definition,
                        }],
                    });
                }
            }
        }
    }

    Ok(symbols)
}

// Detect visibility from line content
fn detect_visibility(line: &str) -> Visibility {
    if line.contains("pub ") || line.contains("export ") {
        Visibility::Public
    } else if line.contains("private ") {
        Visibility::Private
    } else if line.contains("protected ") {
        Visibility::Protected
    } else {
        Visibility::Internal
    }
}

// Find unreferenced symbols
fn find_unreferenced_symbols(symbols: &[Symbol]) -> Vec<String> {
    symbols
        .iter()
        .filter(|s| s.references.len() <= 1)
        .map(|s| s.name.clone())
        .collect()
}

// Find most referenced symbols
fn find_most_referenced(symbols: &[Symbol]) -> Vec<(String, usize)> {
    let mut refs: Vec<_> = symbols
        .iter()
        .map(|s| (s.name.clone(), s.references.len()))
        .collect();

    refs.sort_by(|a, b| b.1.cmp(&a.1));
    refs.truncate(10);
    refs
}
