//! Complexity analysis command handlers with refactored dead code handler
//!
//! This module contains all complexity-related command implementations
//! extracted from the main CLI module to reduce cognitive complexity.

mod analysis;
mod churn;
mod output;
mod satd;
mod watch;

// Re-export public items
pub use churn::handle_analyze_churn;
pub use output::format_satd_summary;
pub use satd::handle_analyze_satd;

use crate::cli::{ComplexityOutputFormat, DagType};
use anyhow::Result;
use std::path::PathBuf;

// Re-export submodule items for tests
#[cfg(test)]
pub(crate) use analysis::{analyze_multiple_files, analyze_single_file, has_complexity_violations};

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod complexity_handlers_tests;

/// Configuration for complexity analysis operations
///
/// This struct centralizes all configuration parameters and provides
/// helper methods to reduce the complexity of the main handler function.
/// Following Toyota Way single responsibility principle.
#[derive(Debug, Clone)]
pub(crate) struct ComplexityConfig {
    project_path: PathBuf,
    toolchain: Option<String>,
    max_cyclomatic: u16,
    max_cognitive: u16,
    include: Vec<String>,
    timeout: u64,
    top_files: usize,
}

impl ComplexityConfig {
    /// Create configuration from CLI arguments
    fn from_args(
        project_path: PathBuf,
        toolchain: Option<String>,
        max_cyclomatic: Option<u16>,
        max_cognitive: Option<u16>,
        include: Vec<String>,
        timeout: u64,
        top_files: usize,
    ) -> Self {
        Self {
            project_path,
            toolchain,
            max_cyclomatic: max_cyclomatic.unwrap_or(10),
            max_cognitive: max_cognitive.unwrap_or(15),
            include,
            timeout,
            top_files,
        }
    }

    /// Detect toolchain for the project, returning detected toolchain or None for multi-language
    fn detect_toolchain(&self) -> Option<String> {
        self.toolchain
            .clone()
            .or_else(|| crate::cli::analysis_utilities::detect_toolchain(&self.project_path))
    }
}

