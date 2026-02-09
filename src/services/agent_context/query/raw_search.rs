#![cfg_attr(coverage_nightly, coverage(off))]
//! Raw file search — rg-compatible line-level search across all project files.
//!
//! Used as a fallback/complement to the AST-based function index when:
//! - Searching non-code files (TOML, YAML, Markdown, etc.)
//! - Finding module-level items (use, const, static, impl blocks)
//! - User wants pure line-level grep-like results (`--raw` mode)

use ignore::WalkBuilder;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// A single line match from raw file search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawSearchResult {
    /// File path relative to project root
    pub file_path: String,
    /// 1-based line number
    pub line_number: usize,
    /// The matching line content (trimmed trailing newline)
    pub line_content: String,
    /// Context lines before the match
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context_before: Vec<String>,
    /// Context lines after the match
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context_after: Vec<String>,
}

/// Options for raw search
pub struct RawSearchOptions<'a> {
    /// Regex pattern to search for
    pub pattern: &'a str,
    /// Whether to treat pattern as literal (no regex)
    pub literal: bool,
    /// Case-insensitive search
    pub case_insensitive: bool,
    /// Lines of context before match
    pub before_context: usize,
    /// Lines of context after match
    pub after_context: usize,
    /// Maximum results to return
    pub limit: usize,
    /// Filter to files matching this language extension
    pub language_filter: Option<&'a str>,
    /// Exclude files matching this glob pattern
    pub exclude_file_pattern: Option<&'a str>,
    /// Exclude results matching this content pattern
    pub exclude_pattern: Option<&'a str>,
    /// Only return file paths (like rg -l)
    pub files_with_matches: bool,
    /// Only return match counts per file (like rg -c)
    pub count_mode: bool,
}

/// Per-file match count for --count mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMatchCount {
    pub file_path: String,
    pub count: usize,
}

/// Result of a raw search operation
pub enum RawSearchOutput {
    /// Normal line matches
    Lines(Vec<RawSearchResult>),
    /// File paths only (--files-with-matches)
    Files(Vec<String>),
    /// Match counts per file (--count)
    Counts(Vec<FileMatchCount>),
}

/// Check whether a file should be skipped based on language filter and exclude glob.
fn should_skip_file(
    path: &Path,
    relative_path: &str,
    lang_extensions: &Option<Vec<&str>>,
    exclude_glob: &Option<globset::GlobSet>,
) -> bool {
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

    let context_after: Vec<String> = lines.get(i + 1..after_end)
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
    let has_match = lines.iter().any(|line| line_matches(line, regex, exclude_regex));
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
    let count = lines.iter().filter(|line| line_matches(line, regex, exclude_regex)).count();
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
            lines, relative_path, i, options.before_context, options.after_context,
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

    let exclude_regex = options
        .exclude_pattern
        .map(|p| Regex::new(&format!("(?i){}", regex::escape(p))))
        .transpose()
        .map_err(|e| format!("Invalid exclude pattern: {e}"))?;

    Ok((regex, exclude_regex))
}

/// Build the exclude file glob from options.
fn build_exclude_glob(options: &RawSearchOptions) -> Option<globset::GlobSet> {
    options.exclude_file_pattern.and_then(|g| {
        globset::GlobBuilder::new(&format!("**{g}**"))
            .case_insensitive(true)
            .build()
            .ok()
            .and_then(|gb| globset::GlobSetBuilder::new().add(gb).build().ok())
    })
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
        let limit_reached = collect_file_matches(&lines, &relative_path, regex, exclude_regex, options, acc);
        if limit_reached {
            break;
        }
    }
}

/// Execute raw file search across all project files
pub fn raw_search(project_path: &Path, options: &RawSearchOptions) -> Result<RawSearchOutput, String> {
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

    walk_and_collect(&project_root, &regex, &exclude_regex, &lang_extensions, &exclude_glob, options, &mut acc);

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
    indexed_results.iter().any(|r| {
        r.file_path == raw_file && raw_line >= r.start_line && raw_line <= r.end_line
    })
}

