// Raw search engine: pattern building, file walking, match collection, and utilities.
// Included into raw_search.rs — shares its module scope (imports, types, etc.).

/// Check whether a file should be skipped based on language filter and exclude glob.
fn should_skip_file(
    path: &Path,
    relative_path: &str,
    lang_extensions: &Option<Vec<&str>>,
    exclude_glob: &Option<globset::GlobSet>,
) -> bool {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    // Apply language filter
    if let Some(ref exts) = lang_extensions {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !exts.contains(&ext) {
            return true;
        }
    }

    // Apply exclude file pattern
    if let Some(ref glob) = exclude_glob {
        if glob.is_match(relative_path) {
            return true;
        }
    }

    false
}

/// Accumulator for per-file match results, passed into the line-matching helper.
struct FileMatchAccumulator {
    /// Line-level results (normal mode)
    results: Vec<RawSearchResult>,
    /// File paths (--files-with-matches mode)
    file_matches: Vec<String>,
    /// Per-file counts (--count mode)
    file_counts: Vec<FileMatchCount>,
    /// Running total of results collected (for limit enforcement)
    total_results: usize,
}

/// Check if a line matches the search pattern and passes the exclude filter.
fn line_matches(line: &str, regex: &Regex, exclude_regex: &Option<Regex>) -> bool {
    if !regex.is_match(line) {
        return false;
    }
    if let Some(ref exc) = exclude_regex {
        if exc.is_match(line) {
            return false;
        }
    }
    true
}

/// Build a RawSearchResult with context lines around the match at index `i`.
fn build_match_result(
    lines: &[&str],
    relative_path: &str,
    i: usize,
    before_ctx: usize,
    after_ctx: usize,
) -> RawSearchResult {
    let before_start = i.saturating_sub(before_ctx);
    let after_end = (i + 1 + after_ctx).min(lines.len());

    let context_before: Vec<String> = lines[before_start..i]
        .iter()
        .map(|s| s.to_string())
        .collect();

    let context_after: Vec<String> = lines
        .get(i + 1..after_end)
        .map(|slice| slice.iter().map(|s| s.to_string()).collect())
        .unwrap_or_default();

    RawSearchResult {
        file_path: relative_path.to_string(),
        line_number: i + 1,
        line_content: lines[i].to_string(),
        context_before,
        context_after,
    }
}

/// Count matching lines in a file for files-with-matches mode.
/// Returns true if any match was found (file is recorded in accumulator).
fn collect_files_with_matches(
    lines: &[&str],
    relative_path: &str,
    regex: &Regex,
    exclude_regex: &Option<Regex>,
    acc: &mut FileMatchAccumulator,
) {
    let has_match = lines
        .iter()
        .any(|line| line_matches(line, regex, exclude_regex));
    if has_match {
        acc.file_matches.push(relative_path.to_string());
    }
}

/// Count matching lines in a file for --count mode.
fn collect_count_matches(
    lines: &[&str],
    relative_path: &str,
    regex: &Regex,
    exclude_regex: &Option<Regex>,
    acc: &mut FileMatchAccumulator,
) {
    let count = lines
        .iter()
        .filter(|line| line_matches(line, regex, exclude_regex))
        .count();
    if count > 0 {
        acc.file_counts.push(FileMatchCount {
            file_path: relative_path.to_string(),
            count,
        });
    }
}

/// Collect line-level match results with context.
/// Returns `true` if the global result limit has been reached.
fn collect_line_matches(
    lines: &[&str],
    relative_path: &str,
    regex: &Regex,
    exclude_regex: &Option<Regex>,
    options: &RawSearchOptions,
    acc: &mut FileMatchAccumulator,
) -> bool {
    for (i, line) in lines.iter().enumerate() {
        if !line_matches(line, regex, exclude_regex) {
            continue;
        }
        if options.limit > 0 && acc.total_results >= options.limit {
            return true;
        }
        acc.results.push(build_match_result(
            lines,
            relative_path,
            i,
            options.before_context,
            options.after_context,
        ));
        acc.total_results += 1;
    }
    options.limit > 0 && acc.total_results >= options.limit
}

/// Process all lines in a single file, collecting matches into the accumulator.
/// Dispatches to mode-specific collectors. Returns `true` if the global limit is reached.
fn collect_file_matches(
    lines: &[&str],
    relative_path: &str,
    regex: &Regex,
    exclude_regex: &Option<Regex>,
    options: &RawSearchOptions,
    acc: &mut FileMatchAccumulator,
) -> bool {
    if options.files_with_matches {
        collect_files_with_matches(lines, relative_path, regex, exclude_regex, acc);
        return false;
    }
    if options.count_mode {
        collect_count_matches(lines, relative_path, regex, exclude_regex, acc);
        return false;
    }
    collect_line_matches(lines, relative_path, regex, exclude_regex, options, acc)
}

/// Build the search and exclude regex patterns from options.
fn build_search_patterns(options: &RawSearchOptions) -> Result<(Regex, Option<Regex>), String> {
    let pattern_str = if options.literal {
        regex::escape(options.pattern)
    } else {
        options.pattern.to_string()
    };

    let regex = if options.case_insensitive {
        Regex::new(&format!("(?i){}", pattern_str))
    } else {
        Regex::new(&pattern_str)
    }
    .map_err(|e| format!("Invalid regex pattern: {e}"))?;

    let mut exclude_regexes = Vec::new();
    for p in &options.exclude_pattern {
        let r = Regex::new(&format!("(?i){}", regex::escape(p)))
            .map_err(|e| format!("Invalid exclude pattern: {e}"))?;
        exclude_regexes.push(r);
    }
    let exclude_regex = if exclude_regexes.is_empty() {
        None
    } else if exclude_regexes.len() == 1 {
        Some(exclude_regexes.remove(0))
    } else {
        // Combine into single alternation regex for efficiency
        let combined = options
            .exclude_pattern
            .iter()
            .map(|p| regex::escape(p))
            .collect::<Vec<_>>()
            .join("|");
        Some(Regex::new(&format!("(?i)(?:{})", combined))
            .map_err(|e| format!("Invalid exclude pattern: {e}"))?)
    };

    Ok((regex, exclude_regex))
}

