pub mod args;

use crate::{
    models::{churn::ChurnOutputFormat, template::*},
    services::template_service::*,
    stateless_server::StatelessTemplateServer,
};
use clap::{Parser, Subcommand, ValueEnum};
use serde_json::Value;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::io::AsyncWriteExt;
use tracing::{debug, info, instrument};

#[derive(Parser)]
#[command(
    name = "paiml-mcp-agent-toolkit",
    about = "Professional project scaffolding toolkit",
    version,
    long_about = None
)]
#[cfg_attr(test, derive(Debug))]
pub(crate) struct Cli {
    /// Force specific mode (auto-detected by default)
    #[arg(long, value_enum, global = true)]
    pub(crate) mode: Option<Mode>,

    /// Enable verbose output (info level)
    #[arg(short, long, global = true)]
    pub(crate) verbose: bool,

    /// Enable debug output (debug level)
    #[arg(long, global = true)]
    pub(crate) debug: bool,

    /// Enable trace output (trace level)
    #[arg(long, global = true)]
    pub(crate) trace: bool,

    /// Custom trace filter (overrides other flags)
    /// Example: --trace-filter="paiml=debug,cache=trace"
    #[arg(long, global = true, env = "RUST_LOG")]
    pub(crate) trace_filter: Option<String>,

    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Clone, Debug, ValueEnum)]
pub(crate) enum Mode {
    Cli,
    Mcp,
}

#[derive(Clone, Debug)]
pub enum ExecutionMode {
    Cli,
    Mcp,
}

#[derive(Subcommand)]
#[cfg_attr(test, derive(Debug))]
pub enum Commands {
    /// Generate a single template
    #[command(visible_aliases = &["gen", "g"])]
    Generate {
        /// Template category
        category: String,

        /// Template path (e.g., rust/cli)
        template: String,

        /// Parameters as key=value pairs
        #[arg(short = 'p', long = "param", value_parser = args::parse_key_val)]
        params: Vec<(String, Value)>,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Create parent directories
        #[arg(long)]
        create_dirs: bool,
    },

    /// Scaffold complete project
    Scaffold {
        /// Target toolchain
        toolchain: String,

        /// Templates to generate
        #[arg(short, long, value_delimiter = ',')]
        templates: Vec<String>,

        /// Parameters
        #[arg(short = 'p', long = "param", value_parser = args::parse_key_val)]
        params: Vec<(String, Value)>,

        /// Parallelism level
        #[arg(long, default_value_t = num_cpus::get())]
        parallel: usize,
    },

    /// List available templates
    List {
        /// Filter by toolchain
        #[arg(long)]
        toolchain: Option<String>,

        /// Filter by category
        #[arg(long)]
        category: Option<String>,

        /// Output format
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,
    },

    /// Search templates
    Search {
        /// Search query
        query: String,

        /// Filter by toolchain
        #[arg(long)]
        toolchain: Option<String>,

        /// Max results
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },

    /// Validate template parameters
    Validate {
        /// Template URI
        uri: String,

        /// Parameters to validate
        #[arg(short = 'p', long = "param", value_parser = args::parse_key_val)]
        params: Vec<(String, Value)>,
    },

    /// Generate project context (AST analysis)
    Context {
        /// Target toolchain (rust, deno, python-uv)
        toolchain: String,

        /// Project path to analyze
        #[arg(short = 'p', long, default_value = ".")]
        project_path: PathBuf,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Output format
        #[arg(long, value_enum, default_value = "markdown")]
        format: ContextFormat,
    },

    /// Analyze code metrics and patterns
    #[command(subcommand)]
    Analyze(AnalyzeCommands),

    /// Run interactive demo of all capabilities
    Demo {
        /// Repository path (defaults to current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Remote repository URL to clone and analyze
        #[arg(long)]
        url: Option<String>,

        /// Output format
        #[arg(short, long, value_enum, default_value = "table")]
        format: OutputFormat,

        /// Skip opening browser (web mode only)
        #[arg(long)]
        no_browser: bool,

        /// Port for demo server (default: random)
        #[arg(long)]
        port: Option<u16>,

        /// Run CLI output mode instead of web-based interactive demo
        #[arg(long)]
        cli: bool,

        /// Target node count for graph complexity reduction
        #[arg(long, default_value_t = 15)]
        target_nodes: usize,

        /// Minimum betweenness centrality threshold for graph reduction
        #[arg(long, default_value_t = 0.1)]
        centrality_threshold: f64,

        /// Component size threshold for merging in graph reduction
        #[arg(long, default_value_t = 3)]
        merge_threshold: usize,
    },

    /// Start HTTP API server
    Serve {
        /// Port to bind the HTTP server to
        #[arg(long, default_value_t = 8080)]
        port: u16,

        /// Host address to bind to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,

        /// Enable CORS for cross-origin requests
        #[arg(long)]
        cors: bool,
    },
}

