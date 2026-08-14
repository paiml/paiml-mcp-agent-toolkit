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

    // GH-682: this used to be a bare `project_path.exists()` check, which a
    // `chmod 000` directory passes — the walk then found zero files and the
    // command exited 0 with "⚠️  Warning: No files were found or analyzed",
    // reporting a clean pass over content it was denied access to. The shared
    // guard also refuses a directory whose entries cannot be listed, which is
    // what `analyze satd` already did (exit 1, "Permission denied").
    crate::cli::ensure_analysis_path_exists(&project_path)?;

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

    // The summary is aggregated over every file that survived the thresholds —
    // NOT over the --top-files slice. It used to be computed after truncation
    // while `total_files` was overwritten with the whole-project count, so one
    // unchanged 1070-file tree reported total_files 1070 next to
    // total_functions 159 (true value 10148, 64x low), technical_debt_hours
    // 388.75 (true 1644.25) and max_cyclomatic 29 (true project max 31) — the
    // default --top-files 10 was a cap wearing the word "total".
    let analyzed_file_count = file_metrics.len();
    let aggregated_metrics = file_metrics.clone();
    analysis::apply_top_files_limit(&mut file_metrics, config.top_files);
    let files_truncated = file_metrics.len() < analyzed_file_count;

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

    // Aggregate over every analyzed file, then list only the top-N slice.
    // `total_files` now means "files this summary aggregates", which is what
    // the other summary fields are computed from.
    let summary = analysis::build_report_over_analyzed_files(
        aggregated_metrics,
        file_metrics.clone(),
        max_cyclomatic,
        max_cognitive,
    );

    // Format and write output
    let listing = output::ListingDisclosure {
        top_files,
        files_listed: file_metrics.len(),
        files_analyzed: analyzed_file_count,
        files_discovered: original_file_count,
        truncated: files_truncated,
    };
    output::format_and_write_output(&summary, &file_metrics, format, output, listing).await?;

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
    dag_type: DagType,
    project_path: PathBuf,
    output: Option<PathBuf>,
    max_depth: Option<usize>,
    target_nodes: Option<usize>,
    filter_external: bool,
    show_complexity: bool,
    include_duplicates: bool,
    include_dead_code: bool,
    enhanced: bool,
) -> Result<()> {
    use crate::services::{
        context::analyze_project,
        mermaid_generator::{MermaidGenerator, MermaidOptions},
    };

    // missing_path_fails: never render a graph for a path that does not exist.
    // GH-666: a nonexistent path walked to zero files and this printed
    // "📁 Analyzed 0 files / 📊 Generated graph with 0 nodes and 0 edges" with
    // exit 0 — an empty-but-successful graph for a tree that was never there.
    crate::cli::ensure_analysis_path_exists(&project_path)?;

    crate::status_eprintln!("🔄 Generating dependency analysis graph...");

    // Analyze project to get context
    let toolchain =
        crate::cli::detect_primary_language(&project_path).unwrap_or_else(|| "rust".to_string());
    let project_context = analyze_project(&project_path, &toolchain).await?;

    crate::status_eprintln!("📁 Analyzed {} files", project_context.files.len());

    // Build DAG based on type
    use crate::services::dag_builder::DagBuilder;

    // DagBuilder covers module/import/inheritance relationships; call edges need the
    // function bodies, which only the sources carry (#653: no Calls edge ever existed).
    let mut graph = DagBuilder::build_from_project(&project_context);
    crate::services::dag_call_edges::add_call_edges(&mut graph, &project_path);

    // #653: `--dag-type` used to be ignored entirely, so call-graph, import-graph,
    // inheritance and full-dependency produced byte-identical output.
    let enriched_graph = filter_graph_by_dag_type(graph, &dag_type);

    // `--filter-external` had the same shape as the already-fixed `--max-depth`
    // bug: it was stored in `MermaidOptions` and the renderer reads only
    // `show_complexity`, so the flag changed nothing. What it filters is the
    // graph, so it has to be applied to the graph.
    let enriched_graph = apply_external_filter(enriched_graph, filter_external);

    // `--max-depth` was only stored in `MermaidOptions`, which the renderer never
    // reads, so every depth produced the same diagram. Traversal depth has to be
    // applied to the graph itself, before it is rendered.
    let enriched_graph = limit_graph_depth(enriched_graph, max_depth);

    // `--include-duplicates` and `--include-dead-code` were bound to
    // `_include_duplicates` / `_include_dead_code` and never referenced: both
    // flags parsed, were forwarded faithfully by the router, and produced a
    // byte-identical diagram. Each now runs the project's ONE existing analyzer
    // for that question and marks the nodes it names.
    let enriched_graph = annotate_duplicates(enriched_graph, &project_path, include_duplicates)?;
    let enriched_graph =
        annotate_dead_code(enriched_graph, &project_path, include_dead_code).await?;

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

    // #653: the announced counts used to be the pre-render graph's, which disagreed
    // with the diagram (75 announced / 41 drawn). Count what was actually emitted.
    report_graph_size(&dag_type, &enriched_graph, &mermaid_content);

    // Write output
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &mermaid_content).await?;
        crate::status_eprintln!("✅ DAG written to: {}", output_path.display());

        // Additional hint for viewing
        if output_path.extension().is_some_and(|ext| ext == "mmd") {
            crate::status_eprintln!("\n💡 To view the graph:");
            crate::status_eprintln!("   - Copy content to https://mermaid.live");
            crate::status_eprintln!("   - Or use VS Code with Mermaid extension");
        }
    } else {
        println!("{mermaid_content}");
    }

    Ok(())
}