/// Build the exclude file glob from options.
fn build_exclude_glob(options: &RawSearchOptions) -> Option<globset::GlobSet> {
    if options.exclude_file_pattern.is_empty() {
        return None;
    }
    let mut builder = globset::GlobSetBuilder::new();
    for g in &options.exclude_file_pattern {
        if let Ok(glob) = globset::GlobBuilder::new(&format!("**{g}**"))
            .case_insensitive(true)
            .build()
        {
            builder.add(glob);
        }
    }
    builder.build().ok().filter(|gs| !gs.is_empty())
}

/// Walk project files and collect matches into the accumulator.
fn walk_and_collect(
    project_root: &Path,
    regex: &Regex,
    exclude_regex: &Option<Regex>,
    lang_extensions: &Option<Vec<&str>>,
    exclude_glob: &Option<globset::GlobSet>,
    options: &RawSearchOptions,
    acc: &mut FileMatchAccumulator,
) {
    debug_assert!(project_root.exists(), "project_root must exist: {}", project_root.display());
    let walker = WalkBuilder::new(project_root)
        .hidden(true)
        .git_ignore(true)
        .git_global(true)
        .add_custom_ignore_filename(".pmatignore")
        .build();

    for entry in walker.filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_file() || is_search_ignored_dir(path) {
            continue;
        }

        let relative_path = path
            .strip_prefix(project_root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        if should_skip_file(path, &relative_path, lang_extensions, exclude_glob) {
            continue;
        }

        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let lines: Vec<&str> = content.lines().collect();
        let limit_reached =
            collect_file_matches(&lines, &relative_path, regex, exclude_regex, options, acc);
        if limit_reached {
            break;
        }
    }
}

/// Execute raw file search across all project files
pub fn raw_search(
    project_path: &Path,
    options: &RawSearchOptions,
) -> Result<RawSearchOutput, String> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let project_root = project_path
        .canonicalize()
        .unwrap_or_else(|_| project_path.to_path_buf());

    let (regex, exclude_regex) = build_search_patterns(options)?;
    let exclude_glob = build_exclude_glob(options);
    let lang_extensions = options.language_filter.map(language_to_extensions);

    let mut acc = FileMatchAccumulator {
        results: Vec::new(),
        file_matches: Vec::new(),
        file_counts: Vec::new(),
        total_results: 0,
    };

    walk_and_collect(
        &project_root,
        &regex,
        &exclude_regex,
        &lang_extensions,
        &exclude_glob,
        options,
        &mut acc,
    );

    if options.files_with_matches {
        Ok(RawSearchOutput::Files(acc.file_matches))
    } else if options.count_mode {
        acc.file_counts.sort_by(|a, b| b.count.cmp(&a.count));
        Ok(RawSearchOutput::Counts(acc.file_counts))
    } else {
        Ok(RawSearchOutput::Lines(acc.results))
    }
}

/// Check if a raw search result falls within an indexed function's line range.
/// Used for deduplication when merging index + raw results.
pub fn is_within_indexed_function(
    raw_file: &str,
    raw_line: usize,
    indexed_results: &[super::types::QueryResult],
) -> bool {
    indexed_results
        .iter()
        .any(|r| r.file_path == raw_file && raw_line >= r.start_line && raw_line <= r.end_line)
}

/// Directories to skip during raw search (beyond .gitignore)
fn is_search_ignored_dir(path: &Path) -> bool {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    path.components().any(|c| {
        let s = c.as_os_str().to_str().unwrap_or("");
        matches!(
            s,
            "target"
                | "node_modules"
                | ".git"
                | ".pmat"
                | "__pycache__"
                | "venv"
                | ".venv"
                | "dist"
                | ".next"
                | ".cache"
                | "vendor"
                | "third_party"
                | "third-party"
                | ".cargo"
        )
    })
}

/// Map language name to file extensions for filtering
fn language_to_extensions(lang: &str) -> Vec<&'static str> {
    match lang.to_lowercase().as_str() {
        "rust" | "rs" => vec!["rs"],
        "python" | "py" => vec!["py", "pyi"],
        "typescript" | "ts" => vec!["ts", "tsx"],
        "javascript" | "js" => vec!["js", "jsx", "mjs"],
        "go" => vec!["go"],
        "java" => vec!["java"],
        "c" => vec!["c", "h"],
        "cpp" | "c++" | "cxx" => vec!["cpp", "cxx", "cc", "hpp", "hxx", "h"],
        "ruby" | "rb" => vec!["rb"],
        "toml" => vec!["toml"],
        "yaml" | "yml" => vec!["yaml", "yml"],
        "json" => vec!["json"],
        "markdown" | "md" => vec!["md", "markdown"],
        "shell" | "bash" | "sh" => vec!["sh", "bash"],
        "makefile" | "make" => vec!["mk", "makefile"],
        _ => vec![],
    }
}