#[derive(Subcommand)]
#[cfg_attr(test, derive(Debug))]
pub enum AnalyzeCommands {
    /// Analyze code churn (change frequency)
    Churn {
        /// Project path to analyze
        #[arg(short = 'p', long, default_value = ".")]
        project_path: PathBuf,

        /// Number of days to analyze
        #[arg(short = 'd', long, default_value_t = 30)]
        days: u32,

        /// Output format
        #[arg(long, value_enum, default_value = "summary")]
        format: ChurnOutputFormat,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Analyze code complexity
    Complexity {
        /// Project path to analyze
        #[arg(short = 'p', long, default_value = ".")]
        project_path: PathBuf,

        /// Filter by toolchain (rust, deno, python-uv)
        #[arg(long)]
        toolchain: Option<String>,

        /// Output format
        #[arg(long, value_enum, default_value = "summary")]
        format: ComplexityOutputFormat,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Custom cyclomatic complexity threshold
        #[arg(long)]
        max_cyclomatic: Option<u16>,

        /// Custom cognitive complexity threshold
        #[arg(long)]
        max_cognitive: Option<u16>,

        /// Include file patterns (e.g., "**/*.rs")
        #[arg(long)]
        include: Vec<String>,

        /// Watch mode for continuous analysis
        #[arg(long)]
        watch: bool,

        /// Number of top complex files to show (0 = show all violations)
        #[arg(long, default_value_t = 0)]
        top_files: usize,
    },

    /// Generate dependency graphs using Mermaid
    Dag {
        /// Type of dependency graph to generate
        #[arg(long, value_enum, default_value = "call-graph")]
        dag_type: DagType,

        /// Project path to analyze
        #[arg(short = 'p', long, default_value = ".")]
        project_path: PathBuf,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Maximum depth for graph traversal
        #[arg(long)]
        max_depth: Option<usize>,

        /// Filter out external dependencies
        #[arg(long)]
        filter_external: bool,

        /// Show complexity metrics in the graph
        #[arg(long)]
        show_complexity: bool,

        /// Include duplicate detection analysis
        #[arg(long)]
        include_duplicates: bool,

        /// Include dead code analysis
        #[arg(long)]
        include_dead_code: bool,

        /// Use enhanced vectorized analysis engine
        #[arg(long)]
        enhanced: bool,
    },

    /// Analyze dead and unreachable code
    #[command(name = "dead-code")]
    DeadCode {
        /// Path to analyze (defaults to current directory)
        #[arg(long, short = 'p', default_value = ".")]
        path: PathBuf,

        /// Output format
        #[arg(long, short = 'f', value_enum, default_value = "summary")]
        format: DeadCodeOutputFormat,

        /// Show top N files with most dead code
        #[arg(long, short = 't')]
        top_files: Option<usize>,

        /// Include unreachable code blocks in analysis
        #[arg(long, short = 'u')]
        include_unreachable: bool,

        /// Minimum dead lines to report a file (default: 10)
        #[arg(long, default_value = "10")]
        min_dead_lines: usize,

        /// Include test files in analysis
        #[arg(long)]
        include_tests: bool,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Analyze Self-Admitted Technical Debt (SATD) in comments
    #[command(name = "satd")]
    Satd {
        /// Path to analyze (defaults to current directory)
        #[arg(long, short = 'p', default_value = ".")]
        path: PathBuf,

        /// Output format
        #[arg(long, short = 'f', value_enum, default_value = "summary")]
        format: SatdOutputFormat,

        /// Filter by severity level
        #[arg(long, value_enum)]
        severity: Option<SatdSeverity>,

        /// Show only critical debt items
        #[arg(long)]
        critical_only: bool,

        /// Include test files in analysis
        #[arg(long)]
        include_tests: bool,

        /// Track debt evolution over time (requires git history)
        #[arg(long)]
        evolution: bool,

        /// Number of days for evolution analysis
        #[arg(long, default_value_t = 30)]
        days: u32,

        /// Show debt metrics summary
        #[arg(long)]
        metrics: bool,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Generate comprehensive deep context analysis with defect detection
    #[command(name = "deep-context")]
    DeepContext {
        /// Project path to analyze (defaults to current directory)
        #[arg(long, short = 'p', default_value = ".")]
        project_path: PathBuf,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Output format
        #[arg(long, value_enum, default_value = "markdown")]
        format: DeepContextOutputFormat,

        /// Comma-separated list of analyses to include
        #[arg(long, value_delimiter = ',')]
        include: Vec<String>,

        /// Comma-separated list of analyses to exclude
        #[arg(long, value_delimiter = ',')]
        exclude: Vec<String>,

        /// Period for churn analysis (default: 30 days)
        #[arg(long, default_value_t = 30)]
        period_days: u32,

        /// DAG type for dependency analysis
        #[arg(long, value_enum, default_value = "call-graph")]
        dag_type: DeepContextDagType,

        /// Maximum directory traversal depth
        #[arg(long)]
        max_depth: Option<usize>,

        /// Include file patterns (can be specified multiple times)
        #[arg(long = "include-pattern")]
        include_patterns: Vec<String>,

        /// Exclude file patterns (can be specified multiple times)  
        #[arg(long = "exclude-pattern")]
        exclude_patterns: Vec<String>,

        /// Cache usage strategy
        #[arg(long, value_enum, default_value = "normal")]
        cache_strategy: DeepContextCacheStrategy,

        /// Parallelism level for analysis
        #[arg(long)]
        parallel: Option<usize>,

        /// Enable verbose logging
        #[arg(long)]
        verbose: bool,
    },
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum ContextFormat {
    Markdown,
    Json,
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum OutputFormat {
    Table,
    Json,
    Yaml,
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum ComplexityOutputFormat {
    /// Summary statistics only
    Summary,
    /// Full report with violations
    Full,
    /// JSON format for tools
    Json,
    /// SARIF format for IDE integration
    Sarif,
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum DeadCodeOutputFormat {
    Summary,
    Json,
    Sarif,
    Markdown,
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum SatdOutputFormat {
    Summary,
    Json,
    Sarif,
    Markdown,
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum SatdSeverity {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Clone, Debug, ValueEnum, PartialEq, Eq, Hash)]
pub enum DagType {
    /// Function call graph
    #[value(name = "call-graph")]
    CallGraph,

    /// Import/dependency graph
    #[value(name = "import-graph")]
    ImportGraph,

    /// Class inheritance hierarchy
    #[value(name = "inheritance")]
    Inheritance,

    /// Complete dependency graph
    #[value(name = "full-dependency")]
    FullDependency,
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum DeepContextOutputFormat {
    Markdown,
    Json,
    Sarif,
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum DeepContextDagType {
    #[value(name = "call-graph")]
    CallGraph,
    #[value(name = "import-graph")]
    ImportGraph,
    #[value(name = "inheritance")]
    Inheritance,
    #[value(name = "full-dependency")]
    FullDependency,
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum DeepContextCacheStrategy {
    Normal,
    #[value(name = "force-refresh")]
    ForceRefresh,
    Offline,
}

/// Early CLI args struct for tracing initialization
#[derive(Debug, Clone)]
pub struct EarlyCliArgs {
    pub verbose: bool,
    pub debug: bool,
    pub trace: bool,
    pub trace_filter: Option<String>,
}

/// Parse CLI early to extract tracing configuration
pub fn parse_early_for_tracing() -> EarlyCliArgs {
    let args: Vec<String> = std::env::args().collect();

    let verbose = args.iter().any(|arg| arg == "-v" || arg == "--verbose");
    let debug = args.iter().any(|arg| arg == "--debug");
    let trace = args.iter().any(|arg| arg == "--trace");

    let trace_filter = args
        .iter()
        .position(|arg| arg == "--trace-filter")
        .and_then(|pos| args.get(pos + 1))
        .cloned()
        .or_else(|| std::env::var("RUST_LOG").ok());

    EarlyCliArgs {
        verbose,
        debug,
        trace,
        trace_filter,
    }
}

#[instrument(level = "debug", skip(server))]
pub async fn run(server: Arc<StatelessTemplateServer>) -> anyhow::Result<()> {
    let cli = Cli::parse();
    debug!("CLI arguments parsed");

    // Handle forced mode
    if let Some(Mode::Mcp) = cli.mode {
        info!("Forced MCP mode detected");
        return crate::run_mcp_server(server).await;
    }

    execute_command(cli.command, server).await
}

#[instrument(level = "debug", skip(command, server))]
async fn execute_command(
    command: Commands,
    server: Arc<StatelessTemplateServer>,
) -> anyhow::Result<()> {
    match command {
        Commands::Generate {
            category,
            template,
            params,
            output,
            create_dirs,
        } => handle_generate(server, category, template, params, output, create_dirs).await,
        Commands::Scaffold {
            toolchain,
            templates,
            params,
            parallel,
        } => handle_scaffold(server, toolchain, templates, params, parallel).await,
        Commands::List {
            toolchain,
            category,
            format,
        } => handle_list(server, toolchain, category, format).await,
        Commands::Search {
            query,
            toolchain,
            limit,
        } => handle_search(server, query, toolchain, limit).await,
        Commands::Validate { uri, params } => handle_validate(server, uri, params).await,
        Commands::Context {
            toolchain,
            project_path,
            output,
            format,
        } => handle_context(toolchain, project_path, output, format).await,
        Commands::Analyze(analyze_cmd) => execute_analyze_command(analyze_cmd).await,
        Commands::Demo {
            path,
            url,
            format,
            no_browser,
            port,
            cli,
            target_nodes,
            centrality_threshold,
            merge_threshold,
        } => {
            execute_demo_command(
                path,
                url,
                format,
                no_browser,
                port,
                cli,
                target_nodes,
                centrality_threshold,
                merge_threshold,
                server,
            )
            .await
        }
        Commands::Serve { port, host, cors } => handle_serve(host, port, cors).await,
    }
}

async fn execute_analyze_command(analyze_cmd: AnalyzeCommands) -> anyhow::Result<()> {
    match analyze_cmd {
        AnalyzeCommands::Churn {
            project_path,
            days,
            format,
            output,
        } => handle_analyze_churn(project_path, days, format, output).await,
        AnalyzeCommands::Dag {
            dag_type,
            project_path,
            output,
            max_depth,
            filter_external,
            show_complexity,
            include_duplicates,
            include_dead_code,
            enhanced,
        } => {
            handle_analyze_dag(
                dag_type,
                project_path,
                output,
                max_depth,
                filter_external,
                show_complexity,
                include_duplicates,
                include_dead_code,
                enhanced,
            )
            .await
        }
        AnalyzeCommands::Complexity {
            project_path,
            toolchain,
            format,
            output,
            max_cyclomatic,
            max_cognitive,
            include,
            watch,
            top_files,
        } => {
            handle_analyze_complexity(
                project_path,
                toolchain,
                format,
                output,
                max_cyclomatic,
                max_cognitive,
                include,
                watch,
                top_files,
            )
            .await
        }
        AnalyzeCommands::DeadCode {
            path,
            format,
            top_files,
            include_unreachable,
            min_dead_lines,
            include_tests,
            output,
        } => {
            handle_analyze_dead_code(
                path,
                format,
                top_files,
                include_unreachable,
                min_dead_lines,
                include_tests,
                output,
            )
            .await
        }
        AnalyzeCommands::Satd {
            path,
            format,
            severity,
            critical_only,
            include_tests,
            evolution,
            days,
            metrics,
            output,
        } => {
            handle_analyze_satd(
                path,
                format,
                severity,
                critical_only,
                include_tests,
                evolution,
                days,
                metrics,
                output,
            )
            .await
        }
        AnalyzeCommands::DeepContext {
            project_path,
            output,
            format,
            include,
            exclude,
            period_days,
            dag_type,
            max_depth,
            include_patterns,
            exclude_patterns,
            cache_strategy,
            parallel,
            verbose,
        } => {
            handle_analyze_deep_context(
                project_path,
                output,
                format,
                include,
                exclude,
                period_days,
                dag_type,
                max_depth,
                include_patterns,
                exclude_patterns,
                cache_strategy,
                parallel,
                verbose,
            )
            .await
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn execute_demo_command(
    path: Option<PathBuf>,
    url: Option<String>,
    format: OutputFormat,
    no_browser: bool,
    port: Option<u16>,
    cli: bool,
    target_nodes: usize,
    centrality_threshold: f64,
    merge_threshold: usize,
    server: Arc<StatelessTemplateServer>,
) -> anyhow::Result<()> {
    let demo_args = crate::demo::DemoArgs {
        path,
        url,
        format,
        no_browser,
        port,
        web: !cli, // Invert the flag - web is default unless --cli is specified
        target_nodes,
        centrality_threshold,
        merge_threshold,
    };
    crate::demo::run_demo(demo_args, server).await
}

// Command handlers - extracted from the main run function for better organization

#[instrument(level = "debug", skip(server))]
async fn handle_generate(
    server: Arc<StatelessTemplateServer>,
    category: String,
    template: String,
    params: Vec<(String, Value)>,
    output: Option<PathBuf>,
    create_dirs: bool,
) -> anyhow::Result<()> {
    let uri = format!("template://{}/{}", category, template);
    let params_json = params_to_json(params);

    let result = generate_template(server.as_ref(), &uri, params_json).await?;

    if let Some(path) = output {
        if create_dirs {
            tokio::fs::create_dir_all(path.parent().unwrap()).await?;
        }
        tokio::fs::write(&path, &result.content).await?;
        eprintln!("✅ Generated: {}", path.display());
    } else {
        tokio::io::stdout()
            .write_all(result.content.as_bytes())
            .await?;
    }
    Ok(())
}

#[instrument(level = "debug", skip(server))]
async fn handle_scaffold(
    server: Arc<StatelessTemplateServer>,
    toolchain: String,
    templates: Vec<String>,
    params: Vec<(String, Value)>,
    parallel: usize,
) -> anyhow::Result<()> {
    use futures::stream::{self, StreamExt};

    let params_json = params_to_json(params);
    let results = scaffold_project(
        server.clone(),
        &toolchain,
        templates,
        serde_json::Value::Object(params_json),
    )
    .await?;

    // Parallel file writing with bounded concurrency
    stream::iter(results.files)
        .map(|file| async move {
            let path = PathBuf::from(&file.path);
            if let Some(parent) = path.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
            tokio::fs::write(&path, &file.content).await?;
            eprintln!("✅ {}", file.path);
            Ok::<_, anyhow::Error>(())
        })
        .buffer_unordered(parallel)
        .collect::<Vec<_>>()
        .await;

    eprintln!("\n🚀 Project scaffolded successfully!");
    Ok(())
}

async fn handle_list(
    server: Arc<StatelessTemplateServer>,
    toolchain: Option<String>,
    category: Option<String>,
    format: OutputFormat,
) -> anyhow::Result<()> {
    let templates =
        list_templates(server.as_ref(), toolchain.as_deref(), category.as_deref()).await?;

    match format {
        OutputFormat::Table => print_table(&templates),
        OutputFormat::Json => {
            let templates_deref: Vec<&TemplateResource> =
                templates.iter().map(|t| t.as_ref()).collect();
            println!("{}", serde_json::to_string_pretty(&templates_deref)?);
        }
        OutputFormat::Yaml => {
            let templates_deref: Vec<&TemplateResource> =
                templates.iter().map(|t| t.as_ref()).collect();
            println!("{}", serde_yaml::to_string(&templates_deref)?);
        }
    }
    Ok(())
}

async fn handle_search(
    server: Arc<StatelessTemplateServer>,
    query: String,
    toolchain: Option<String>,
    limit: usize,
) -> anyhow::Result<()> {
    let results = search_templates(server.clone(), &query, toolchain.as_deref()).await?;

    for (i, result) in results.iter().take(limit).enumerate() {
        println!(
            "{:2}. {} (score: {:.2})",
            i + 1,
            result.template.uri,
            result.relevance
        );
        if !result.matches.is_empty() {
            println!("    Matches: {}", result.matches.join(", "));
        }
    }
    Ok(())
}

async fn handle_validate(
    server: Arc<StatelessTemplateServer>,
    uri: String,
    params: Vec<(String, Value)>,
) -> anyhow::Result<()> {
    let params_json = params_to_json(params);
    let result = validate_template(
        server.clone(),
        &uri,
        &serde_json::Value::Object(params_json),
    )
    .await?;

    if result.valid {
        eprintln!("✅ All parameters valid");
    } else {
        eprintln!("❌ Validation errors:");
        for error in result.errors {
            eprintln!("  - {}: {}", error.field, error.message);
        }
        std::process::exit(1);
    }
    Ok(())
}

#[instrument(level = "debug")]
async fn handle_context(
    toolchain: String,
    project_path: PathBuf,
    output: Option<PathBuf>,
    format: ContextFormat,
) -> anyhow::Result<()> {
    use crate::services::cache::{config::CacheConfig, persistent_manager::PersistentCacheManager};
    use crate::services::context::{
        analyze_project_with_persistent_cache, format_context_as_markdown,
    };
    use std::sync::Arc;

    // Create a persistent cache manager for cross-session caching
    let cache_config = CacheConfig::default();
    let cache_manager = Arc::new(
        PersistentCacheManager::with_default_dir(cache_config)
            .map_err(|e| anyhow::anyhow!("Failed to create cache manager: {}", e))?,
    );

    // Analyze the project with caching
    let context = analyze_project_with_persistent_cache(
        &project_path,
        &toolchain,
        Some(cache_manager.clone()),
    )
    .await?;

    // Print cache diagnostics
    let diagnostics = cache_manager.get_diagnostics();
    eprintln!(
        "Cache hit rate: {:.1}%, memory efficiency: {:.1}%, time saved: {}ms",
        diagnostics.effectiveness.overall_hit_rate * 100.0,
        diagnostics.effectiveness.memory_efficiency * 100.0,
        diagnostics.effectiveness.time_saved_ms
    );

    // Format the output
    let content = match format {
        ContextFormat::Markdown => format_context_as_markdown(&context),
        ContextFormat::Json => serde_json::to_string_pretty(&context)?,
    };

    // Write output
    if let Some(path) = output {
        tokio::fs::write(&path, &content).await?;
        eprintln!("✅ Context written to: {}", path.display());
    } else {
        println!("{}", content);
    }
    Ok(())
}

async fn handle_analyze_churn(
    project_path: PathBuf,
    days: u32,
    format: ChurnOutputFormat,
    output: Option<PathBuf>,
) -> anyhow::Result<()> {
    use crate::handlers::tools::{
        format_churn_as_csv, format_churn_as_markdown, format_churn_summary,
    };
    use crate::services::git_analysis::GitAnalysisService;

    let analysis = GitAnalysisService::analyze_code_churn(&project_path, days)?;

    let content = match format {
        ChurnOutputFormat::Summary => format_churn_summary(&analysis),
        ChurnOutputFormat::Markdown => format_churn_as_markdown(&analysis),
        ChurnOutputFormat::Json => serde_json::to_string_pretty(&analysis)?,
        ChurnOutputFormat::Csv => format_churn_as_csv(&analysis),
    };

    if let Some(path) = output {
        tokio::fs::write(&path, &content).await?;
        eprintln!("✅ Code churn analysis written to: {}", path.display());
    } else {
        println!("{}", content);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn handle_analyze_dag(
    dag_type: DagType,
    project_path: PathBuf,
    output: Option<PathBuf>,
    max_depth: Option<usize>,
    filter_external: bool,
    show_complexity: bool,
    include_duplicates: bool,
    include_dead_code: bool,
    enhanced: bool,
) -> anyhow::Result<()> {
    // If enhanced mode is requested, use the new vectorized architecture
    if enhanced {
        use crate::services::code_intelligence::analyze_dag_enhanced;

        let result = analyze_dag_enhanced(
            project_path.to_str().unwrap(),
            dag_type,
            max_depth,
            filter_external,
            show_complexity,
            include_duplicates,
            include_dead_code,
        )
        .await?;

        // Write output
        if let Some(path) = output {
            tokio::fs::write(&path, &result).await?;
            eprintln!(
                "✅ Enhanced dependency graph written to: {}",
                path.display()
            );
        } else {
            println!("{}", result);
        }

        return Ok(());
    }

    // Otherwise, use the existing implementation
    use crate::services::{
        context::analyze_project,
        dag_builder::{
            filter_call_edges, filter_import_edges, filter_inheritance_edges, DagBuilder,
        },
        mermaid_generator::{MermaidGenerator, MermaidOptions},
    };

    // Analyze the project to get AST information
    // We'll analyze as Rust by default, but could be enhanced
    let project_context = analyze_project(&project_path, "rust").await?;

    // Build the dependency graph
    let graph = DagBuilder::build_from_project(&project_context);

    // Apply filters based on DAG type
    let filtered_graph = match dag_type {
        DagType::CallGraph => filter_call_edges(graph),
        DagType::ImportGraph => filter_import_edges(graph),
        DagType::Inheritance => filter_inheritance_edges(graph),
        DagType::FullDependency => graph,
    };

    // Generate Mermaid output
    let generator = MermaidGenerator::new(MermaidOptions {
        max_depth,
        filter_external,
        show_complexity,
        ..Default::default()
    });

    let mermaid_output = generator.generate(&filtered_graph);

    // Add stats as comments
    let mut output_with_stats = format!(
        "{}\n%% Graph Statistics:\n%% Nodes: {}\n%% Edges: {}\n",
        mermaid_output,
        filtered_graph.nodes.len(),
        filtered_graph.edges.len()
    );

    // Add warnings if enhanced features were requested but not used
    if include_duplicates || include_dead_code {
        output_with_stats.push_str(
            "\n%% Note: Use --enhanced flag to enable duplicate detection and dead code analysis\n",
        );
    }

    // Write output
    if let Some(path) = output {
        tokio::fs::write(&path, &output_with_stats).await?;
        eprintln!("✅ Dependency graph written to: {}", path.display());
    } else {
        println!("{}", output_with_stats);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn handle_analyze_complexity(
    project_path: PathBuf,
    toolchain: Option<String>,
    format: ComplexityOutputFormat,
    output: Option<PathBuf>,
    max_cyclomatic: Option<u16>,
    max_cognitive: Option<u16>,
    include: Vec<String>,
    watch: bool,
    top_files: usize,
) -> anyhow::Result<()> {
    use crate::services::complexity::{
        aggregate_results, format_as_sarif, format_complexity_report, format_complexity_summary,
    };

    if watch {
        eprintln!("❌ Watch mode not yet implemented");
        return Ok(());
    }

    // Detect toolchain if not specified
    let detected_toolchain = detect_toolchain(&project_path, toolchain)?;

    eprintln!("🔍 Analyzing {} project complexity...", detected_toolchain);

    // Custom thresholds
    let _thresholds = build_complexity_thresholds(max_cyclomatic, max_cognitive);

    // Analyze files
    let file_metrics = analyze_project_files(&project_path, &detected_toolchain, &include).await?;

    eprintln!("📊 Analyzed {} files", file_metrics.len());

    // Aggregate results
    let report = aggregate_results(file_metrics.clone());

    // Handle top-files ranking if requested
    let mut content = match format {
        ComplexityOutputFormat::Summary => format_complexity_summary(&report),
        ComplexityOutputFormat::Full => format_complexity_report(&report),
        ComplexityOutputFormat::Json => serde_json::to_string_pretty(&report)?,
        ComplexityOutputFormat::Sarif => format_as_sarif(&report)?,
    };

    // Add top files ranking if requested
    if top_files > 0 {
        use crate::services::ranking::{rank_files_by_complexity, ComplexityRanker};

        let ranker = ComplexityRanker::default();
        let rankings = rank_files_by_complexity(&file_metrics, top_files, &ranker);

        let ranking_content = format_top_files_ranking(&rankings);

        // Prepend ranking to existing content for non-JSON formats
        match format {
            ComplexityOutputFormat::Json => {
                // For JSON, we need to merge the ranking data
                let mut report_json: serde_json::Value = serde_json::from_str(&content)?;
                if let Some(obj) = report_json.as_object_mut() {
                    obj.insert(
                        "top_files".to_string(),
                        serde_json::json!({
                            "requested": top_files,
                            "returned": rankings.len(),
                            "rankings": rankings.iter().enumerate().map(|(i, (file, score))| {
                                serde_json::json!({
                                    "rank": i + 1,
                                    "file": file,
                                    "function_count": score.function_count,
                                    "max_cyclomatic": score.cyclomatic_max,
                                    "avg_cognitive": score.cognitive_avg,
                                    "halstead_effort": score.halstead_effort,
                                    "total_score": score.total_score
                                })
                            }).collect::<Vec<_>>()
                        }),
                    );
                }
                content = serde_json::to_string_pretty(&report_json)?;
            }
            _ => {
                content = format!("{}\n{}", ranking_content, content);
            }
        }
    }

    // Write output
    if let Some(path) = output {
        tokio::fs::write(&path, &content).await?;
        eprintln!("✅ Complexity analysis written to: {}", path.display());
    } else {
        println!("{}", content);
    }
    Ok(())
}

async fn handle_analyze_dead_code(
    path: PathBuf,
    format: DeadCodeOutputFormat,
    top_files: Option<usize>,
    include_unreachable: bool,
    min_dead_lines: usize,
    include_tests: bool,
    output: Option<PathBuf>,
) -> anyhow::Result<()> {
    use crate::models::dead_code::DeadCodeAnalysisConfig;
    use crate::services::dead_code_analyzer::DeadCodeAnalyzer;

    eprintln!("☠️ Analyzing dead code in project...");

    // Create analyzer with a reasonable capacity (we'll adjust this as needed)
    let mut analyzer = DeadCodeAnalyzer::new(10000);

    // TODO: Support coverage data integration
    // if let Some(coverage_data) = load_coverage_data(&path) {
    //     analyzer = analyzer.with_coverage(coverage_data);
    // }

    // Configure analysis
    let config = DeadCodeAnalysisConfig {
        include_unreachable,
        include_tests,
        min_dead_lines,
    };

    // Run analysis with ranking
    let mut result = analyzer.analyze_with_ranking(&path, config).await?;

    // Apply top_files limit if specified
    if let Some(limit) = top_files {
        result.ranked_files.truncate(limit);
    }

    eprintln!(
        "📊 Analysis complete: {} files analyzed, {} with dead code",
        result.summary.total_files_analyzed, result.summary.files_with_dead_code
    );

    // Format and output results
    let content = format_dead_code_output(&result, &format)?;

    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &content).await?;
        eprintln!(
            "✅ Dead code analysis written to: {}",
            output_path.display()
        );
    } else {
        println!("{}", content);
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn handle_analyze_satd(
    path: PathBuf,
    format: SatdOutputFormat,
    severity: Option<SatdSeverity>,
    critical_only: bool,
    include_tests: bool,
    evolution: bool,
    days: u32,
    metrics: bool,
    output: Option<PathBuf>,
) -> anyhow::Result<()> {
    use crate::services::satd_detector::SATDDetector;

    eprintln!("🔍 Analyzing Self-Admitted Technical Debt...");

    let detector = SATDDetector::new();
    let mut results = detector.analyze_project(&path, include_tests).await?;

    // Apply severity filter if specified
    if let Some(min_severity) = severity {
        let min_level = match min_severity {
            SatdSeverity::Critical => crate::services::satd_detector::Severity::Critical,
            SatdSeverity::High => crate::services::satd_detector::Severity::High,
            SatdSeverity::Medium => crate::services::satd_detector::Severity::Medium,
            SatdSeverity::Low => crate::services::satd_detector::Severity::Low,
        };
        results
            .items
            .retain(|item| item.severity as u8 >= min_level as u8);
    }

    // Apply critical-only filter
    if critical_only {
        results.items.retain(|item| {
            matches!(
                item.severity,
                crate::services::satd_detector::Severity::Critical
            )
        });
    }

    // Handle evolution analysis
    if evolution {
        eprintln!("📈 Tracking SATD evolution over {} days...", days);
        // Note: Evolution tracking would require git history analysis
        // This is a placeholder for future implementation
    }

    // Include metrics if requested
    if metrics {
        let total_items = results.items.len();
        let by_severity = results.items.iter().fold([0; 4], |mut acc, item| {
            match item.severity {
                crate::services::satd_detector::Severity::Critical => acc[0] += 1,
                crate::services::satd_detector::Severity::High => acc[1] += 1,
                crate::services::satd_detector::Severity::Medium => acc[2] += 1,
                crate::services::satd_detector::Severity::Low => acc[3] += 1,
            }
            acc
        });
        eprintln!(
            "📊 SATD Metrics: {} total items (Critical: {}, High: {}, Medium: {}, Low: {})",
            total_items, by_severity[0], by_severity[1], by_severity[2], by_severity[3]
        );
    }

    // Format output
    let content = format_satd_output(&results, &format)?;

    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &content).await?;
        eprintln!("✅ SATD analysis written to: {}", output_path.display());
    } else {
        println!("{}", content);
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn handle_analyze_deep_context(
    project_path: PathBuf,
    output: Option<PathBuf>,
    format: DeepContextOutputFormat,
    include: Vec<String>,
    exclude: Vec<String>,
    period_days: u32,
    dag_type: DeepContextDagType,
    max_depth: Option<usize>,
    include_patterns: Vec<String>,
    exclude_patterns: Vec<String>,
    cache_strategy: DeepContextCacheStrategy,
    parallel: Option<usize>,
    verbose: bool,
) -> anyhow::Result<()> {
    use crate::services::deep_context::{
        DeepContextAnalyzer, DeepContextConfig, AnalysisType, DagType as InternalDagType, CacheStrategy as InternalCacheStrategy
    };

    if verbose {
        eprintln!("🧬 Starting comprehensive deep context analysis...");
    }

    // Build configuration from CLI args
    let mut config = DeepContextConfig {
        period_days,
        max_depth,
        include_patterns,
        exclude_patterns: {
            let mut patterns = DeepContextConfig::default().exclude_patterns;
            patterns.extend(exclude_patterns);
            patterns
        },
        ..DeepContextConfig::default()
    };
    
    if let Some(p) = parallel {
        config.parallel = p;
    }

    // Convert DAG type
    config.dag_type = match dag_type {
        DeepContextDagType::CallGraph => InternalDagType::CallGraph,
        DeepContextDagType::ImportGraph => InternalDagType::ImportGraph,
        DeepContextDagType::Inheritance => InternalDagType::Inheritance,
        DeepContextDagType::FullDependency => InternalDagType::FullDependency,
    };

    // Convert cache strategy
    config.cache_strategy = match cache_strategy {
        DeepContextCacheStrategy::Normal => InternalCacheStrategy::Normal,
        DeepContextCacheStrategy::ForceRefresh => InternalCacheStrategy::ForceRefresh,
        DeepContextCacheStrategy::Offline => InternalCacheStrategy::Offline,
    };

    // Parse include/exclude filters
    if !include.is_empty() {
        config.include_analyses = include.iter().filter_map(|s| match s.as_str() {
            "ast" => Some(AnalysisType::Ast),
            "complexity" => Some(AnalysisType::Complexity),
            "churn" => Some(AnalysisType::Churn),
            "dag" => Some(AnalysisType::Dag),
            "dead-code" => Some(AnalysisType::DeadCode),
            "satd" => Some(AnalysisType::Satd),
            "defect-probability" => Some(AnalysisType::DefectProbability),
            _ => {
                eprintln!("⚠️  Unknown analysis type: {}", s);
                None
            }
        }).collect();
    }

    // Remove excluded analyses
    for exclude_item in &exclude {
        match exclude_item.as_str() {
            "ast" => config.include_analyses.retain(|a| !matches!(a, AnalysisType::Ast)),
            "complexity" => config.include_analyses.retain(|a| !matches!(a, AnalysisType::Complexity)),
            "churn" => config.include_analyses.retain(|a| !matches!(a, AnalysisType::Churn)),
            "dag" => config.include_analyses.retain(|a| !matches!(a, AnalysisType::Dag)),
            "dead-code" => config.include_analyses.retain(|a| !matches!(a, AnalysisType::DeadCode)),
            "satd" => config.include_analyses.retain(|a| !matches!(a, AnalysisType::Satd)),
            "defect-probability" => config.include_analyses.retain(|a| !matches!(a, AnalysisType::DefectProbability)),
            _ => eprintln!("⚠️  Unknown analysis type to exclude: {}", exclude_item),
        }
    }

    if verbose {
        eprintln!("📊 Analysis configuration:");
        eprintln!("  - Analyses: {:?}", config.include_analyses);
        eprintln!("  - Period: {} days", config.period_days);
        eprintln!("  - DAG type: {:?}", config.dag_type);
        eprintln!("  - Parallelism: {}", config.parallel);
    }

    // Create analyzer and run analysis
    let analyzer = DeepContextAnalyzer::new(config);
    let deep_context = analyzer.analyze_project(&project_path).await?;

    if verbose {
        eprintln!("✅ Analysis completed in {:?}", deep_context.metadata.analysis_duration);
        eprintln!("📈 Quality score: {:.1}/100", deep_context.quality_scorecard.overall_health);
        eprintln!("🔍 Defects found: {}", deep_context.defect_summary.total_defects);
    }

    // Format output
    let content = match format {
        DeepContextOutputFormat::Markdown => format_deep_context_as_markdown(&deep_context)?,
        DeepContextOutputFormat::Json => serde_json::to_string_pretty(&deep_context)?,
        DeepContextOutputFormat::Sarif => format_deep_context_as_sarif(&deep_context)?,
    };

    // Write output
    if let Some(path) = output {
        tokio::fs::write(&path, &content).await?;
        eprintln!("✅ Deep context analysis written to: {}", path.display());
    } else {
        println!("{}", content);
    }

    Ok(())
}

fn format_deep_context_as_markdown(context: &crate::services::deep_context::DeepContext) -> anyhow::Result<String> {
    let mut output = String::new();
    
    output.push_str(&format!("# Deep Context Analysis: {}\n\n", 
        context.metadata.project_root.file_name()
            .unwrap_or_default()
            .to_string_lossy()));
    
    output.push_str(&format!("**Generated:** {}\n", context.metadata.generated_at.format("%Y-%m-%d %H:%M:%S UTC")));
    output.push_str(&format!("**Tool Version:** {}\n", context.metadata.tool_version));
    output.push_str(&format!("**Analysis Time:** {:?}\n\n", context.metadata.analysis_duration));
    
    // Executive Summary
    output.push_str("## Executive Summary\n\n");
    output.push_str(&format!("**Overall Health Score:** {:.1}/100", context.quality_scorecard.overall_health));
    
    let health_emoji = if context.quality_scorecard.overall_health >= 80.0 {
        " ✅"
    } else if context.quality_scorecard.overall_health >= 60.0 {
        " ⚠️"
    } else {
        " ❌"
    };
    output.push_str(health_emoji);
    output.push_str("\n\n");
    
    output.push_str(&format!("- **Complexity Score:** {:.1}/100\n", context.quality_scorecard.complexity_score));
    output.push_str(&format!("- **Maintainability Index:** {:.1}/100\n", context.quality_scorecard.maintainability_index));
    output.push_str(&format!("- **Modularity Score:** {:.1}/100\n", context.quality_scorecard.modularity_score));
    output.push_str(&format!("- **Technical Debt:** {:.1} hours estimated\n\n", context.quality_scorecard.technical_debt_hours));
    
    // Defect Summary
    if context.defect_summary.total_defects > 0 {
        output.push_str("### Critical Issues Found\n\n");
        output.push_str(&format!("- **Total Defects:** {}\n", context.defect_summary.total_defects));
        
        for (defect_type, count) in &context.defect_summary.by_type {
            let emoji = match defect_type.as_str() {
                "dead_code" => "☠️",
                "technical_debt" => "🔧",
                "complexity" => "🔥",
                _ => "⚠️",
            };
            output.push_str(&format!("- {} **{}:** {}\n", emoji, 
                defect_type.replace('_', " ").to_title_case(), count));
        }
        output.push('\n');
    }
    
    // Recommendations
    if !context.recommendations.is_empty() {
        output.push_str("## Recommendations\n\n");
        
        for (i, rec) in context.recommendations.iter().enumerate() {
            let priority_emoji = match rec.priority {
                crate::services::deep_context::Priority::Critical => "🔴",
                crate::services::deep_context::Priority::High => "🟠", 
                crate::services::deep_context::Priority::Medium => "🟡",
                crate::services::deep_context::Priority::Low => "🟢",
            };
            
            output.push_str(&format!("{}. {} {:?} **{}**\n", 
                i + 1, priority_emoji, rec.priority, rec.title));
            output.push_str(&format!("   - {}\n", rec.description));
            output.push_str(&format!("   - **Estimated Effort:** {:?}\n", rec.estimated_effort));
            output.push_str(&format!("   - **Impact:** {:?}\n\n", rec.impact));
        }
    }
    
    // Project Structure
    output.push_str("## Project Structure\n\n");
    output.push_str(&format!("- **Total Files:** {}\n", context.file_tree.total_files));
    output.push_str(&format!("- **Total Size:** {:.1} MB\n\n", context.file_tree.total_size_bytes as f64 / 1_048_576.0));
    
    // Analysis Results Summary
    output.push_str("## Analysis Results\n\n");
    
    if let Some(ref complexity) = context.analyses.complexity_report {
        output.push_str("### Complexity Analysis\n");
        if complexity.summary.avg_cyclomatic > 0.0 {
            let avg_complexity = complexity.summary.avg_cyclomatic;
            output.push_str(&format!("- **Average Cyclomatic Complexity:** {:.1}\n", avg_complexity));
        }
        output.push_str(&format!("- **Total Violations:** {}\n\n", complexity.violations.len()));
    }
    
    if let Some(ref churn) = context.analyses.churn_analysis {
        output.push_str("### Code Churn Analysis\n");
        output.push_str(&format!("- **Total Commits:** {}\n", churn.summary.total_commits));
        output.push_str(&format!("- **Files Changed:** {}\n\n", churn.files.len()));
    }
    
    if let Some(ref dead_code) = context.analyses.dead_code_results {
        output.push_str("### Dead Code Analysis\n");
        output.push_str(&format!("- **Dead Functions:** {}\n", dead_code.summary.dead_functions));
        output.push_str(&format!("- **Dead Percentage:** {:.1}%\n\n", dead_code.summary.dead_percentage));
    }
    
    if let Some(ref satd) = context.analyses.satd_results {
        output.push_str("### Technical Debt Analysis\n");
        output.push_str(&format!("- **SATD Items:** {}\n\n", satd.items.len()));
    }
    
    Ok(output)
}

fn format_deep_context_as_sarif(context: &crate::services::deep_context::DeepContext) -> anyhow::Result<String> {
    use serde_json::json;
    
    let mut results = Vec::new();
    
    // Add results from different analyses
    if let Some(ref dead_code) = context.analyses.dead_code_results {
        for file_metrics in &dead_code.ranked_files {
            for item in &file_metrics.items {
                results.push(json!({
                    "ruleId": "deep-context-dead-code",
                    "level": "warning",
                    "message": {
                        "text": format!("Dead {}: {}", 
                            format!("{:?}", item.item_type).to_lowercase(), 
                            item.reason)
                    },
                    "locations": [{
                        "physicalLocation": {
                            "artifactLocation": {
                                "uri": file_metrics.path
                            },
                            "region": {
                                "startLine": item.line
                            }
                        }
                    }],
                    "properties": {
                        "confidence": format!("{:?}", file_metrics.confidence),
                        "analysisType": "dead-code"
                    }
                }));
            }
        }
    }
    
    if let Some(ref satd) = context.analyses.satd_results {
        for item in &satd.items {
            let level = match item.severity {
                crate::services::satd_detector::Severity::Critical => "error",
                crate::services::satd_detector::Severity::High => "warning", 
                crate::services::satd_detector::Severity::Medium => "note",
                crate::services::satd_detector::Severity::Low => "info",
            };
            
            results.push(json!({
                "ruleId": "deep-context-technical-debt",
                "level": level,
                "message": {
                    "text": format!("Technical debt: {}", item.text.trim())
                },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": item.file.display().to_string()
                        },
                        "region": {
                            "startLine": item.line,
                            "startColumn": item.column
                        }
                    }
                }],
                "properties": {
                    "category": format!("{:?}", item.category),
                    "severity": format!("{:?}", item.severity),
                    "analysisType": "technical-debt"
                }
            }));
        }
    }
    
    let sarif = json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "paiml-mcp-agent-toolkit",
                    "version": context.metadata.tool_version,
                    "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit"
                }
            },
            "results": results,
            "properties": {
                "overallHealthScore": context.quality_scorecard.overall_health,
                "complexityScore": context.quality_scorecard.complexity_score,
                "maintainabilityIndex": context.quality_scorecard.maintainability_index,
                "technicalDebtHours": context.quality_scorecard.technical_debt_hours,
                "analysisTimestamp": context.metadata.generated_at.to_rfc3339(),
                "analysisDuration": format!("{:?}", context.metadata.analysis_duration)
            }
        }]
    });
    
    Ok(serde_json::to_string_pretty(&sarif)?)
}

// Helper trait for string title case
trait ToTitleCase {
    fn to_title_case(&self) -> String;
}

impl ToTitleCase for str {
    fn to_title_case(&self) -> String {
        self.split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str().to_lowercase().as_str(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Format SATD analysis output
fn format_satd_output(
    result: &crate::services::satd_detector::SATDAnalysisResult,
    format: &SatdOutputFormat,
) -> anyhow::Result<String> {
    match format {
        SatdOutputFormat::Summary => format_satd_summary(result),
        SatdOutputFormat::Json => Ok(serde_json::to_string_pretty(result)?),
        SatdOutputFormat::Sarif => format_satd_as_sarif(result),
        SatdOutputFormat::Markdown => format_satd_as_markdown(result),
    }
}

/// Format SATD analysis as summary text
fn format_satd_summary(
    result: &crate::services::satd_detector::SATDAnalysisResult,
) -> anyhow::Result<String> {
    let mut output = String::new();

    output.push_str("Self-Admitted Technical Debt Analysis:\n");
    output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    output.push_str(&format!("  Total SATD items: {}\n", result.items.len()));

    // Count by severity
    let by_severity = result.items.iter().fold([0; 4], |mut acc, item| {
        match item.severity {
            crate::services::satd_detector::Severity::Critical => acc[0] += 1,
            crate::services::satd_detector::Severity::High => acc[1] += 1,
            crate::services::satd_detector::Severity::Medium => acc[2] += 1,
            crate::services::satd_detector::Severity::Low => acc[3] += 1,
        }
        acc
    });

    output.push_str(&format!("  Critical: {}\n", by_severity[0]));
    output.push_str(&format!("  High: {}\n", by_severity[1]));
    output.push_str(&format!("  Medium: {}\n", by_severity[2]));
    output.push_str(&format!("  Low: {}\n", by_severity[3]));

    // Count by category
    let mut by_category = std::collections::HashMap::new();
    for item in &result.items {
        *by_category.entry(item.category).or_insert(0) += 1;
    }

    output.push('\n');
    output.push_str("By Category:\n");
    for (category, count) in by_category {
        output.push_str(&format!("  {:?}: {}\n", category, count));
    }

    // Show top items if any
    if !result.items.is_empty() {
        output.push('\n');
        output.push_str("🔺 Top Critical Items:\n");
        output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

        let critical_items: Vec<_> = result
            .items
            .iter()
            .filter(|item| {
                matches!(
                    item.severity,
                    crate::services::satd_detector::Severity::Critical
                )
            })
            .take(5)
            .collect();

        if critical_items.is_empty() {
            output.push_str("  No critical items found.\n");
        } else {
            for (i, item) in critical_items.iter().enumerate() {
                output.push_str(&format!(
                    "{}. {} ({}:{})\n",
                    i + 1,
                    item.text.trim(),
                    item.file.display(),
                    item.line
                ));
                output.push_str(&format!("   Category: {:?}\n", item.category));
                output.push('\n');
            }
        }
    }

    Ok(output)
}

/// Format SATD analysis as SARIF (Static Analysis Results Interchange Format)
fn format_satd_as_sarif(
    result: &crate::services::satd_detector::SATDAnalysisResult,
) -> anyhow::Result<String> {
    use serde_json::json;

    let mut results = Vec::new();

    for item in &result.items {
        let level = match item.severity {
            crate::services::satd_detector::Severity::Critical => "error",
            crate::services::satd_detector::Severity::High => "warning",
            crate::services::satd_detector::Severity::Medium => "note",
            crate::services::satd_detector::Severity::Low => "info",
        };

        results.push(json!({
            "ruleId": format!("satd-{}", format!("{:?}", item.category).to_lowercase()),
            "level": level,
            "message": {
                "text": format!("Self-admitted technical debt: {}", item.text.trim())
            },
            "locations": [{
                "physicalLocation": {
                    "artifactLocation": {
                        "uri": item.file.display().to_string()
                    },
                    "region": {
                        "startLine": item.line,
                        "startColumn": item.column
                    }
                }
            }]
        }));
    }

    let sarif = json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "paiml-mcp-agent-toolkit",
                    "version": "0.1.0",
                    "informationUri": "https://github.com/paiml/mcp-agent-toolkit"
                }
            },
            "results": results
        }]
    });

    Ok(serde_json::to_string_pretty(&sarif)?)
}

/// Format SATD analysis as Markdown
fn format_satd_as_markdown(
    result: &crate::services::satd_detector::SATDAnalysisResult,
) -> anyhow::Result<String> {
    let mut output = String::new();

    output.push_str("# Self-Admitted Technical Debt Report\n\n");
    output.push_str(&format!(
        "**Analysis Date:** {}\n\n",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    ));

    // Summary section
    output.push_str("## Summary\n\n");
    output.push_str(&format!("- **Total SATD items:** {}\n", result.items.len()));

    // Count by severity
    let by_severity = result.items.iter().fold([0; 4], |mut acc, item| {
        match item.severity {
            crate::services::satd_detector::Severity::Critical => acc[0] += 1,
            crate::services::satd_detector::Severity::High => acc[1] += 1,
            crate::services::satd_detector::Severity::Medium => acc[2] += 1,
            crate::services::satd_detector::Severity::Low => acc[3] += 1,
        }
        acc
    });

    output.push_str(&format!("- **Critical:** {}\n", by_severity[0]));
    output.push_str(&format!("- **High:** {}\n", by_severity[1]));
    output.push_str(&format!("- **Medium:** {}\n", by_severity[2]));
    output.push_str(&format!("- **Low:** {}\n\n", by_severity[3]));

    // Items by severity
    if !result.items.is_empty() {
        output.push_str("## Technical Debt Items\n\n");
        output.push_str("| Severity | Category | File | Line | Description |\n");
        output.push_str("|----------|----------|------|------|-------------|\n");

        for item in &result.items {
            let severity_emoji = match item.severity {
                crate::services::satd_detector::Severity::Critical => "🔴",
                crate::services::satd_detector::Severity::High => "🟠",
                crate::services::satd_detector::Severity::Medium => "🟡",
                crate::services::satd_detector::Severity::Low => "🟢",
            };

            output.push_str(&format!(
                "| {} {:?} | {:?} | `{}` | {} | {} |\n",
                severity_emoji,
                item.severity,
                item.category,
                item.file.display(),
                item.line,
                item.text.trim().replace('|', "\\|").replace('\n', " ")
            ));
        }
        output.push('\n');
    }

    Ok(output)
}

/// Format dead code analysis output
fn format_dead_code_output(
    result: &crate::models::dead_code::DeadCodeRankingResult,
    format: &DeadCodeOutputFormat,
) -> anyhow::Result<String> {
    match format {
        DeadCodeOutputFormat::Summary => format_dead_code_summary(result),
        DeadCodeOutputFormat::Json => Ok(serde_json::to_string_pretty(result)?),
        DeadCodeOutputFormat::Sarif => format_dead_code_as_sarif(result),
        DeadCodeOutputFormat::Markdown => format_dead_code_as_markdown(result),
    }
}

/// Format dead code analysis as summary text
fn format_dead_code_summary(
    result: &crate::models::dead_code::DeadCodeRankingResult,
) -> anyhow::Result<String> {
    let mut output = String::new();

    output.push_str("Dead Code Analysis Summary:\n");
    output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    output.push_str(&format!(
        "  Total files analyzed: {}\n",
        result.summary.total_files_analyzed
    ));
    output.push_str(&format!(
        "  Files with dead code: {} ({:.1}%)\n",
        result.summary.files_with_dead_code,
        if result.summary.total_files_analyzed > 0 {
            (result.summary.files_with_dead_code as f32
                / result.summary.total_files_analyzed as f32)
                * 100.0
        } else {
            0.0
        }
    ));
    output.push('\n');
    output.push_str(&format!(
        "  Total dead lines: {} ({:.1}% of codebase)\n",
        result.summary.total_dead_lines, result.summary.dead_percentage
    ));
    output.push_str(&format!(
        "  Dead functions: {}\n",
        result.summary.dead_functions
    ));
    output.push_str(&format!(
        "  Dead classes: {}\n",
        result.summary.dead_classes
    ));
    output.push_str(&format!(
        "  Dead modules: {}\n",
        result.summary.dead_modules
    ));
    output.push_str(&format!(
        "  Unreachable blocks: {}\n",
        result.summary.unreachable_blocks
    ));

    // Show top files if available
    if !result.ranked_files.is_empty() {
        let top_count = result.ranked_files.len().min(5);
        output.push_str(&format!(
            "\n🏆 Top {} Files with Most Dead Code:\n",
            top_count
        ));
        output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

        for (i, file_metrics) in result.ranked_files.iter().take(top_count).enumerate() {
            let confidence_text = match file_metrics.confidence {
                crate::models::dead_code::ConfidenceLevel::High => "HIGH",
                crate::models::dead_code::ConfidenceLevel::Medium => "MEDIUM",
                crate::models::dead_code::ConfidenceLevel::Low => "LOW",
            };

            output.push_str(&format!(
                "{}. {} (Score: {:.1}) [{}confidence]\n",
                i + 1,
                file_metrics.path,
                file_metrics.dead_score,
                confidence_text
            ));
            output.push_str(&format!(
                "   └─ {} dead lines ({:.1}% of file)\n",
                file_metrics.dead_lines, file_metrics.dead_percentage
            ));
            if file_metrics.dead_functions > 0 || file_metrics.dead_classes > 0 {
                output.push_str(&format!(
                    "   └─ {} functions, {} classes\n",
                    file_metrics.dead_functions, file_metrics.dead_classes
                ));
            }

            // Add recommendation based on percentage
            let recommendation = if file_metrics.dead_percentage > 80.0 {
                "Recommendation: Consider removing entire file"
            } else if file_metrics.dead_percentage > 50.0 {
                "Recommendation: Review and remove unused sections"
            } else if file_metrics.dead_percentage > 20.0 {
                "Recommendation: Clean up dead code items"
            } else {
                "Recommendation: Minor cleanup needed"
            };
            output.push_str(&format!("   └─ {}\n", recommendation));
            output.push('\n');
        }
    }

    Ok(output)
}

/// Format dead code analysis as SARIF (Static Analysis Results Interchange Format)
fn format_dead_code_as_sarif(
    result: &crate::models::dead_code::DeadCodeRankingResult,
) -> anyhow::Result<String> {
    use serde_json::json;

    let mut results = Vec::new();

    for file_metrics in &result.ranked_files {
        for item in &file_metrics.items {
            results.push(json!({
                "ruleId": format!("dead-code-{}", format!("{:?}", item.item_type).to_lowercase()),
                "level": "info",
                "message": {
                    "text": format!("Dead {} '{}': {}",
                        format!("{:?}", item.item_type).to_lowercase(),
                        item.name,
                        item.reason
                    )
                },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": file_metrics.path
                        },
                        "region": {
                            "startLine": item.line
                        }
                    }
                }]
            }));
        }
    }

    let sarif = json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "paiml-mcp-agent-toolkit",
                    "version": "0.1.0",
                    "informationUri": "https://github.com/paiml/mcp-agent-toolkit"
                }
            },
            "results": results
        }]
    });

    Ok(serde_json::to_string_pretty(&sarif)?)
}