/// Reduce the full dependency graph to the sub-graph the requested `--dag-type` names.
///
/// Each type keeps only its own edges and the nodes those edges touch, so the four
/// types genuinely differ (#653: they were byte-identical).
fn filter_graph_by_dag_type(
    graph: crate::models::dag::DependencyGraph,
    dag_type: &DagType,
) -> crate::models::dag::DependencyGraph {
    use crate::models::dag::EdgeType;

    match dag_type {
        DagType::CallGraph => graph.filter_by_edge_types(&[EdgeType::Calls]),
        DagType::ImportGraph => graph.filter_by_edge_types(&[EdgeType::Imports]),
        DagType::Inheritance => {
            graph.filter_by_edge_types(&[EdgeType::Inherits, EdgeType::Implements])
        }
        DagType::FullDependency => graph,
    }
}

/// Directory names that hold code this project did not write. A node whose file
/// sits under one of them is an external dependency, whatever the language.
const VENDORED_DIR_NAMES: &[&str] = &[
    "vendor",
    "vendored",
    "third_party",
    "thirdparty",
    "node_modules",
    "bower_components",
    "site-packages",
    "dist-packages",
    ".venv",
    "venv",
    ".cargo",
    "target",
    "Pods",
    "Godeps",
    "external",
];

/// Is this file external to the project being analyzed?
///
/// True for anything under a vendored-dependency directory and for any path
/// that climbs out of the analyzed tree (`..`). Deliberately independent of the
/// walker's own ignore rules: the walk DOES descend into `vendor/`, which is how
/// `--filter-external` has something to filter.
pub(crate) fn is_external_file(file_path: &str) -> bool {
    use std::path::{Component, Path};

    Path::new(file_path)
        .components()
        .any(|component| match component {
            Component::Normal(name) => VENDORED_DIR_NAMES
                .iter()
                .any(|dir| name.eq_ignore_ascii_case(std::ffi::OsStr::new(dir))),
            Component::ParentDir => true,
            _ => false,
        })
}

