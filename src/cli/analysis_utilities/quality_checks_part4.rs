
/// Toyota Way: Extract Method - Get QG violation summary data rows (complexity ≤3)
fn get_qg_violation_summary_rows(results: &QualityGateResults) -> [(&'static str, u64); 9] {
    [
        (
            "Complexity",
            results.complexity_violations.try_into().unwrap_or(0),
        ),
        (
            "Dead Code",
            results.dead_code_violations.try_into().unwrap_or(0),
        ),
        ("SATD", results.satd_violations.try_into().unwrap_or(0)),
        (
            "Entropy",
            results.entropy_violations.try_into().unwrap_or(0),
        ),
        (
            "Security",
            results.security_violations.try_into().unwrap_or(0),
        ),
        (
            "Duplicates",
            results.duplicate_violations.try_into().unwrap_or(0),
        ),
        (
            "Coverage",
            results.coverage_violations.try_into().unwrap_or(0),
        ),
        (
            "Sections",
            results.section_violations.try_into().unwrap_or(0),
        ),
        (
            "Provability",
            results.provability_violations.try_into().unwrap_or(0),
        ),
    ]
}

// Helper functions
#[must_use]
pub fn detect_toolchain(path: &Path) -> Option<String> {
    super::detect_primary_language(path)
}

#[must_use]
pub fn build_complexity_thresholds(
    max_cyclomatic: Option<u16>,
    max_cognitive: Option<u16>,
) -> (u16, u16) {
    (max_cyclomatic.unwrap_or(10), max_cognitive.unwrap_or(15))
}

/// Analyzes project files for complexity metrics using a systematic approach.
///
/// This function walks through a project directory, filtering files based on toolchain
/// and include patterns, then analyzes each applicable file for complexity metrics.
/// The implementation follows Toyota Way principles by breaking down complexity into
/// focused, single-responsibility helper functions.
///
/// # Arguments
///
/// * `project_path` - Root directory of the project to analyze
/// * `toolchain` - Optional toolchain specifier ("rust", "typescript", "python", etc.)
/// * `include` - Patterns for files to include in analysis (empty = use defaults)
/// * `cyclomatic_threshold` - Threshold for cyclomatic complexity warnings
/// * `cognitive_threshold` - Threshold for cognitive complexity warnings
///
/// # Returns
///
/// A `Result` containing a vector of `FileComplexityMetrics` for each analyzed file.
///
/// # Examples
///
/// ```ignore
/// use pmat::cli::analysis_utilities::analyze_project_files;
/// use std::path::Path;
///
/// # async fn example() -> anyhow::Result<()> {
/// let project_path = Path::new(".");
/// let metrics = analyze_project_files(
///     project_path,
///     Some("rust"),
///     &[],
///     10,
///     15
/// ).await?;
///
/// assert!(metrics.len() >= 0);
/// # Ok(())
/// # }
/// ```ignore
///
/// # Quality Improvements
///
/// This function was refactored from a monolithic implementation (complexity 40)
/// into focused helper functions, achieving:
/// - Reduced cyclomatic complexity from 40 to <8
/// - Improved readability through single-responsibility functions
/// - Better maintainability following Toyota Way Kaizen principles
pub async fn analyze_project_files(
    project_path: &Path,
    toolchain: Option<&str>,
    include: &[String],
    cyclomatic_threshold: u16,
    cognitive_threshold: u16,
) -> Result<Vec<crate::services::complexity::FileComplexityMetrics>> {
    use crate::services::file_discovery::{FileDiscoveryConfig, ProjectFileDiscovery};

    // CRITICAL FIX: Use ProjectFileDiscovery instead of WalkDir
    // This ensures .pmatignore and .paimlignore files are respected
    // Bug: Previously used walkdir directly, bypassing ignore file support
    let discovery_config = FileDiscoveryConfig {
        respect_gitignore: true, // Respect .gitignore, .pmatignore, .paimlignore
        ..Default::default()
    };

    let discovery =
        ProjectFileDiscovery::new(project_path.to_path_buf()).with_config(discovery_config);

    // Discover all files using the intelligent file discovery service
    let discovered_files = discovery.discover_files()?;

    // CRITICAL: ProjectFileDiscovery already handles exclusions via .gitignore/.pmatignore
    // We only need to filter by extension and include patterns here
    let extensions = get_file_extensions(toolchain);

    // Filter discovered files ONLY by extension and include patterns
    // Do NOT use should_analyze_file() as it has is_excluded_path() which filters /tmp/
    let files_to_analyze: Vec<_> = discovered_files
        .into_iter()
        .filter(|path| {
            // Check extension
            let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
            if !extensions.contains(&extension) {
                return false;
            }

            // Check include patterns (if specified)
            if !include.is_empty() {
                matches_include_patterns(path, project_path, include)
            } else {
                true // No include patterns, accept all files with correct extension
            }
        })
        .collect();

    // PERFORMANCE OPTIMIZATION: Process files in parallel batches
    // Return early if no files to analyze
    if files_to_analyze.is_empty() {
        return Ok(Vec::new());
    }

    let batch_size = std::cmp::min(files_to_analyze.len(), 20); // Optimize batch size
    let mut results = Vec::new();

    for batch in files_to_analyze.chunks(batch_size) {
        let batch_futures: Vec<_> = batch
            .iter()
            .map(|path| analyze_complexity_file(path, cyclomatic_threshold, cognitive_threshold))
            .collect();

        let batch_results = futures::future::try_join_all(batch_futures).await?;

        for metrics in batch_results.into_iter().flatten() {
            results.push(metrics);
        }
    }

    Ok(results)
}

/// Get file extensions for the specified toolchain.
///
/// Maps toolchain identifiers to their corresponding file extensions.
/// Supports multiple programming languages and defaults to Rust.
///
/// # Arguments
///
/// * `toolchain` - Optional toolchain identifier
///
/// # Returns
///
/// Vector of file extensions to analyze for the given toolchain
///
/// # Examples
///
/// ```ignore
/// # use pmat::cli::analysis_utilities::get_file_extensions;
/// let rust_extensions = get_file_extensions(Some("rust"));
/// assert_eq!(rust_extensions, vec!["rs"]);
///
/// let ts_extensions = get_file_extensions(Some("typescript"));
/// assert_eq!(ts_extensions, vec!["ts", "tsx", "js", "jsx"]);
///
/// let default_extensions = get_file_extensions(None);
/// assert_eq!(default_extensions, vec!["rs"]);
/// ```ignore
#[must_use]
pub fn get_file_extensions(toolchain: Option<&str>) -> Vec<&'static str> {
    match toolchain {
        Some("rust") => vec!["rs"],
        Some("deno" | "typescript") => vec!["ts", "tsx", "js", "jsx"],
        Some("javascript") => vec!["js", "jsx"], // PMAT-BUG-002 fix
        Some("python-uv" | "python") => vec!["py"],
        Some("c") => vec!["c", "h"], // PMAT-BUG-003 fix
        Some("cpp" | "c++") => vec!["cpp", "cc", "cxx", "hpp", "h", "hxx"], // PMAT-BUG-004 fix
        Some("go") => vec!["go"],
        Some("java") => vec!["java"],
        Some("kotlin") => vec!["kt", "kts"],
        Some("ruby") => vec!["rb"],
        Some("php") => vec!["php"],
        Some("swift") => vec!["swift"],
        Some("csharp" | "cs") => vec!["cs"],
        Some("bash") => vec!["sh", "bash"],
        Some(_) => vec!["rs"], // unknown toolchain defaults to rust
        None => {
            // Issue #42 fix: When no toolchain detected, analyze ALL supported languages
            vec![
                "rs", "py", "ts", "tsx", "js", "jsx", "go", "java", "kt", "kts", "c", "cpp", "cc",
                "cxx", "rb", "php", "swift", "cs",
            ]
        }
    }
}