/// Format dead code analysis as Markdown
fn format_dead_code_as_markdown(
    result: &crate::models::dead_code::DeadCodeRankingResult,
) -> anyhow::Result<String> {
    let mut output = String::new();

    output.push_str("# Dead Code Analysis Report\n\n");
    output.push_str(&format!(
        "**Analysis Date:** {}\n\n",
        result.analysis_timestamp.format("%Y-%m-%d %H:%M:%S UTC")
    ));

    // Summary section
    output.push_str("## Summary\n\n");
    output.push_str(&format!(
        "- **Total files analyzed:** {}\n",
        result.summary.total_files_analyzed
    ));
    output.push_str(&format!(
        "- **Files with dead code:** {} ({:.1}%)\n",
        result.summary.files_with_dead_code,
        if result.summary.total_files_analyzed > 0 {
            (result.summary.files_with_dead_code as f32
                / result.summary.total_files_analyzed as f32)
                * 100.0
        } else {
            0.0
        }
    ));
    output.push_str(&format!(
        "- **Total dead lines:** {} ({:.1}% of codebase)\n",
        result.summary.total_dead_lines, result.summary.dead_percentage
    ));
    output.push_str(&format!(
        "- **Dead functions:** {}\n",
        result.summary.dead_functions
    ));
    output.push_str(&format!(
        "- **Dead classes:** {}\n",
        result.summary.dead_classes
    ));
    output.push_str(&format!(
        "- **Dead modules:** {}\n",
        result.summary.dead_modules
    ));
    output.push_str(&format!(
        "- **Unreachable blocks:** {}\n\n",
        result.summary.unreachable_blocks
    ));

    // Configuration section
    output.push_str("## Configuration\n\n");
    output.push_str(&format!(
        "- **Include unreachable code:** {}\n",
        result.config.include_unreachable
    ));
    output.push_str(&format!(
        "- **Include test files:** {}\n",
        result.config.include_tests
    ));
    output.push_str(&format!(
        "- **Minimum dead lines threshold:** {}\n\n",
        result.config.min_dead_lines
    ));

    // Top files section
    if !result.ranked_files.is_empty() {
        output.push_str("## Top Files with Dead Code\n\n");
        output.push_str("| Rank | File | Dead Lines | Percentage | Functions | Classes | Score | Confidence |\n");
        output.push_str("|------|------|------------|------------|-----------|---------|-------|------------|\n");

        for (i, file_metrics) in result.ranked_files.iter().enumerate() {
            let confidence_text = match file_metrics.confidence {
                crate::models::dead_code::ConfidenceLevel::High => "🔴 High",
                crate::models::dead_code::ConfidenceLevel::Medium => "🟡 Medium",
                crate::models::dead_code::ConfidenceLevel::Low => "🟢 Low",
            };

            output.push_str(&format!(
                "| {:>4} | `{}` | {:>10} | {:>9.1}% | {:>9} | {:>7} | {:>5.1} | {} |\n",
                i + 1,
                file_metrics.path,
                file_metrics.dead_lines,
                file_metrics.dead_percentage,
                file_metrics.dead_functions,
                file_metrics.dead_classes,
                file_metrics.dead_score,
                confidence_text
            ));
        }
        output.push('\n');
    }

    Ok(output)
}