/// Keep the nodes `keep` accepts, dropping every edge that touched a removed one.
fn retain_graph_nodes(
    graph: crate::models::dag::DependencyGraph,
    keep: impl Fn(&str) -> bool,
) -> crate::models::dag::DependencyGraph {
    let nodes: rustc_hash::FxHashMap<String, crate::models::dag::NodeInfo> =
        graph.nodes.into_iter().filter(|(id, _)| keep(id)).collect();

    let edges = graph
        .edges
        .into_iter()
        .filter(|e| nodes.contains_key(&e.from) && nodes.contains_key(&e.to))
        .collect();

    crate::models::dag::DependencyGraph { nodes, edges }
}

/// Apply `--filter-external`: drop the nodes that live in vendored dependencies.
///
/// Always reports what it removed when the flag is on — "dropped 0" is a
/// measurement, silence is not.
pub(crate) fn apply_external_filter(
    graph: crate::models::dag::DependencyGraph,
    filter_external: bool,
) -> crate::models::dag::DependencyGraph {
    if !filter_external {
        return graph;
    }

    let total = graph.nodes.len();
    let external: rustc_hash::FxHashSet<String> = graph
        .nodes
        .iter()
        .filter(|(_, node)| is_external_file(&node.file_path))
        .map(|(id, _)| id.clone())
        .collect();

    crate::status_eprintln!(
        "🔗 --filter-external: dropped {} external node(s) of {total}",
        external.len()
    );

    retain_graph_nodes(graph, |id| !external.contains(id))
}

/// Append a marker to the labels of `ids`, so the annotation reaches the
/// rendered diagram (the Mermaid label is the node label, via `SemanticNamer`).
fn mark_graph_nodes(
    mut graph: crate::models::dag::DependencyGraph,
    ids: &rustc_hash::FxHashSet<String>,
    marker: &str,
    metadata_key: &str,
) -> crate::models::dag::DependencyGraph {
    for (id, node) in &mut graph.nodes {
        if ids.contains(id) {
            node.label = format!("{} {marker}", node.label);
            node.metadata
                .insert(metadata_key.to_string(), "true".to_string());
        }
    }
    graph
}

/// Do two recorded paths name the same file? Node paths are project-relative,
/// analyzer paths may be absolute, so compare on the longest common suffix.
fn same_file(a: &str, b: &str) -> bool {
    let a = a.replace('\\', "/");
    let b = b.replace('\\', "/");
    if a == b {
        return true;
    }
    let (long, short) = if a.len() >= b.len() {
        (&a, &b)
    } else {
        (&b, &a)
    };
    !short.is_empty() && long.ends_with(short.as_str()) && {
        // Only a whole path component may match: "b.rs" must not match "sub.rs".
        let boundary = long.len() - short.len();
        boundary == 0 || long.as_bytes()[boundary - 1] == b'/'
    }
}

/// Apply `--include-duplicates`: mark the nodes that sit inside a detected clone.
///
/// Runs the project's existing clone detector (`DuplicateDetectionEngine`, the
/// same engine `DuplicationDefectAnalyzer` uses) with its default config, so
/// this adds no second opinion about what a duplicate is.
fn annotate_duplicates(
    graph: crate::models::dag::DependencyGraph,
    project_path: &std::path::Path,
    include_duplicates: bool,
) -> Result<crate::models::dag::DependencyGraph> {
    if !include_duplicates {
        return Ok(graph);
    }

    use crate::services::duplicate_detector::{
        DuplicateDetectionConfig, DuplicateDetectionEngine, Language,
    };

    let files =
        crate::services::file_discovery::ProjectFileDiscovery::new(project_path.to_path_buf())
            .discover_files()
            .unwrap_or_default();

    let mut contents = Vec::new();
    for file in files {
        let language = match file.extension().and_then(|e| e.to_str()) {
            Some("rs") => Language::Rust,
            Some("ts") => Language::TypeScript,
            Some("js") => Language::JavaScript,
            Some("py") => Language::Python,
            Some("c") => Language::C,
            Some("cpp" | "cc" | "cxx" | "cu" | "cuh") => Language::Cpp,
            Some("kt") => Language::Kotlin,
            _ => continue,
        };
        if let Ok(source) = std::fs::read_to_string(&file) {
            contents.push((file, source, language));
        }
    }

    let engine = DuplicateDetectionEngine::new(DuplicateDetectionConfig::default());
    let report = engine.detect_duplicates(&contents)?;

    let mut fragments: Vec<(String, usize, usize)> = Vec::new();
    for group in &report.groups {
        for fragment in &group.fragments {
            fragments.push((
                fragment.file.to_string_lossy().to_string(),
                fragment.start_line,
                fragment.end_line,
            ));
        }
    }

    let marked = nodes_in_line_ranges(&graph, &fragments);

    crate::status_eprintln!(
        "⧉ --include-duplicates: {} clone group(s), {} node(s) marked",
        report.groups.len(),
        marked.len()
    );

    Ok(mark_graph_nodes(graph, &marked, "(dup)", "duplicate"))
}