/// Check if a file should be analyzed based on extension, patterns, and exclusions.
///
/// This function implements the filtering logic for determining whether a file
/// should be included in complexity analysis, based on file extension,
/// include patterns, and standard exclusions.
///
/// # Arguments
///
/// * `path` - The file path to evaluate
/// * `project_path` - Root project directory
/// * `extensions` - Allowed file extensions
/// * `include` - Include patterns (if empty, uses default exclusions)
///
/// # Returns
///
/// `true` if the file should be analyzed, `false` otherwise
#[must_use]
pub fn should_analyze_file(
    path: &Path,
    project_path: &Path,
    extensions: &[&str],
    include: &[String],
) -> bool {
    let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");

    if !extensions.contains(&extension) {
        return false;
    }

    if include.is_empty() {
        !is_excluded_path(path)
    } else {
        matches_include_patterns(path, project_path, include)
    }
}

/// Check if path matches any of the include patterns
fn matches_include_patterns(path: &Path, project_path: &Path, include: &[String]) -> bool {
    use glob::Pattern;

    let path_str = path.to_string_lossy();
    let relative_path = path.strip_prefix(project_path).unwrap_or(path);
    let relative_str = relative_path.to_string_lossy();

    include.iter().any(|pattern| match Pattern::new(pattern) {
        Ok(glob_pattern) => glob_pattern.matches(&relative_str) || glob_pattern.matches(&path_str),
        Err(_) => path_str.contains(pattern),
    })
}