// Helper functions for complexity analysis

fn detect_toolchain(project_path: &Path, toolchain: Option<String>) -> anyhow::Result<String> {
    if let Some(t) = toolchain {
        return Ok(t);
    }

    if project_path.join("Cargo.toml").exists() {
        Ok("rust".to_string())
    } else if project_path.join("package.json").exists() || project_path.join("deno.json").exists()
    {
        Ok("deno".to_string())
    } else if project_path.join("pyproject.toml").exists()
        || project_path.join("requirements.txt").exists()
    {
        Ok("python-uv".to_string())
    } else {
        eprintln!("⚠️  Could not detect toolchain, defaulting to rust");
        Ok("rust".to_string())
    }
}

fn build_complexity_thresholds(
    max_cyclomatic: Option<u16>,
    max_cognitive: Option<u16>,
) -> crate::services::complexity::ComplexityThresholds {
    use crate::services::complexity::ComplexityThresholds;

    let mut thresholds = ComplexityThresholds::default();
    if let Some(max) = max_cyclomatic {
        thresholds.cyclomatic_error = max;
        thresholds.cyclomatic_warn = (max * 3 / 4).max(1);
    }
    if let Some(max) = max_cognitive {
        thresholds.cognitive_error = max;
        thresholds.cognitive_warn = (max * 3 / 4).max(1);
    }
    thresholds
}