/// Node ids whose file and line fall inside one of `ranges`.
///
/// A module node (line 0) is matched by file alone: the module contains the
/// duplicated fragment even though it has no line of its own.
fn nodes_in_line_ranges(
    graph: &crate::models::dag::DependencyGraph,
    ranges: &[(String, usize, usize)],
) -> rustc_hash::FxHashSet<String> {
    graph
        .nodes
        .iter()
        .filter(|(_, node)| {
            ranges.iter().any(|(file, start, end)| {
                same_file(&node.file_path, file)
                    && (node.line_number == 0
                        || (node.line_number >= *start && node.line_number <= *end))
            })
        })
        .map(|(id, _)| id.clone())
        .collect()
}

/// Apply `--include-dead-code`: mark the nodes the dead-code analyzer names.
///
/// Uses `services::cargo_dead_code_analyzer`, the same analyzer `pmat analyze
/// dead-code` runs for Rust projects, so the diagram cannot contradict the
/// report. When that analysis cannot run this fails loudly rather than drawing
/// an unannotated graph that would read as "nothing is dead".
async fn annotate_dead_code(
    graph: crate::models::dag::DependencyGraph,
    project_path: &std::path::Path,
    include_dead_code: bool,
) -> Result<crate::models::dag::DependencyGraph> {
    if !include_dead_code {
        return Ok(graph);
    }

    let report = crate::services::cargo_dead_code_analyzer::analyze_dead_code(project_path)
        .await
        .map_err(|e| {
            anyhow::anyhow!(
                "--include-dead-code could not run the dead-code analyzer on {}: {e}. \
                 Re-run without --include-dead-code for an unannotated graph.",
                project_path.display()
            )
        })?;

    let mut dead: Vec<(String, String, usize)> = Vec::new();
    for file in &report.files_with_dead_code {
        for item in &file.dead_items {
            dead.push((
                file.file_path.to_string_lossy().to_string(),
                item.name.clone(),
                item.line,
            ));
        }
    }

    let marked: rustc_hash::FxHashSet<String> = graph
        .nodes
        .iter()
        .filter(|(_, node)| {
            dead.iter().any(|(file, name, line)| {
                same_file(&node.file_path, file)
                    && (node.label == *name || node.line_number == *line)
            })
        })
        .map(|(id, _)| id.clone())
        .collect();

    crate::status_eprintln!(
        "☠️ --include-dead-code: {} dead item(s) reported, {} node(s) marked",
        report.total_dead_items,
        marked.len()
    );

    Ok(mark_graph_nodes(graph, &marked, "(dead)", "dead_code"))
}