/// Handle complexity analysis command with MCP tool composition support
///
/// This function enables AI agents to perform sophisticated code analysis workflows
/// by supporting three distinct modes of operation:
///
/// 1. **Project Mode**: Analyze entire project using include patterns
/// 2. **Single File Mode**: Deep analysis of one specific file
/// 3. **Multi-File Mode**: Process specific file lists for MCP tool chaining
///
/// # Filtering Behavior
///
/// When `max_cyclomatic` or `max_cognitive` thresholds are specified:
/// - Only files containing functions that EXCEED the thresholds are included
/// - This filtering happens BEFORE the `top_files` limit is applied
/// - A file with all functions below the threshold will be excluded from results
///
/// # MCP Tool Composition Examples
///
/// ```no_run
/// // Example 1: AI agent discovers complexity hotspots
/// use std::path::PathBuf;
/// use pmat::cli::{ComplexityOutputFormat, handlers::complexity_handlers::handle_analyze_complexity};
///
/// # async fn mcp_workflow_example() -> anyhow::Result<()> {
/// // Step 1: Find top 5 most complex files
/// handle_analyze_complexity(
///     PathBuf::from("."),
///     None,                           // file
///     vec![],                         // files (empty = project mode)
///     Some("rust".to_string()),       // toolchain
///     ComplexityOutputFormat::Json,   // format for parsing
///     None,                           // output (stdout)
///     Some(20),                       // max_cyclomatic
///     Some(15),                       // max_cognitive
///     vec![],                         // include patterns
///     false,                          // watch
///     5,                              // top_files = 5 hotspots
///     false,                          // fail_on_violation
///     60,                             // timeout (seconds)
/// ).await?;
///
/// // AI agent would parse JSON output to extract file paths:
/// // let hotspot_files = parse_json_extract_paths(json_output);
///
/// // Step 2: Deep analyze just those hotspot files
/// let hotspot_files = vec![
///     PathBuf::from("src/complex_module.rs"),
///     PathBuf::from("src/legacy_code.rs"),
/// ];
///
/// handle_analyze_complexity(
///     PathBuf::from("."),
///     None,                           // file
///     hotspot_files,                  // files (MCP composition)
///     Some("rust".to_string()),       // toolchain
///     ComplexityOutputFormat::Json,   // format
///     None,                           // output
///     Some(10),                       // stricter threshold
///     Some(8),                        // stricter threshold
///     vec![],                         // include patterns
///     false,                          // watch
///     0,                              // top_files (show all)
///     false,                          // fail_on_violation
///     60,                             // timeout (seconds)
/// ).await?;
/// # Ok(())
/// # }
/// ```
///
/// ```no_run
/// // Example 2: AI agent builds refactoring pipeline
/// use std::path::PathBuf;
/// use pmat::cli::{ComplexityOutputFormat, handlers::complexity_handlers::handle_analyze_complexity};
///
/// # async fn mcp_refactor_pipeline() -> anyhow::Result<()> {
/// // Step 1: Identify files needing refactoring
/// let candidate_files = vec![
///     PathBuf::from("src/user_service.rs"),
///     PathBuf::from("src/payment_processor.rs"),
///     PathBuf::from("src/notification_engine.rs"),
/// ];
///
/// // Step 2: Analyze complexity metrics for prioritization
/// handle_analyze_complexity(
///     PathBuf::from("."),
///     None,                           // file
///     candidate_files,                // files (targeted analysis)
///     Some("rust".to_string()),       // toolchain
///     ComplexityOutputFormat::Json,   // format for decision making
///     None,                           // output
///     Some(15),                       // max_cyclomatic
///     Some(12),                       // max_cognitive
///     vec![],                         // include patterns
///     false,                          // watch
///     0,                              // top_files (analyze all provided)
///     false,                          // fail_on_violation
///     60,                             // timeout (seconds)
/// ).await?;
///
/// // AI agent would then:
/// // 1. Parse complexity metrics
/// // 2. Prioritize by technical debt impact
/// // 3. Generate refactoring recommendations
/// // 4. Chain to other pmat tools (dead-code, duplicates, etc.)
/// # Ok(())
/// # }
/// ```
///
/// # Threshold Filtering Examples
///
/// ```no_run
/// // Example: Filtering behavior with --max-cyclomatic
/// use std::path::PathBuf;
/// use pmat::cli::{ComplexityOutputFormat, handlers::complexity_handlers::handle_analyze_complexity};
///
/// # async fn threshold_filtering_example() -> anyhow::Result<()> {
/// // Scenario: Find only files with functions exceeding cyclomatic complexity of 20
/// handle_analyze_complexity(
///     PathBuf::from("."),
///     None,                           // file
///     vec![],                         // files
///     Some("rust".to_string()),       // toolchain
///     ComplexityOutputFormat::Json,   // format
///     None,                           // output
///     Some(20),                       // max_cyclomatic - only show files with functions > 20
///     None,                           // max_cognitive
///     vec!["src/**/*.rs".to_string()],// include patterns
///     false,                          // watch
///     10,                             // top_files
///     false,                          // fail_on_violation
///     60,                             // timeout (seconds)
/// ).await?;
///
/// // Expected behavior:
/// // - File with functions [5, 10, 15] complexity -> EXCLUDED (all below 20)
/// // - File with functions [5, 25, 10] complexity -> INCLUDED (one function > 20)
/// // - File with functions [21, 30, 40] complexity -> INCLUDED (all above 20)
/// # Ok(())
/// # }
/// ```
///
/// ```no_run
/// // Example: Combined threshold filtering
/// use std::path::PathBuf;
/// use pmat::cli::{ComplexityOutputFormat, handlers::complexity_handlers::handle_analyze_complexity};
///
/// # async fn combined_threshold_example() -> anyhow::Result<()> {
/// // Scenario: Find files with either high cyclomatic OR high cognitive complexity
/// handle_analyze_complexity(
///     PathBuf::from("."),
///     None,                           // file
///     vec![],                         // files
///     Some("rust".to_string()),       // toolchain
///     ComplexityOutputFormat::Json,   // format
///     None,                           // output
///     Some(15),                       // max_cyclomatic
///     Some(12),                       // max_cognitive
///     vec!["src/**/*.rs".to_string()],// include patterns
///     false,                          // watch
///     5,                              // top_files - applied AFTER filtering
///     false,                          // fail_on_violation
///     60,                             // timeout (seconds)
/// ).await?;
///
/// // Expected behavior:
/// // - Files are first filtered to only include those with functions exceeding either threshold
/// // - Then the top 5 most complex files from the filtered set are returned
/// // - A file needs at least ONE function with cyclomatic > 15 OR cognitive > 12 to be included
/// # Ok(())
/// # }
/// ```
///
/// # Parameters
///
/// * `project_path` - Root directory of the project
/// * `file` - Single file for focused analysis (conflicts with `files`)
/// * `files` - **MCP Composition**: List of specific files to analyze
/// * `toolchain` - Language detection override
/// * `format` - Output format (JSON recommended for MCP workflows)
/// * `output` - File output path (None = stdout for MCP parsing)
/// * `max_cyclomatic` - Complexity threshold for violations
/// * `max_cognitive` - Cognitive load threshold for violations
/// * `include` - Glob patterns for project mode (conflicts with `files`)
/// * `watch` - Continuous analysis mode
/// * `top_files` - Limit output to N most complex files
///
/// # Exit Status
///
/// The command returns different exit codes based on results (addressing issue #28):
/// - `0`: Success - no violations found, all violations below threshold, or --fail-on-violation not specified
/// - `1`: Failure - violations found that exceed thresholds AND --fail-on-violation flag is used
///
/// ```bash
/// # Exit with code 0 even if violations found (default behavior)
/// pmat analyze complexity --max-cyclomatic 10
///
/// # Exit with code 1 if violations exceed threshold
/// pmat analyze complexity --max-cyclomatic 10 --fail-on-violation
/// ```
///
/// # Returns
///
/// JSON-structured complexity analysis suitable for MCP tool chaining
#[allow(clippy::too_many_arguments)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn handle_analyze_complexity(
    project_path: PathBuf,
    file: Option<PathBuf>,
    files: Vec<PathBuf>,
    toolchain: Option<String>,
    format: ComplexityOutputFormat,
    output: Option<PathBuf>,
    max_cyclomatic: Option<u16>,
    max_cognitive: Option<u16>,
    include: Vec<String>,
    watch: bool,
    top_files: usize,
    fail_on_violation: bool,
    timeout: u64,
) -> Result<()> {
    use crate::services::complexity::aggregate_results_with_thresholds;

    if watch {
        #[cfg(feature = "watch")]
        {
            return watch::handle_watch_mode(
                &project_path,
                toolchain.as_deref(),
                max_cyclomatic,
                max_cognitive,
                include,
                timeout,
                top_files,
                format,
                output.as_deref(),
            );
        }
        #[cfg(not(feature = "watch"))]
        {
            anyhow::bail!("Watch mode requires the 'watch' feature. Rebuild with: cargo build --features watch");
        }
    }

    // Validate path exists
    if !project_path.exists() {
        anyhow::bail!("Path not found: {}", project_path.display());
    }

    // Create configuration and analyze files
    let config = ComplexityConfig::from_args(
        project_path,
        toolchain,
        max_cyclomatic,
        max_cognitive,
        include,
        timeout,
        top_files,
    );

    let mut file_metrics = analysis::analyze_files_by_mode(file, files, &config).await?;

    // Track original count before filtering for better UX
    let original_file_count = file_metrics.len();

    // Apply filtering and aggregation
    let _filtered_count =
        analysis::apply_complexity_filters(&mut file_metrics, max_cyclomatic, max_cognitive);
    analysis::apply_top_files_limit(&mut file_metrics, config.top_files);

    // Check if all files were filtered out and provide helpful message
    if original_file_count > 0 && file_metrics.is_empty() {
        eprintln!(
            "\n⚠️  Warning: All {} file(s) were filtered out",
            original_file_count
        );
        eprintln!("   No functions found exceeding the complexity thresholds:");
        if let Some(cyc) = max_cyclomatic {
            eprintln!("   - Cyclomatic complexity > {}", cyc);
        }
        if let Some(cog) = max_cognitive {
            eprintln!("   - Cognitive complexity > {}", cog);
        }
        eprintln!("\n💡 Suggestions:");
        eprintln!("   1. Lower the thresholds using --max-cyclomatic or --max-cognitive");
        eprintln!("   2. Remove thresholds to see all files");
        eprintln!("   3. Use --verbose to see detailed analysis of all files\n");
    }

    // Create summary with original file count for accurate reporting
    let mut summary =
        aggregate_results_with_thresholds(file_metrics.clone(), max_cyclomatic, max_cognitive);

    // Fix: Update summary to reflect actual files analyzed before filtering
    summary.summary.total_files = original_file_count;

    // Format and write output
    output::format_and_write_output(&summary, &file_metrics, format, output, top_files).await?;

    // Check violations if required
    analysis::check_complexity_violations(
        &file_metrics,
        fail_on_violation,
        max_cyclomatic,
        max_cognitive,
    );

    Ok(())
}