/// Check if path should be excluded from analysis
fn is_excluded_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy();

    if is_excluded_directory(&path_str) {
        return true;
    }

    if let Some(file_name) = path.file_name() {
        let fname = file_name.to_string_lossy();
        is_excluded_filename(&fname)
    } else {
        false
    }
}

/// Check if path contains excluded directories
pub fn is_excluded_directory(path_str: &str) -> bool {
    // Normalize path for consistent matching
    let normalized = path_str.replace('\\', "/");

    // Directory name patterns to exclude (gitignore-style)
    let excluded_dir_names = [
        "target",
        "build",
        "out",
        ".cargo",
        "node_modules",
        "dist",
        ".git",
        "vendor",
        "generated",
        ".aws-sam",
        "coverage",
        "__pycache__",
        ".pytest_cache",
        ".cache",
        "tmp",
        ".venv",
        "venv",
        "ENV",
        "env",
        ".terraform",
        "site",
        "_site",
        ".jekyll-cache",
        ".idea",
        ".vscode",
    ];

    // Path patterns that should be excluded
    let excluded_path_patterns = [
        "/target/",
        "/build/",
        "/out/",
        "/.cargo/",
        "/node_modules/",
        "/dist/",
        "/.git/",
        "/vendor/",
        "/generated/",
        "/.aws-sam/",
        "/coverage/",
        "/__pycache__/",
        "/.pytest_cache/",
        "/.cache/",
        "/tmp/",
        "/.venv/",
        "/venv/",
        "/ENV/",
        "/env/",
        "/.terraform/",
        "/site/",
        "/_site/",
        "/.jekyll-cache/",
        "/.idea/",
        "/.vscode/",
        "/tests/",
        "/test/",
        "/examples/",
        "/benches/",
        "/benchmarks/",
        "/fixtures/",
        "/testdata/",
        "/test_data/",
        "/debug_test/",
        "/test-",
    ];

    // Check if the path contains any excluded directory patterns
    if excluded_path_patterns
        .iter()
        .any(|pattern| normalized.contains(pattern))
    {
        return true;
    }

    // Check if path starts with excluded directories (./target, target/, etc.)
    let path_components: Vec<&str> = normalized.trim_start_matches("./").split('/').collect();
    if let Some(first_component) = path_components.first() {
        if excluded_dir_names.contains(first_component) {
            return true;
        }
    }

    false
}

/// Check if filename indicates a test file
#[must_use]
pub fn is_excluded_filename(filename: &str) -> bool {
    is_test_file(filename)
        || is_example_or_demo_file(filename)
        || is_benchmark_file(filename)
        || is_mock_or_stub_file(filename)
}

