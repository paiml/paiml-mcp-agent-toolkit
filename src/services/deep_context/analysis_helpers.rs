// Analysis helper functions - extracted for file health (CB-040)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) async fn analyze_ast_contexts(
    path: &std::path::Path,
    _config: Option<FileClassifierConfig>,
) -> anyhow::Result<Vec<EnhancedFileContext>> {
    let _start_time = std::time::Instant::now();
    info!("Starting AST analysis for path: {:?}", path);

    let source_files = discover_and_categorize_source_files(path)?;
    let enhanced_contexts = analyze_source_files_for_contexts(source_files).await?;

    info!(
        "AST analysis completed. Generated {} file contexts",
        enhanced_contexts.len()
    );
    Ok(enhanced_contexts)
}

/// Discover files and filter for source files only
fn discover_and_categorize_source_files(path: &std::path::Path) -> anyhow::Result<Vec<PathBuf>> {
    use crate::services::file_discovery::ProjectFileDiscovery;

    let discovery_config = create_ast_discovery_config();
    let discovery = ProjectFileDiscovery::new(path.to_path_buf()).with_config(discovery_config);
    let all_files = discovery.discover_files()?;

    let categorized_files = categorize_files_in_parallel(all_files);
    let source_files = filter_and_categorize_files(categorized_files);

    Ok(source_files)
}

/// Create discovery configuration for AST analysis
fn create_ast_discovery_config() -> crate::services::file_discovery::FileDiscoveryConfig {
    crate::services::file_discovery::FileDiscoveryConfig {
        respect_gitignore: true,
        filter_external_repos: true,
        max_files: Some(10_000), // Reasonable limit for AST analysis
        ..Default::default()
    }
}

/// Categorize files in parallel for better performance
fn categorize_files_in_parallel(
    all_files: Vec<PathBuf>,
) -> Vec<(PathBuf, crate::services::file_discovery::FileCategory)> {
    use crate::services::file_discovery::ProjectFileDiscovery;

    all_files
        .into_par_iter()
        .map(|file_path| {
            let category = ProjectFileDiscovery::categorize_file(&file_path);
            (file_path, category)
        })
        .collect()
}

/// Filter categorized files to extract only source files.
/// Skips test files (*_tests.rs, *_test.rs, tests/*) since they don't contribute
/// to complexity/provability/DAG analysis and avoid ~475 MB of syn parsing.
fn filter_and_categorize_files(
    categorized_files: Vec<(PathBuf, crate::services::file_discovery::FileCategory)>,
) -> Vec<PathBuf> {
    use crate::services::file_discovery::FileCategory;

    let mut source_files = Vec::new();
    let mut skipped_files = 0;
    let mut skipped_test_files = 0;

    for (file_path, category) in categorized_files {
        match category {
            FileCategory::SourceCode => {
                // Skip test files from deep context AST analysis — they are noise
                // for complexity, provability, and DAG phases, and parsing them
                // with syn wastes ~475 MB of allocations.
                if is_test_file(&file_path) {
                    skipped_test_files += 1;
                    continue;
                }
                source_files.push(file_path);
            }
            FileCategory::GeneratedOutput | FileCategory::TestArtifact => {
                skipped_files += 1;
                debug!("Skipping generated/test file: {:?}", file_path);
            }
            FileCategory::EssentialDoc | FileCategory::BuildConfig => {
                debug!("Will compress metadata file: {:?}", file_path);
            }
            FileCategory::DevelopmentDoc => {
                debug!("Skipping development doc: {:?}", file_path);
            }
        }
    }

    info!(
        "Discovered {} source files for AST analysis (skipped {} generated + {} test files)",
        source_files.len(),
        skipped_files,
        skipped_test_files
    );

    source_files
}

/// Check if a file is a test file based on naming conventions.
/// Matches: *_tests.rs, *_test.rs, tests/*.rs, test_*.rs
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn is_test_file(path: &std::path::Path) -> bool {
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    // Common Rust test file patterns
    if file_name.ends_with("_tests.rs")
        || file_name.ends_with("_test.rs")
        || file_name.starts_with("test_")
    {
        return true;
    }

    // Files in tests/ directory
    for component in path.components() {
        if let std::path::Component::Normal(c) = component {
            if c == "tests" {
                return true;
            }
        }
    }

    false
}