/// Handle DAG (Dependency Analysis Graph) generation command
#[allow(clippy::too_many_arguments)]
/// Generate dependency analysis graphs using Mermaid
///
/// # Examples
///
/// ```no_run
/// use pmat::cli::handlers::complexity_handlers::handle_analyze_dag;
/// use pmat::cli::DagType;
/// use std::path::PathBuf;
/// use tempfile::tempdir;
///
/// # tokio_test::block_on(async {
/// let dir = tempdir().expect("internal error");
///
/// // Generate a full dependency graph
/// let result = handle_analyze_dag(
///     DagType::FullDependency,
///     dir.path().to_path_buf(),
///     None, // output to stdout
///     None, // no max depth
///     Some(10), // limit to 10 nodes
///     false, // include external deps
///     false, // don't show complexity
///     false, // no duplicate analysis
///     false, // no dead code analysis
///     false, // not enhanced
/// ).await;
///
/// assert!(result.is_ok());
/// # });
/// ```
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn handle_analyze_dag(
    _dag_type: DagType,
    project_path: PathBuf,
    output: Option<PathBuf>,
    max_depth: Option<usize>,
    target_nodes: Option<usize>,
    filter_external: bool,
    show_complexity: bool,
    _include_duplicates: bool,
    _include_dead_code: bool,
    enhanced: bool,
) -> Result<()> {
    use crate::services::{
        context::analyze_project,
        mermaid_generator::{MermaidGenerator, MermaidOptions},
    };

    eprintln!("🔄 Generating dependency analysis graph...");

    // Analyze project to get context
    let toolchain =
        crate::cli::detect_primary_language(&project_path).unwrap_or_else(|| "rust".to_string());
    let project_context = analyze_project(&project_path, &toolchain).await?;

    eprintln!("📁 Analyzed {} files", project_context.files.len());

    // Build DAG based on type
    use crate::services::dag_builder::DagBuilder;

    // DagBuilder builds a full dependency graph by default
    let graph = DagBuilder::build_from_project(&project_context);

    eprintln!(
        "📊 Generated graph with {} nodes and {} edges",
        graph.nodes.len(),
        graph.edges.len()
    );

    let enriched_graph = graph;

    // Generate Mermaid diagram
    let options = MermaidOptions {
        max_depth,
        filter_external,
        group_by_module: enhanced,
        show_complexity,
    };

    let generator = MermaidGenerator::new(options);
    let mermaid_content = if enhanced || target_nodes.is_some() {
        // Use advanced graph configuration
        use crate::services::fixed_graph_builder::{GraphConfig, GroupingStrategy};
        let config = GraphConfig {
            max_nodes: target_nodes.unwrap_or(100),
            max_edges: target_nodes.map_or(400, |n| n * 4),
            grouping: GroupingStrategy::Module,
        };
        generator.generate_with_config(&enriched_graph, &config)
    } else {
        generator.generate(&enriched_graph)
    };

    // Write output
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &mermaid_content).await?;
        eprintln!("✅ DAG written to: {}", output_path.display());

        // Additional hint for viewing
        if output_path.extension().is_some_and(|ext| ext == "mmd") {
            eprintln!("\n💡 To view the graph:");
            eprintln!("   - Copy content to https://mermaid.live");
            eprintln!("   - Or use VS Code with Mermaid extension");
        }
    } else {
        println!("{mermaid_content}");
    }

    Ok(())
}
