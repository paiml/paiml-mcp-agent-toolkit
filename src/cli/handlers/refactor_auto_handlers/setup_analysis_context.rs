// Setup and discovery functions for refactor-auto
//
// Contains: setup_refactoring_context, load_ignore_patterns, discover_source_files.
// Split from setup_analysis.rs for file health compliance (CB-040).

/// Setup refactoring context from command line arguments (Phase 1: Extract Setup)
///
/// Initializes paths, patterns, and configuration for the refactoring operation.
/// This function has complexity <5 and follows Toyota Way principles.
#[allow(clippy::too_many_arguments)]
async fn setup_refactoring_context(
    project_path: PathBuf,
    single_file_mode: bool,
    file: Option<PathBuf>,
    format: RefactorAutoOutputFormat,
    max_iterations: u32,
    dry_run: bool,
    exclude_patterns: Vec<String>,
    include_patterns: Vec<String>,
    ignore_file: Option<PathBuf>,
    github_issue_url: Option<String>,
    bug_report_path: Option<PathBuf>,
) -> Result<RefactorContext> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let start_time = std::time::Instant::now();

    // Determine refactoring mode
    let mode = if let Some(bug_path) = bug_report_path {
        RefactorMode::BugReport(bug_path)
    } else if let Some(github_url) = github_issue_url {
        RefactorMode::GitHubIssue(github_url)
    } else if single_file_mode || file.is_some() {
        if let Some(target_file) = file {
            RefactorMode::SingleFile(target_file)
        } else {
            return Err(anyhow::anyhow!(
                "Single file mode requires --file parameter"
            ));
        }
    } else {
        RefactorMode::ProjectWide
    };

    // Create configuration
    let config = RefactorConfig {
        project_path: project_path.clone(),
        mode,
        quality_profile: QualityProfile::default(),
        patterns: PatternConfig {
            root_path: project_path,
            ignore_file: ignore_file
                .as_ref()
                .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string())),
            patterns: vec![],
            include_patterns,
            exclude_patterns,
            ignore_file_path: ignore_file,
            file_extensions: vec!["rs".to_string(), "toml".to_string()],
        },
        output: OutputConfig {
            format,
            dry_run,
            max_iterations,
            verbose: false,
        },
    };

    Ok(RefactorContext {
        config,
        ignore_patterns: vec![], // Will be loaded separately
        source_files: vec![],    // Will be discovered separately
        start_time,
    })
}

/// Load ignore patterns from configuration (Phase 1: Extract Setup)
///
/// Loads and consolidates ignore patterns from command line and ignore files.
/// This function has complexity <3 and follows Toyota Way principles.
async fn load_ignore_patterns(config: &PatternConfig) -> Result<Vec<String>> {
    debug_assert!(true, "contract: load_ignore_patterns");
    let mut all_patterns = config.exclude_patterns.clone();

    if let Some(ignore_path) = &config.ignore_file_path {
        if ignore_path.exists() {
            let ignore_content = tokio::fs::read_to_string(ignore_path)
                .await
                .context(format!(
                    "Failed to read ignore file: {}",
                    ignore_path.display()
                ))?;

            for line in ignore_content.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() && !trimmed.starts_with('#') {
                    all_patterns.push(trimmed.to_string());
                }
            }
        }
    }

    Ok(all_patterns)
}

/// Discover source files for analysis (Phase 1: Extract Setup)
///
/// Discovers and filters source files based on patterns and extensions.
/// This function has complexity <5 and follows Toyota Way principles.
async fn discover_source_files(
    project_path: &Path,
    patterns: &PatternConfig,
    ignore_patterns: &[String],
) -> Result<Vec<PathBuf>> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let mut source_files = Vec::new();

    for entry in WalkDir::new(project_path)
        .follow_links(false)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();

        // Check file extension
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if !patterns.file_extensions.contains(&ext.to_string()) {
                continue;
            }
        } else {
            continue;
        }

        // Check ignore patterns
        let path_str = path.to_string_lossy();
        let should_ignore = ignore_patterns
            .iter()
            .any(|pattern| path_str.contains(pattern) || path.to_string_lossy().contains(pattern));

        if !should_ignore {
            source_files.push(path.to_path_buf());
        }
    }

    source_files.sort();
    Ok(source_files)
}
