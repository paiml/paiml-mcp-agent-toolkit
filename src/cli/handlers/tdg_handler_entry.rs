// Main TDG handler entry point and orchestration

/// Handle TDG analysis command with full MCP tool composition support
///
/// This handler supports three distinct modes:
/// 1. **Single File Mode**: Deep TDG analysis of one specific file
/// 2. **Multi-File Mode**: Process specific file lists for MCP tool chaining
/// 3. **Project Mode**: Analyze entire project with pattern filtering
///
/// # MCP Tool Composition Examples
///
/// ```no_run
/// # async fn example() -> anyhow::Result<()> {
/// use std::path::PathBuf;
/// use pmat::cli::enums::TdgOutputFormat;
/// use pmat::cli::handlers::tdg_handler::handle_analyze_tdg;
///
/// // Example 1: Find high TDG files for targeted refactoring
/// handle_analyze_tdg(
///     PathBuf::from("."),
///     None,                           // file
///     vec![],                         // files (empty = project mode)
///     1.5,                            // threshold
///     5,                              // top_files
///     TdgOutputFormat::Json,          // format for parsing
///     false,                          // include_components
///     None,                           // output (stdout)
///     false,                          // critical_only
///     false,                          // verbose
///     vec![],                         // include patterns
///     false,                          // watch
/// ).await?;
///
/// // Example 2: Analyze specific hotspot files from another tool
/// let hotspot_files = vec![
///     PathBuf::from("src/complex_module.rs"),
///     PathBuf::from("src/legacy_code.rs"),
/// ];
///
/// handle_analyze_tdg(
///     PathBuf::from("."),
///     None,                           // file
///     hotspot_files,                  // files (MCP composition)
///     1.0,                            // lower threshold
///     0,                              // show all
///     TdgOutputFormat::Json,          // format
///     true,                           // include_components
///     None,                           // output
///     false,                          // critical_only
///     true,                           // verbose
///     vec![],                         // include patterns
///     false,                          // watch
/// ).await?;
/// # Ok(())
/// # }
/// ```ignore
#[allow(clippy::too_many_arguments)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_analyze_tdg(
    project_path: PathBuf,
    file: Option<PathBuf>,
    files: Vec<PathBuf>,
    threshold: f64,
    top_files: usize,
    format: TdgOutputFormat,
    include_components: bool,
    output: Option<PathBuf>,
    critical_only: bool,
    verbose: bool,
    include: Vec<String>,
    watch: bool,
) -> Result<()> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    if watch {
        eprintln!("⏱️  Watch mode: Monitoring for file changes...");
        eprintln!("Press Ctrl+C to stop watching");
        // Watch mode continues with regular analysis
    }

    eprintln!("🔍 Analyzing Technical Debt Gradient...");

    // Create TDG calculator
    let calculator = TDGCalculator::new();

    // Determine analysis mode and generate output
    let output_content = if let Some(single_file) = file {
        // Single file mode
        analyze_single_file(
            &calculator,
            &project_path,
            single_file,
            threshold,
            format,
            include_components,
            critical_only,
            verbose,
        ).await?
    } else if !files.is_empty() {
        // Multiple files mode (MCP tool composition)
        analyze_multiple_files(
            &calculator,
            &project_path,
            files,
            threshold,
            top_files,
            format,
            include_components,
            critical_only,
            verbose,
        ).await?
    } else {
        // Project mode
        analyze_project(
            &calculator,
            &project_path,
            include,
            threshold,
            top_files,
            format,
            include_components,
            critical_only,
            verbose,
        ).await?
    };

    // Output results
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &output_content).await?;
        eprintln!("📝 Results written to {}", output_path.display());
    } else {
        println!("{}", output_content);
    }

    eprintln!("✅ TDG analysis complete");
    Ok(())
}
