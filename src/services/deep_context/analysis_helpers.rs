// Analysis helper functions - extracted for file health (CB-040)
async fn analyze_ast_contexts(
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

/// Filter categorized files to extract only source files
fn filter_and_categorize_files(
    categorized_files: Vec<(PathBuf, crate::services::file_discovery::FileCategory)>,
) -> Vec<PathBuf> {
    use crate::services::file_discovery::FileCategory;

    let mut source_files = Vec::new();
    let mut skipped_files = 0;

    for (file_path, category) in categorized_files {
        match category {
            FileCategory::SourceCode => {
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
        "Discovered {} source files for AST analysis (skipped {} generated/test files)",
        source_files.len(),
        skipped_files
    );

    source_files
}

/// Analyze source files and create enhanced contexts
async fn analyze_source_files_for_contexts(
    source_files: Vec<PathBuf>,
) -> anyhow::Result<Vec<EnhancedFileContext>> {
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
    let total_time = analysis_start.elapsed();
    info!(
        "AST analysis phase took {:?} for {} files ({:?} per file average)",
        total_time,
        file_count,
        total_time / file_count.max(1) as u32
    );
}