async fn analyze_project_files(
    project_path: &PathBuf,
    toolchain: &str,
    include_patterns: &[String],
) -> anyhow::Result<Vec<crate::services::complexity::FileComplexityMetrics>> {
    use walkdir::WalkDir;

    let mut file_metrics = Vec::new();

    for entry in WalkDir::new(project_path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        // Skip directories and non-source files
        if path.is_dir() {
            continue;
        }

        // Check file extension based on toolchain
        if !should_analyze_file(path, toolchain) {
            continue;
        }

        // Apply include filters if specified
        if !include_patterns.is_empty() && !matches_include_patterns(path, include_patterns) {
            continue;
        }

        // Analyze file complexity
        if let Some(metrics) = analyze_file_by_toolchain(path, toolchain).await {
            file_metrics.push(metrics);
        }
    }

    Ok(file_metrics)
}

fn should_analyze_file(path: &std::path::Path, toolchain: &str) -> bool {
    match toolchain {
        "rust" => path.extension().and_then(|s| s.to_str()) == Some("rs"),
        "deno" => matches!(
            path.extension().and_then(|s| s.to_str()),
            Some("ts") | Some("tsx") | Some("js") | Some("jsx")
        ),
        "python-uv" => path.extension().and_then(|s| s.to_str()) == Some("py"),
        _ => false,
    }
}

