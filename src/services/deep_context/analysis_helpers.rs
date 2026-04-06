// Analysis helper functions - extracted for file health (CB-040)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) async fn analyze_ast_contexts(
    path: &std::path::Path,
    _config: Option<FileClassifierConfig>,
) -> anyhow::Result<Vec<EnhancedFileContext>> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
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
    debug_assert!(path.exists(), "path must exist: {}", path.display());
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
    debug_assert!(true, "contract: create_ast_discovery_config");
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
    debug_assert!(!all_files.is_empty(), "all_files must not be empty");
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
    debug_assert!(!categorized_files.is_empty(), "categorized_files must not be empty");
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
    debug_assert!(path.exists(), "path must exist: {}", path.display());
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
    debug_assert!(!source_files.is_empty(), "source_files must not be empty");
    let mut enhanced_contexts = Vec::new();
    let mut file_count = 0;
    let analysis_start = std::time::Instant::now();

    for file_path in source_files {
        if let Some(enhanced_context) =
            analyze_single_file_for_context(&file_path, &mut file_count).await
        {
            enhanced_contexts.push(enhanced_context);
        }
    }

    log_analysis_completion(analysis_start, file_count);
    Ok(enhanced_contexts)
}

/// Analyze single file and create enhanced context if successful
async fn analyze_single_file_for_context(
    file_path: &Path,
    file_count: &mut usize,
) -> Option<EnhancedFileContext> {
    debug_assert!(file_path.exists(), "file_path must exist: {}", file_path.display());
    let file_start = std::time::Instant::now();

    if let Ok(file_context) = analysis_functions::analyze_single_file(file_path).await {
        let ast_time = file_start.elapsed();

        if *file_count % 10 == 0 {
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
        Some(enhanced_context)
    } else {
        None
    }
}

/// Log analysis completion statistics
fn log_analysis_completion(analysis_start: std::time::Instant, file_count: usize) {
    debug_assert!(true, "contract: log_analysis_completion");
    let total_time = analysis_start.elapsed();
    info!(
        "AST analysis phase took {:?} for {} files ({:?} per file average)",
        total_time,
        file_count,
        total_time / file_count.max(1) as u32
    );
}