/// Keep only the nodes reachable within `max_depth` traversal hops from a root.
///
/// Roots are the nodes nothing depends on (in-degree 0); a graph that is all
/// cycles has no such node, so every node is treated as a root and the depth
/// bound then measures distance from the nearest cycle entry. `--max-depth 0`
/// is the roots alone.
fn limit_graph_depth(
    graph: crate::models::dag::DependencyGraph,
    max_depth: Option<usize>,
) -> crate::models::dag::DependencyGraph {
    use rustc_hash::{FxHashMap, FxHashSet};
    use std::collections::VecDeque;

    let Some(max_depth) = max_depth else {
        return graph;
    };

    let mut has_incoming: FxHashSet<&String> = FxHashSet::default();
    for edge in &graph.edges {
        if edge.from != edge.to {
            has_incoming.insert(&edge.to);
        }
    }

    let mut roots: Vec<&String> = graph
        .nodes
        .keys()
        .filter(|id| !has_incoming.contains(*id))
        .collect();
    if roots.is_empty() {
        roots = graph.nodes.keys().collect();
    }

    let mut successors: FxHashMap<&String, Vec<&String>> = FxHashMap::default();
    for edge in &graph.edges {
        successors.entry(&edge.from).or_default().push(&edge.to);
    }

    let mut depth: FxHashMap<&String, usize> = FxHashMap::default();
    let mut queue: VecDeque<&String> = VecDeque::new();
    for root in roots {
        depth.insert(root, 0);
        queue.push_back(root);
    }

    while let Some(node) = queue.pop_front() {
        let next_depth = depth[node] + 1;
        if next_depth > max_depth {
            continue;
        }
        if let Some(children) = successors.get(node) {
            for child in children {
                if !depth.contains_key(*child) {
                    depth.insert(child, next_depth);
                    queue.push_back(child);
                }
            }
        }
    }

    let kept: FxHashSet<String> = depth.keys().map(|id| (*id).clone()).collect();

    let edges = graph
        .edges
        .iter()
        .filter(|e| kept.contains(&e.from) && kept.contains(&e.to))
        .cloned()
        .collect();
    let nodes = graph
        .nodes
        .iter()
        .filter(|(id, _)| kept.contains(*id))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    crate::models::dag::DependencyGraph { nodes, edges }
}

/// Report the size of the diagram that was actually emitted, plus the size of the
/// graph it was rendered from when the renderer's node budget dropped some of it.
fn report_graph_size(
    dag_type: &DagType,
    graph: &crate::models::dag::DependencyGraph,
    mermaid_content: &str,
) {
    let (rendered_nodes, rendered_edges) = count_rendered_elements(mermaid_content);

    crate::status_eprintln!(
        "📊 {dag_type}: rendered {rendered_nodes} nodes and {rendered_edges} edges"
    );

    if rendered_nodes < graph.nodes.len() || rendered_edges < graph.edges.len() {
        crate::status_eprintln!(
            "   (analyzed {} nodes and {} edges; the diagram is capped for readability)",
            graph.nodes.len(),
            graph.edges.len()
        );
    }
}

/// Count the node and edge lines present in a generated Mermaid diagram.
fn count_rendered_elements(mermaid_content: &str) -> (usize, usize) {
    let mut nodes = 0;
    let mut edges = 0;

    for line in mermaid_content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty()
            || trimmed.starts_with("graph ")
            || trimmed.starts_with("style ")
            || trimmed.starts_with("classDef ")
            || trimmed.starts_with("subgraph")
            || trimmed == "end"
        {
            continue;
        }

        if is_mermaid_edge_line(trimmed) {
            edges += 1;
        } else {
            nodes += 1;
        }
    }

    (nodes, edges)
}