fn matches_include_patterns(path: &std::path::Path, patterns: &[String]) -> bool {
    let path_str = path.to_string_lossy();
    patterns.iter().any(|pattern| {
        // Simple glob-like matching
        if pattern.contains("**") {
            // Match any path containing the pattern after **
            let parts: Vec<&str> = pattern.split("**").collect();
            if parts.len() == 2 {
                path_str.contains(parts[1].trim_start_matches('/'))
            } else {
                false
            }
        } else if pattern.starts_with("*.") {
            // Match by extension
            path_str.ends_with(&pattern[1..])
        } else {
            // Direct substring match
            path_str.contains(pattern)
        }
    })
}

async fn analyze_file_by_toolchain(
    path: &std::path::Path,
    toolchain: &str,
) -> Option<crate::services::complexity::FileComplexityMetrics> {
    match toolchain {
        "rust" => {
            use crate::services::ast_rust;
            ast_rust::analyze_rust_file_with_complexity(path).await.ok()
        }
        "deno" => {
            use crate::services::ast_typescript;
            ast_typescript::analyze_typescript_file_with_complexity(path)
                .await
                .ok()
        }
        "python-uv" => {
            use crate::services::ast_python;
            ast_python::analyze_python_file_with_complexity(path)
                .await
                .ok()
        }
        _ => None,
    }
}