/// Check if filename is a test file (cognitive complexity ≤6)
pub fn is_test_file(filename: &str) -> bool {
    const TEST_SUFFIXES: &[&str] = &["_test.rs", "_tests.rs", "tests.rs"];
    const TEST_PREFIXES: &[&str] = &["test_", "tests_"];
    const TEST_CONTAINS: &[&str] = &[
        "_test_",
        "_tests_",
        "test_harness",
        "test_helpers",
        "test_utils",
        "_property_test",
        "property_tests",
    ];

    TEST_SUFFIXES.iter().any(|s| filename.ends_with(s))
        || TEST_PREFIXES.iter().any(|p| filename.starts_with(p))
        || TEST_CONTAINS.iter().any(|c| filename.contains(c))
}

/// Check if filename is an example or demo file (cognitive complexity ≤4)
pub fn is_example_or_demo_file(filename: &str) -> bool {
    const EXAMPLE_DEMO_PREFIXES: &[&str] = &["example_", "demo_"];
    const EXAMPLE_DEMO_CONTAINS: &[&str] = &["_example", "_demo"];

    EXAMPLE_DEMO_PREFIXES
        .iter()
        .any(|p| filename.starts_with(p))
        || EXAMPLE_DEMO_CONTAINS.iter().any(|c| filename.contains(c))
}

/// Check if filename is a benchmark file (cognitive complexity ≤4)
pub fn is_benchmark_file(filename: &str) -> bool {
    const BENCH_SUFFIXES: &[&str] = &["_bench.rs", "_benchmark.rs"];
    const BENCH_CONTAINS: &[&str] = &["bench_", "benchmark_"];

    BENCH_SUFFIXES.iter().any(|s| filename.ends_with(s))
        || BENCH_CONTAINS.iter().any(|c| filename.contains(c))
}

/// Check if filename is a mock or stub file (cognitive complexity ≤4)
pub fn is_mock_or_stub_file(filename: &str) -> bool {
    const MOCK_STUB_PREFIXES: &[&str] = &["mock_", "stub_", "stubs_"];
    const MOCK_STUB_CONTAINS: &[&str] = &["_mock", "_stub", "_stubs"];

    MOCK_STUB_PREFIXES.iter().any(|p| filename.starts_with(p))
        || MOCK_STUB_CONTAINS.iter().any(|c| filename.contains(c))
}

/// Analyze a single file for complexity metrics
async fn analyze_complexity_file(
    path: &Path,
    cyclomatic_threshold: u16,
    cognitive_threshold: u16,
) -> Result<Option<crate::services::complexity::FileComplexityMetrics>> {
    // PERFORMANCE OPTIMIZATION: Use async file I/O instead of blocking
    match tokio::fs::read_to_string(path).await {
        Ok(content) => {
            let metrics = analyze_file_complexity_async(
                path,
                &content,
                cyclomatic_threshold,
                cognitive_threshold,
            )
            .await?;
            Ok(Some(metrics))
        }
        Err(_) => Ok(None),
    }
}

async fn analyze_file_complexity_async(
    path: &Path,
    content: &str,
    _cyclomatic_threshold: u16,
    _cognitive_threshold: u16,
) -> Result<crate::services::complexity::FileComplexityMetrics> {
    crate::cli::language_analyzer::analyze_file_complexity(path, content).await
}

#[must_use]
pub fn add_top_files_ranking(
    files: Vec<crate::services::complexity::FileComplexityMetrics>,
    top_files: usize,
) -> Vec<crate::services::complexity::FileComplexityMetrics> {
    if top_files == 0 {
        files
    } else {
        files.into_iter().take(top_files).collect()
    }
}

pub fn format_dead_code_output(
    format: DeadCodeOutputFormat,
    dead_code_result: &crate::models::dead_code::DeadCodeResult,
    _output: Option<PathBuf>,
) -> Result<()> {
    crate::cli::dead_code_formatter::format_and_output_dead_code(format, dead_code_result, _output)
}