fn is_mermaid_edge_line(line: &str) -> bool {
    line.contains("-->") || line.contains("-.->") || line.contains(" --- ")
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod dag_content_flag_tests {
    //! `--filter-external`, `--include-duplicates` and `--include-dead-code`
    //! all change what the GRAPH contains, and all three used to change
    //! nothing: the first was stored in `MermaidOptions` (which the renderer
    //! never reads) and the other two were bound to `_include_duplicates` /
    //! `_include_dead_code` and never referenced.
    use super::*;
    use crate::models::dag::{DependencyGraph, Edge, EdgeType, NodeInfo, NodeType};
    use rustc_hash::{FxHashMap, FxHashSet};

    fn node(id: &str, file: &str, line: usize) -> NodeInfo {
        NodeInfo {
            id: id.to_string(),
            label: id.to_string(),
            node_type: NodeType::Function,
            file_path: file.to_string(),
            line_number: line,
            complexity: 1,
            metadata: FxHashMap::default(),
        }
    }

    fn two_file_graph() -> DependencyGraph {
        let mut graph = DependencyGraph::new();
        graph.add_node(node("own", "src/a.rs", 3));
        graph.add_node(node("ext", "vendor/dep/src/lib.rs", 1));
        graph.add_edge(Edge {
            from: "own".to_string(),
            to: "ext".to_string(),
            edge_type: EdgeType::Calls,
            weight: 1,
        });
        graph
    }

    #[test]
    fn filter_external_drops_vendored_nodes_and_their_edges() {
        let kept = apply_external_filter(two_file_graph(), true);
        assert!(kept.nodes.contains_key("own"));
        assert!(
            !kept.nodes.contains_key("ext"),
            "a node under vendor/ is an external dependency"
        );
        assert!(
            kept.edges.is_empty(),
            "an edge to a dropped node cannot survive it"
        );
    }

    #[test]
    fn without_the_flag_the_graph_is_untouched() {
        let kept = apply_external_filter(two_file_graph(), false);
        assert_eq!(kept.nodes.len(), 2);
        assert_eq!(kept.edges.len(), 1);
    }

    #[test]
    fn external_recognises_dependency_trees_not_arbitrary_names() {
        assert!(is_external_file("node_modules/left-pad/index.js"));
        assert!(is_external_file("third_party/zlib/zlib.c"));
        assert!(is_external_file("../outside/lib.rs"));
        assert!(!is_external_file("src/vendored_config.rs"));
        assert!(!is_external_file("src/services/target_analysis.rs"));
    }

    #[test]
    fn same_file_matches_whole_components_only() {
        assert!(same_file("src/b.rs", "/home/u/proj/src/b.rs"));
        assert!(same_file("src/b.rs", "src/b.rs"));
        assert!(
            !same_file("src/sub.rs", "src/b.rs"),
            "a suffix match must not straddle a component boundary"
        );
    }

    #[test]
    fn duplicate_ranges_mark_functions_by_line_and_modules_by_file() {
        let mut graph = DependencyGraph::new();
        graph.add_node(node("inside", "src/dup.rs", 12));
        graph.add_node(node("outside", "src/dup.rs", 400));
        let mut module = node("module", "src/dup.rs", 0);
        module.node_type = NodeType::Module;
        graph.add_node(module);

        let ranges = vec![("src/dup.rs".to_string(), 10usize, 40usize)];
        let marked = nodes_in_line_ranges(&graph, &ranges);

        assert!(marked.contains("inside"));
        assert!(marked.contains("module"), "the module holds the clone");
        assert!(!marked.contains("outside"));
    }

    #[test]
    fn marking_reaches_the_rendered_label() {
        let mut ids = FxHashSet::default();
        ids.insert("own".to_string());
        let marked = mark_graph_nodes(two_file_graph(), &ids, "(dead)", "dead_code");

        let node = &marked.nodes["own"];
        assert_eq!(node.label, "own (dead)");
        assert_eq!(
            node.metadata.get("dead_code").map(String::as_str),
            Some("true")
        );
        assert_eq!(
            marked.nodes["ext"].label, "ext",
            "an unmarked node keeps its label"
        );
    }

    /// End-to-end through the public handler, which is where the defect was
    /// visible: `analyze dag` and `analyze dag --filter-external` wrote
    /// byte-identical diagrams.
    #[tokio::test]
    async fn handler_honours_filter_external() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        std::fs::create_dir_all(root.join("src")).expect("src");
        std::fs::create_dir_all(root.join("vendor/dep/src")).expect("vendor");
        std::fs::write(root.join("src/lib.rs"), "pub fn own_fn() -> u64 { 1 }\n").expect("lib");
        std::fs::write(
            root.join("vendor/dep/src/lib.rs"),
            "pub fn vendored_fn() -> u64 { 2 }\n",
        )
        .expect("vendor lib");

        let unfiltered = root.join("all.mmd");
        let filtered = root.join("own.mmd");

        handle_analyze_dag(
            DagType::FullDependency,
            root.to_path_buf(),
            Some(unfiltered.clone()),
            None,
            None,
            false,
            false,
            false,
            false,
            false,
        )
        .await
        .expect("unfiltered dag");

        handle_analyze_dag(
            DagType::FullDependency,
            root.to_path_buf(),
            Some(filtered.clone()),
            None,
            None,
            true,
            false,
            false,
            false,
            false,
        )
        .await
        .expect("filtered dag");

        let all = std::fs::read_to_string(&unfiltered).expect("read all");
        let own = std::fs::read_to_string(&filtered).expect("read own");

        assert!(
            all.contains("vendored_fn"),
            "the baseline must contain the vendored node, else the test proves nothing: {all}"
        );
        assert!(
            !own.contains("vendored_fn"),
            "--filter-external must drop the vendored node: {own}"
        );
        assert!(own.contains("own_fn"), "project code must survive: {own}");
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod dag_depth_tests {
    use super::limit_graph_depth;
    use crate::models::dag::{DependencyGraph, Edge, EdgeType, NodeInfo, NodeType};
    use rustc_hash::FxHashMap;

    /// a -> b -> c -> d, one chain four deep.
    fn chain() -> DependencyGraph {
        let mut graph = DependencyGraph::new();
        for id in ["a", "b", "c", "d"] {
            graph.add_node(NodeInfo {
                id: id.to_string(),
                label: id.to_string(),
                node_type: NodeType::Function,
                file_path: "src/lib.rs".to_string(),
                line_number: 1,
                complexity: 1,
                metadata: FxHashMap::default(),
            });
        }
        for (from, to) in [("a", "b"), ("b", "c"), ("c", "d")] {
            graph.add_edge(Edge {
                from: from.to_string(),
                to: to.to_string(),
                edge_type: EdgeType::Calls,
                weight: 1,
            });
        }
        graph
    }

    /// `--max-depth` was stored in `MermaidOptions` and never read, so
    /// `analyze dag --max-depth 1` and no `--max-depth` at all wrote
    /// byte-identical diagrams at every value.
    #[test]
    fn max_depth_limits_traversal() {
        let full = limit_graph_depth(chain(), None);
        assert_eq!(full.nodes.len(), 4);

        let depth_1 = limit_graph_depth(chain(), Some(1));
        assert_eq!(depth_1.nodes.len(), 2, "root plus one hop");
        assert!(depth_1.nodes.contains_key("a") && depth_1.nodes.contains_key("b"));
        assert_eq!(depth_1.edges.len(), 1);

        let depth_0 = limit_graph_depth(chain(), Some(0));
        assert_eq!(depth_0.nodes.len(), 1, "roots only");
        assert!(depth_0.edges.is_empty());

        let depth_9 = limit_graph_depth(chain(), Some(9));
        assert_eq!(
            depth_9.nodes.len(),
            4,
            "a depth past the graph keeps it all"
        );
    }

    /// A graph that is one cycle has no in-degree-0 node; depth limiting must
    /// still return a graph rather than dropping every node.
    #[test]
    fn cyclic_graph_is_not_emptied() {
        let mut graph = chain();
        graph.add_edge(Edge {
            from: "d".to_string(),
            to: "a".to_string(),
            edge_type: EdgeType::Calls,
            weight: 1,
        });

        let limited = limit_graph_depth(graph, Some(1));
        assert!(!limited.nodes.is_empty());
    }
}