fn params_to_json(params: Vec<(String, Value)>) -> serde_json::Map<String, Value> {
    let mut map = serde_json::Map::new();
    for (k, v) in params {
        map.insert(k, v);
    }
    map
}

/// Format top files ranking table
fn format_top_files_ranking(
    rankings: &[(String, crate::services::ranking::CompositeComplexityScore)],
) -> String {
    if rankings.is_empty() {
        return "## Top Complex Files\n\nNo files found.\n".to_string();
    }

    let mut output = format!("## Top {} Most Complex Files\n\n", rankings.len());

    // Add table header
    output.push_str(
        "| Rank | File | Functions | Max Cyclomatic | Avg Cognitive | Halstead | Score |\n",
    );
    output.push_str(
        "|------|------|-----------|----------------|---------------|----------|-------|\n",
    );

    // Add table rows
    for (i, (file, score)) in rankings.iter().enumerate() {
        output.push_str(&format!(
            "| {:>4} | {:<50} | {:>9} | {:>14} | {:>13.1} | {:>8.1} | {:>5.1} |\n",
            i + 1,
            file,
            score.function_count,
            score.cyclomatic_max,
            score.cognitive_avg,
            score.halstead_effort,
            score.total_score
        ));
    }

    output.push('\n');
    output
}


/// Handle the serve command to start HTTP API server
async fn handle_serve(host: String, port: u16, cors: bool) -> anyhow::Result<()> {
    info!("🚀 Starting HTTP API server on {}:{}", host, port);
    
    use crate::unified_protocol::service::UnifiedService;
    use std::net::SocketAddr;
    
    // Create UnifiedService which contains the router
    let service = UnifiedService::new();
    
    // Get the router from the service (it already has all middleware and extensions)
    let mut app = service.router();
    
    // Add CORS if enabled (this is the only additional middleware we need)
    if cors {
        use tower_http::cors::CorsLayer;
        info!("🌐 CORS enabled for cross-origin requests");
        app = app.layer(CorsLayer::permissive());
    }
    
    // Parse bind address
    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
    
    // Create TCP listener
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("✅ HTTP API server listening on http://{}", addr);
    
    // Display available endpoints
    eprintln!("📡 Available endpoints:");
    eprintln!("  • GET  /health                          - Health check");
    eprintln!("  • GET  /metrics                         - Service metrics");
    eprintln!("  • GET  /api/v1/templates                - List templates");
    eprintln!("  • GET  /api/v1/templates/{{id}}           - Get template details");
    eprintln!("  • POST /api/v1/generate                 - Generate template");
    eprintln!("  • GET  /api/v1/analyze/complexity       - Complexity analysis (with query params)");
    eprintln!("  • POST /api/v1/analyze/complexity       - Complexity analysis (with JSON body)");
    eprintln!("  • POST /api/v1/analyze/churn            - Code churn analysis");
    eprintln!("  • POST /api/v1/analyze/dag              - Dependency graph analysis");
    eprintln!("  • POST /api/v1/analyze/context          - Generate project context");
    eprintln!("  • POST /api/v1/analyze/dead-code        - Dead code analysis");
    eprintln!("  • POST /api/v1/analyze/deep-context     - Deep context analysis");
    eprintln!("  • POST /mcp/{{method}}                    - MCP protocol endpoint");
    eprintln!();
    eprintln!("💡 Example: curl http://{}:{}/api/v1/analyze/complexity?top_files=5", host, port);
    eprintln!("💡 Example: curl http://{}:{}/health", host, port);
    eprintln!();
    eprintln!("🛑 Press Ctrl+C to stop the server");
    
    // Start the server
    axum::serve(listener, app).await?;
    
    Ok(())
}

fn print_table(templates: &[Arc<TemplateResource>]) {
    // Calculate column widths
    let uri_width = templates.iter().map(|t| t.uri.len()).max().unwrap_or(20);

    // Print header
    println!(
        "{:<width$} {:>10} {:>12} {:>8}",
        "URI",
        "Toolchain",
        "Category",
        "Params",
        width = uri_width
    );
    println!("{}", "─".repeat(uri_width + 35));

    // Print rows
    for template in templates {
        println!(
            "{:<width$} {:>10} {:>12} {:>8}",
            template.uri,
            format!("{:?}", template.toolchain),
            format!("{:?}", template.category),
            template.parameters.len(),
            width = uri_width
        );
    }
}