/// Analyze source files and create enhanced contexts
async fn analyze_source_files_for_contexts(
    source_files: Vec<PathBuf>,
) -> anyhow::Result<Vec<EnhancedFileContext>> {
    let mut enhanced_contexts = Vec::new();
    let mut file_count = 0;
    let analysis_start = std::time::Instant::now();
    let discovered = source_files.len();
    let mut failures: Vec<(PathBuf, String)> = Vec::new();

    for file_path in source_files {
        match analyze_single_file_for_context(&file_path, &mut file_count).await {
            Ok(enhanced_context) => enhanced_contexts.push(enhanced_context),
            Err(e) => failures.push((file_path, e.to_string())),
        }
    }

    report_parse_failures(discovered, &failures);

    log_analysis_completion(analysis_start, file_count);
    Ok(enhanced_contexts)
}

/// Report files whose AST parse failed.
///
/// A crate whose every file fails to parse used to render exactly like an empty
/// directory — "Total Files: 0", no warning, exit 0 — because the parse error was
/// dropped by an `if let Ok(..)` with no else branch. Warnings go to stderr because
/// the tracing subscriber is off in several of the modes that generate context.
fn report_parse_failures(discovered: usize, failures: &[(PathBuf, String)]) {
    if failures.is_empty() {
        return;
    }
    const MAX_LISTED: usize = 10;
    for (path, err) in failures.iter().take(MAX_LISTED) {
        eprintln!("⚠️  failed to parse {}: {}", path.display(), err);
    }
    if failures.len() > MAX_LISTED {
        eprintln!("⚠️  … and {} more", failures.len() - MAX_LISTED);
    }
    eprintln!(
        "⚠️  {} of {} discovered source file(s) failed to parse and are absent from this context",
        failures.len(),
        discovered
    );
}

/// Analyze single file and create enhanced context if successful
/// Returns the parse error instead of swallowing it: the caller counts and reports
/// unparseable files so they cannot vanish from the context without a word.
async fn analyze_single_file_for_context(
    file_path: &Path,
    file_count: &mut usize,
) -> anyhow::Result<EnhancedFileContext> {
    let file_start = std::time::Instant::now();

    let file_context = analysis_functions::analyze_single_file(file_path).await?;
    let ast_time = file_start.elapsed();

    if (*file_count).is_multiple_of(10) {
        info!(
            "Progress: {} files processed. Last file - AST: {:?}",
            file_count, ast_time
        );
    }

    let enhanced_context = EnhancedFileContext {
        base: file_context,
        complexity_metrics: None,
        churn_metrics: None,
        defects: DefectAnnotations {
            dead_code: None,
            technical_debt: Vec::new(),
            complexity_violations: Vec::new(),
            tdg_score: None, // Skip TDG calculation for context generation
        },
        symbol_id: uuid::Uuid::new_v4().to_string(),
    };

    *file_count += 1;
    Ok(enhanced_context)
}

/// Log analysis completion statistics
fn log_analysis_completion(analysis_start: std::time::Instant, file_count: usize) {
    let total_time = analysis_start.elapsed();
    info!(
        "AST analysis phase took {:?} for {} files ({:?} per file average)",
        total_time,
        file_count,
        total_time / file_count.max(1) as u32
    );
}


#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod analysis_helpers_tests {
    use super::*;

    /// An unparseable source file must surface its parse error, not be silently
    /// dropped: a crate where every file fails to parse used to render exactly like
    /// an empty directory.
    #[tokio::test]
    async fn test_analyze_single_file_for_context_reports_parse_error() {
        let tmp = tempfile::TempDir::new().unwrap();
        let broken = tmp.path().join("main.rs");
        std::fs::write(&broken, "fn main( { let x = ;;;\n").unwrap();

        let mut count = 0usize;
        let result = analyze_single_file_for_context(&broken, &mut count).await;
        assert!(
            result.is_err(),
            "unparseable file must yield an error, not a silent skip"
        );
        assert_eq!(count, 0, "a failed file must not be counted as analyzed");
    }

    #[tokio::test]
    async fn test_analyze_single_file_for_context_ok_on_valid_rust() {
        let tmp = tempfile::TempDir::new().unwrap();
        let good = tmp.path().join("main.rs");
        std::fs::write(&good, "fn main() {}\n").unwrap();

        let mut count = 0usize;
        let result = analyze_single_file_for_context(&good, &mut count).await;
        assert!(result.is_ok(), "valid rust must parse: {result:?}");
        assert_eq!(count, 1);
    }
}