/// Directories to skip during raw search (beyond .gitignore)
fn is_search_ignored_dir(path: &Path) -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_project() -> TempDir {
        let dir = TempDir::new().unwrap();
        // Create a Rust file
        let src = dir.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("main.rs"),
            "use serde::Serialize;\n\nconst TIMEOUT: u32 = 30;\n\nfn main() {\n    println!(\"hello\");\n}\n",
        ).unwrap();
        // Create a TOML file
        fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\n\n[dependencies]\nserde = \"1.0\"\n",
        ).unwrap();
        // Create a markdown file
        fs::write(
            dir.path().join("README.md"),
            "# Test Project\n\nThis has a TIMEOUT of 30 seconds.\n",
        ).unwrap();
        dir
    }

    #[test]
    fn test_raw_search_literal() {
        let dir = create_test_project();
        let opts = RawSearchOptions {
            pattern: "TIMEOUT",
            literal: true,
            case_insensitive: false,
            before_context: 0,
            after_context: 0,
            limit: 100,
            language_filter: None,
            exclude_file_pattern: None,
            exclude_pattern: None,
            files_with_matches: false,
            count_mode: false,
        };
        let result = raw_search(dir.path(), &opts).unwrap();
        match result {
            RawSearchOutput::Lines(lines) => {
                assert!(lines.len() >= 2, "Should find TIMEOUT in .rs and .md");
                let files: Vec<&str> = lines.iter().map(|r| r.file_path.as_str()).collect();
                assert!(files.iter().any(|f| f.ends_with(".rs")));
                assert!(files.iter().any(|f| f.ends_with(".md")));
            }
            _ => panic!("Expected Lines output"),
        }
    }

    #[test]
    fn test_raw_search_regex() {
        let dir = create_test_project();
        let opts = RawSearchOptions {
            pattern: r"version\s*=",
            literal: false,
            case_insensitive: false,
            before_context: 0,
            after_context: 0,
            limit: 100,
            language_filter: None,
            exclude_file_pattern: None,
            exclude_pattern: None,
            files_with_matches: false,
            count_mode: false,
        };
        let result = raw_search(dir.path(), &opts).unwrap();
        match result {
            RawSearchOutput::Lines(lines) => {
                assert!(!lines.is_empty());
                assert!(lines[0].file_path.contains("Cargo.toml"));
            }
            _ => panic!("Expected Lines output"),
        }
    }

    #[test]
    fn test_raw_search_files_with_matches() {
        let dir = create_test_project();
        let opts = RawSearchOptions {
            pattern: "serde",
            literal: true,
            case_insensitive: false,
            before_context: 0,
            after_context: 0,
            limit: 100,
            language_filter: None,
            exclude_file_pattern: None,
            exclude_pattern: None,
            files_with_matches: true,
            count_mode: false,
        };
        let result = raw_search(dir.path(), &opts).unwrap();
        match result {
            RawSearchOutput::Files(files) => {
                assert!(files.len() >= 2, "serde in main.rs and Cargo.toml");
            }
            _ => panic!("Expected Files output"),
        }
    }

    #[test]
    fn test_raw_search_count_mode() {
        let dir = create_test_project();
        let opts = RawSearchOptions {
            pattern: "serde",
            literal: true,
            case_insensitive: false,
            before_context: 0,
            after_context: 0,
            limit: 100,
            language_filter: None,
            exclude_file_pattern: None,
            exclude_pattern: None,
            files_with_matches: false,
            count_mode: true,
        };
        let result = raw_search(dir.path(), &opts).unwrap();
        match result {
            RawSearchOutput::Counts(counts) => {
                assert!(!counts.is_empty());
                for c in &counts {
                    assert!(c.count > 0);
                }
            }
            _ => panic!("Expected Counts output"),
        }
    }

    #[test]
    fn test_raw_search_language_filter() {
        let dir = create_test_project();
        let opts = RawSearchOptions {
            pattern: "TIMEOUT",
            literal: true,
            case_insensitive: false,
            before_context: 0,
            after_context: 0,
            limit: 100,
            language_filter: Some("rust"),
            exclude_file_pattern: None,
            exclude_pattern: None,
            files_with_matches: false,
            count_mode: false,
        };
        let result = raw_search(dir.path(), &opts).unwrap();
        match result {
            RawSearchOutput::Lines(lines) => {
                assert!(!lines.is_empty());
                for l in &lines {
                    assert!(l.file_path.ends_with(".rs"), "Should only find .rs files");
                }
            }
            _ => panic!("Expected Lines output"),
        }
    }

    #[test]
    fn test_raw_search_context_lines() {
        let dir = create_test_project();
        let opts = RawSearchOptions {
            pattern: "TIMEOUT",
            literal: true,
            case_insensitive: false,
            before_context: 1,
            after_context: 1,
            limit: 100,
            language_filter: Some("rust"),
            exclude_file_pattern: None,
            exclude_pattern: None,
            files_with_matches: false,
            count_mode: false,
        };
        let result = raw_search(dir.path(), &opts).unwrap();
        match result {
            RawSearchOutput::Lines(lines) => {
                assert!(!lines.is_empty());
                let first = &lines[0];
                assert!(first.line_content.contains("TIMEOUT"));
                // Should have context
                assert!(!first.context_before.is_empty() || first.line_number == 1);
                assert!(!first.context_after.is_empty());
            }
            _ => panic!("Expected Lines output"),
        }
    }

    #[test]
    fn test_raw_search_case_insensitive() {
        let dir = create_test_project();
        let opts = RawSearchOptions {
            pattern: "timeout",
            literal: true,
            case_insensitive: true,
            before_context: 0,
            after_context: 0,
            limit: 100,
            language_filter: None,
            exclude_file_pattern: None,
            exclude_pattern: None,
            files_with_matches: false,
            count_mode: false,
        };
        let result = raw_search(dir.path(), &opts).unwrap();
        match result {
            RawSearchOutput::Lines(lines) => {
                // Should match TIMEOUT (uppercase) with case-insensitive search
                assert!(lines.len() >= 2);
            }
            _ => panic!("Expected Lines output"),
        }
    }

    #[test]
    fn test_raw_search_exclude_pattern() {
        let dir = create_test_project();
        let opts = RawSearchOptions {
            pattern: "serde",
            literal: true,
            case_insensitive: false,
            before_context: 0,
            after_context: 0,
            limit: 100,
            language_filter: None,
            exclude_file_pattern: None,
            exclude_pattern: Some("Serialize"),
            files_with_matches: false,
            count_mode: false,
        };
        let result = raw_search(dir.path(), &opts).unwrap();
        match result {
            RawSearchOutput::Lines(lines) => {
                // The "use serde::Serialize" line should be excluded
                for l in &lines {
                    assert!(!l.line_content.contains("Serialize"));
                }
            }
            _ => panic!("Expected Lines output"),
        }
    }

    #[test]
    fn test_is_within_indexed_function() {
        let indexed = vec![super::super::types::QueryResult {
            file_path: "src/main.rs".to_string(),
            function_name: "main".to_string(),
            signature: "fn main()".to_string(),
            definition_type: "function".to_string(),
            doc_comment: None,
            start_line: 5,
            end_line: 7,
            language: "rust".to_string(),
            tdg_score: 1.0,
            tdg_grade: "A".to_string(),
            complexity: 1,
            big_o: "O(1)".to_string(),
            satd_count: 0,
            loc: 3,
            relevance_score: 1.0,
            source: None,
            calls: Vec::new(),
            called_by: Vec::new(),
            pagerank: 0.0,
            in_degree: 0,
            out_degree: 0,
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            duplication_score: 0.0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(),
            line_coverage_pct: 0.0,
            lines_covered: 0,
            lines_total: 0,
            missed_lines: 0,
            impact_score: 0.0,
            coverage_status: String::new(),
            coverage_diff: 0.0,
            coverage_exclusion: Default::default(),
            coverage_excluded: false,
            cross_project_callers: 0,
        }];

        // Line 6 is within the function (5-7)
        assert!(is_within_indexed_function("src/main.rs", 6, &indexed));
        // Line 3 is outside the function
        assert!(!is_within_indexed_function("src/main.rs", 3, &indexed));
        // Different file
        assert!(!is_within_indexed_function("src/lib.rs", 6, &indexed));
    }
}
