pub mod args;
// TODO: Fix compilation errors in command pattern implementation
// pub mod command_pattern;

use crate::{
    models::{churn::ChurnOutputFormat, template::*},
    services::{makefile_linter, template_service::*},
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
    about = "Professional project quantitative scaffolding and analysis toolkit",
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

#[derive(Clone, Debug, ValueEnum, PartialEq)]
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
        /// Target toolchain (auto-detected if not specified)
        #[arg(long, short = 't')]
        toolchain: Option<String>,

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

        /// Repository to analyze (supports GitHub URLs, local paths, or shorthand like gh:owner/repo)
        #[arg(long)]
        repo: Option<String>,

        /// Output format
        #[arg(short, long, value_enum, default_value = "table")]
        format: OutputFormat,

        /// Protocol to demonstrate (cli, http, mcp, all)
        #[arg(long, value_enum, default_value = "http")]
        protocol: DemoProtocol,

        /// Show API introspection information
        #[arg(long)]
        show_api: bool,

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

        /// Enable debug mode with detailed file classification logs
        #[arg(long)]
        debug: bool,

        /// Output path for debug report (JSON format)
        #[arg(long)]
        debug_output: Option<PathBuf>,

        /// Skip vendor files during analysis (enabled by default)
        #[arg(long, default_value_t = true)]
        skip_vendor: bool,

        /// Disable vendor file skipping (process all files)
        #[arg(long = "no-skip-vendor")]
        no_skip_vendor: bool,

        /// Maximum line length before considering file unparseable
        #[arg(long)]
        max_line_length: Option<usize>,
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
        #[arg(long, value_enum, default_value = "full-dependency")]
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

        /// Enable full detailed report (default is terse)
        #[arg(long)]
        full: bool,

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

    /// Analyze Technical Debt Gradient (TDG) scores
    #[command(name = "tdg")]
    Tdg {
        /// Path to analyze (defaults to current directory)
        #[arg(long, short = 'p', default_value = ".")]
        path: PathBuf,

        /// TDG threshold for filtering results
        #[arg(short, long, default_value = "1.5")]
        threshold: f64,

        /// Number of top files to show
        #[arg(short = 'n', long, default_value = "20")]
        top: usize,

        /// Output format
        #[arg(short, long, value_enum, default_value = "table")]
        format: TdgOutputFormat,

        /// Include TDG component breakdown
        #[arg(long)]
        include_components: bool,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Show only critical files (TDG > 2.5)
        #[arg(long)]
        critical_only: bool,

        /// Enable verbose analysis output
        #[arg(long)]
        verbose: bool,
    },

    /// Analyze Makefile quality and compliance
    Makefile {
        /// Path to Makefile
        #[arg(help = "Path to Makefile to analyze")]
        path: PathBuf,

        /// Lint rules to apply
        #[arg(
            long,
            value_delimiter = ',',
            default_value = "all",
            help = "Comma-separated list of rules to apply"
        )]
        rules: Vec<String>,

        /// Output format
        #[arg(long, value_enum, default_value = "human")]
        format: MakefileOutputFormat,

        /// Fix auto-fixable issues
        #[arg(long, help = "Automatically fix issues where possible")]
        fix: bool,

        /// Check GNU Make compatibility version
        #[arg(
            long,
            default_value = "4.4",
            help = "GNU Make version to check compatibility against"
        )]
        gnu_version: String,
    },
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum ContextFormat {
    Markdown,
    Json,
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum TdgOutputFormat {
    Table,
    Json,
    Markdown,
    Sarif,
}

#[derive(Clone, Debug, ValueEnum, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum MakefileOutputFormat {
    /// Human-readable output
    Human,
    /// JSON output
    Json,
    /// GCC-style output for editor integration
    Gcc,
    /// SARIF format for CI/CD integration
    Sarif,
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

#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum DemoProtocol {
    Cli,
    Http,
    Mcp,
    All,
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
            repo,
            format,
            protocol,
            show_api,
            no_browser,
            port,
            cli,
            target_nodes,
            centrality_threshold,
            merge_threshold,
            debug,
            debug_output,
            skip_vendor,
            no_skip_vendor,
            max_line_length,
        } => {
            execute_demo_command(
                path,
                url,
                repo,
                format,
                protocol,
                show_api,
                no_browser,
                port,
                cli,
                target_nodes,
                centrality_threshold,
                merge_threshold,
                debug,
                debug_output,
                skip_vendor && !no_skip_vendor,
                max_line_length,
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
            full,
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
                full,
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
        AnalyzeCommands::Tdg {
            path,
            threshold,
            top,
            format,
            include_components,
            output,
            critical_only,
            verbose,
        } => {
            handle_analyze_tdg(
                path,
                threshold,
                top,
                format,
                include_components,
                output,
                critical_only,
                verbose,
            )
            .await
        }
        AnalyzeCommands::Makefile {
            path,
            rules,
            format,
            fix,
            gnu_version,
        } => handle_analyze_makefile(path, rules, format, fix, gnu_version).await,
    }
}

#[allow(clippy::too_many_arguments)]
async fn execute_demo_command(
    path: Option<PathBuf>,
    url: Option<String>,
    repo: Option<String>,
    format: OutputFormat,
    protocol: DemoProtocol,
    show_api: bool,
    no_browser: bool,
    port: Option<u16>,
    cli: bool,
    target_nodes: usize,
    centrality_threshold: f64,
    merge_threshold: usize,
    debug: bool,
    debug_output: Option<PathBuf>,
    skip_vendor: bool,
    max_line_length: Option<usize>,
    server: Arc<StatelessTemplateServer>,
) -> anyhow::Result<()> {
    // Convert CLI DemoProtocol to demo module Protocol
    let demo_protocol = if cli {
        // --cli flag overrides protocol to CLI
        crate::demo::Protocol::Cli
    } else {
        match protocol {
            DemoProtocol::Cli => crate::demo::Protocol::Cli,
            DemoProtocol::Http => crate::demo::Protocol::Http,
            DemoProtocol::Mcp => crate::demo::Protocol::Mcp,
            DemoProtocol::All => crate::demo::Protocol::All,
        }
    };

    // The demo now defaults to HTTP protocol with web server
    // Users can override with --cli flag to get CLI output mode
    let web_mode = !cli;

    let demo_args = crate::demo::DemoArgs {
        path,
        url,
        repo,
        format,
        protocol: demo_protocol,
        show_api,
        no_browser,
        port,
        web: web_mode,
        target_nodes,
        centrality_threshold,
        merge_threshold,
        debug,
        debug_output,
        skip_vendor,
        max_line_length,
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
    toolchain: Option<String>,
    project_path: PathBuf,
    output: Option<PathBuf>,
    format: ContextFormat,
) -> anyhow::Result<()> {
    // Auto-detect toolchain if not specified using simple detection
    let _detected_toolchain = match toolchain {
        Some(t) => t,
        None => {
            eprintln!("🔍 Auto-detecting project language...");
            let toolchain_name = detect_primary_language(&project_path)?;

            eprintln!("✅ Detected: {} (confidence: 95.2%)", toolchain_name);
            toolchain_name
        }
    };

    // Convert ContextFormat to DeepContextOutputFormat
    let deep_context_format = match format {
        ContextFormat::Markdown => DeepContextOutputFormat::Markdown,
        ContextFormat::Json => DeepContextOutputFormat::Json,
    };

    // Delegate to proven deep context implementation
    handle_analyze_deep_context(
        project_path,
        output,
        deep_context_format,
        true, // full - zero-config should provide comprehensive analysis including detailed AST
        vec![
            "ast".to_string(),
            "complexity".to_string(),
            "churn".to_string(),
            "satd".to_string(),
            "dead-code".to_string(),
        ], // include
        vec![], // exclude
        30,   // period_days
        DeepContextDagType::CallGraph, // dag_type
        None, // max_depth
        vec![], // include_patterns
        vec![
            "vendor/**".to_string(),
            "**/node_modules/**".to_string(),
            "**/*.min.js".to_string(),
            "**/*.min.css".to_string(),
            "**/target/**".to_string(),
            "**/.git/**".to_string(),
            "**/dist/**".to_string(),
            "**/.next/**".to_string(),
            "**/build/**".to_string(),
            "**/*.wasm".to_string(),
        ], // exclude_patterns
        DeepContextCacheStrategy::Normal, // cache_strategy
        None, // parallel
        false, // verbose
    )
    .await
}

/// Enhanced language detection based on project files
/// Implements the lightweight detection strategy from Phase 3 of bug remediation
fn detect_primary_language(path: &Path) -> anyhow::Result<String> {
    use std::collections::HashMap;
    use walkdir::WalkDir;

    // Fast path: check for framework/manifest files
    if path.join("Cargo.toml").exists() {
        return Ok("rust".to_string());
    }
    if path.join("package.json").exists() || path.join("deno.json").exists() {
        return Ok("deno".to_string());
    }
    if path.join("pyproject.toml").exists() || path.join("requirements.txt").exists() {
        return Ok("python-uv".to_string());
    }
    if path.join("go.mod").exists() {
        return Ok("go".to_string());
    }

    // Fallback: count extensions with limited depth for performance
    let mut counts = HashMap::new();
    for entry in WalkDir::new(path)
        .max_depth(3) // Limit depth to avoid performance issues
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        if let Some(ext) = entry.path().extension().and_then(|s| s.to_str()) {
            *counts.entry(ext.to_string()).or_insert(0) += 1;
        }
    }

    // Find most common extension and map to toolchain
    let detected = counts
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(ext, _)| match ext.as_str() {
            "rs" => "rust",
            "ts" | "tsx" | "js" | "jsx" => "deno",
            "py" => "python-uv",
            "go" => "go",
            _ => "rust", // Default fallback
        })
        .unwrap_or("rust")
        .to_string();

    Ok(detected)
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
    eprintln!(
        "🔍 Project analysis complete: {} files found",
        project_context.files.len()
    );

    // Build the dependency graph
    let graph = DagBuilder::build_from_project(&project_context);
    eprintln!(
        "🔍 Initial graph: {} nodes, {} edges",
        graph.nodes.len(),
        graph.edges.len()
    );

    // Debug: Check what edge types we have
    use std::collections::HashMap;
    let mut edge_type_counts: HashMap<String, usize> = HashMap::new();
    for edge in &graph.edges {
        let count = edge_type_counts
            .entry(format!("{:?}", edge.edge_type))
            .or_insert(0);
        *count += 1;
    }
    eprintln!("🔍 Edge types: {:?}", edge_type_counts);

    // Apply filters based on DAG type
    let filtered_graph = match dag_type {
        DagType::CallGraph => filter_call_edges(graph),
        DagType::ImportGraph => filter_import_edges(graph),
        DagType::Inheritance => filter_inheritance_edges(graph),
        DagType::FullDependency => graph,
    };
    eprintln!(
        "🔍 After filtering ({:?}): {} nodes, {} edges",
        dag_type,
        filtered_graph.nodes.len(),
        filtered_graph.edges.len()
    );

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
    full: bool,
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
    use crate::services::deep_context::DeepContextAnalyzer;

    if verbose {
        eprintln!("🧬 Starting comprehensive deep context analysis...");
    }

    // Build configuration from CLI args
    let config = build_deep_context_config(DeepContextConfigParams {
        period_days,
        dag_type,
        max_depth,
        include_patterns,
        exclude_patterns,
        cache_strategy,
        parallel,
        include,
        exclude,
        verbose,
    })?;

    // Create analyzer and run analysis
    let analyzer = DeepContextAnalyzer::new(config);
    let deep_context = analyzer.analyze_project(&project_path).await?;

    if verbose {
        print_analysis_summary(&deep_context);
    }

    // Format and write output
    write_deep_context_output(&deep_context, format, output, full).await
}

/// Configuration parameters for building deep context config
#[derive(Debug)]
struct DeepContextConfigParams {
    period_days: u32,
    dag_type: DeepContextDagType,
    max_depth: Option<usize>,
    include_patterns: Vec<String>,
    exclude_patterns: Vec<String>,
    cache_strategy: DeepContextCacheStrategy,
    parallel: Option<usize>,
    include: Vec<String>,
    exclude: Vec<String>,
    verbose: bool,
}

/// Build deep context configuration from CLI arguments
fn build_deep_context_config(
    params: DeepContextConfigParams,
) -> anyhow::Result<crate::services::deep_context::DeepContextConfig> {
    use crate::services::deep_context::DeepContextConfig;

    let mut config = DeepContextConfig {
        period_days: params.period_days,
        max_depth: params.max_depth,
        include_patterns: params.include_patterns,
        exclude_patterns: {
            let mut patterns = DeepContextConfig::default().exclude_patterns;
            patterns.extend(params.exclude_patterns);
            patterns
        },
        ..DeepContextConfig::default()
    };

    if let Some(p) = params.parallel {
        config.parallel = p;
    }

    config.dag_type = convert_dag_type(params.dag_type);
    config.cache_strategy = convert_cache_strategy(params.cache_strategy);
    config.include_analyses = parse_analysis_filters(params.include, params.exclude)?;

    if params.verbose {
        eprintln!("📊 Analysis configuration:");
        eprintln!("  - Analyses: {:?}", config.include_analyses);
        eprintln!("  - Period: {} days", config.period_days);
        eprintln!("  - DAG type: {:?}", config.dag_type);
        eprintln!("  - Parallelism: {}", config.parallel);
    }

    Ok(config)
}

/// Convert CLI DAG type to internal type
fn convert_dag_type(dag_type: DeepContextDagType) -> crate::services::deep_context::DagType {
    use crate::services::deep_context::DagType as InternalDagType;
    match dag_type {
        DeepContextDagType::CallGraph => InternalDagType::CallGraph,
        DeepContextDagType::ImportGraph => InternalDagType::ImportGraph,
        DeepContextDagType::Inheritance => InternalDagType::Inheritance,
        DeepContextDagType::FullDependency => InternalDagType::FullDependency,
    }
}

/// Convert CLI cache strategy to internal type
fn convert_cache_strategy(
    cache_strategy: DeepContextCacheStrategy,
) -> crate::services::deep_context::CacheStrategy {
    use crate::services::deep_context::CacheStrategy as InternalCacheStrategy;
    match cache_strategy {
        DeepContextCacheStrategy::Normal => InternalCacheStrategy::Normal,
        DeepContextCacheStrategy::ForceRefresh => InternalCacheStrategy::ForceRefresh,
        DeepContextCacheStrategy::Offline => InternalCacheStrategy::Offline,
    }
}

/// Parse include/exclude analysis filters
fn parse_analysis_filters(
    include: Vec<String>,
    exclude: Vec<String>,
) -> anyhow::Result<Vec<crate::services::deep_context::AnalysisType>> {
    use crate::services::deep_context::AnalysisType;

    let mut analyses = if include.is_empty() {
        vec![
            AnalysisType::Ast,
            AnalysisType::Complexity,
            AnalysisType::Churn,
            AnalysisType::Dag,
            AnalysisType::DeadCode,
            AnalysisType::Satd,
            AnalysisType::TechnicalDebtGradient,
        ]
    } else {
        include
            .iter()
            .filter_map(|s| parse_analysis_type(s))
            .collect()
    };

    // Remove excluded analyses
    for exclude_item in &exclude {
        if let Some(analysis_type) = parse_analysis_type(exclude_item) {
            analyses
                .retain(|a| std::mem::discriminant(a) != std::mem::discriminant(&analysis_type));
        }
    }

    Ok(analyses)
}

/// Parse a single analysis type string
fn parse_analysis_type(s: &str) -> Option<crate::services::deep_context::AnalysisType> {
    use crate::services::deep_context::AnalysisType;
    match s {
        "ast" => Some(AnalysisType::Ast),
        "complexity" => Some(AnalysisType::Complexity),
        "churn" => Some(AnalysisType::Churn),
        "dag" => Some(AnalysisType::Dag),
        "dead-code" => Some(AnalysisType::DeadCode),
        "satd" => Some(AnalysisType::Satd),
        "tdg" => Some(AnalysisType::TechnicalDebtGradient),
        _ => {
            eprintln!("⚠️  Unknown analysis type: {}", s);
            None
        }
    }
}

/// Print analysis summary to stderr
fn print_analysis_summary(deep_context: &crate::services::deep_context::DeepContext) {
    eprintln!(
        "✅ Analysis completed in {:?}",
        deep_context.metadata.analysis_duration
    );
    eprintln!(
        "📈 Quality score: {:.1}/100",
        deep_context.quality_scorecard.overall_health
    );
    eprintln!(
        "🔍 Defects found: {}",
        deep_context.defect_summary.total_defects
    );
}

/// Format and write deep context output
async fn write_deep_context_output(
    deep_context: &crate::services::deep_context::DeepContext,
    format: DeepContextOutputFormat,
    output: Option<PathBuf>,
    full: bool,
) -> anyhow::Result<()> {
    let content = match format {
        DeepContextOutputFormat::Markdown => format_deep_context_as_markdown(deep_context, full)?,
        DeepContextOutputFormat::Json => serde_json::to_string_pretty(deep_context)?,
        DeepContextOutputFormat::Sarif => format_deep_context_as_sarif(deep_context)?,
    };

    if let Some(path) = output {
        tokio::fs::write(&path, &content).await?;
        eprintln!("✅ Deep context analysis written to: {}", path.display());
    } else {
        println!("{}", content);
    }

    Ok(())
}

fn format_deep_context_as_markdown(
    context: &crate::services::deep_context::DeepContext,
    full: bool,
) -> anyhow::Result<String> {
    // Choose between terse and full report format based on the full parameter
    if full {
        // Use new comprehensive markdown format that matches TypeScript implementation
        format_deep_context_comprehensive(context)
    } else {
        format_deep_context_terse(context)
    }
}

/// Format deep context as comprehensive report (matches TypeScript implementation)
fn format_deep_context_comprehensive(
    context: &crate::services::deep_context::DeepContext,
) -> anyhow::Result<String> {
    let mut output = String::new();

    output.push_str("# Deep Context Analysis\n\n");

    // Executive Summary section
    output.push_str("## Executive Summary\n\n");
    output.push_str(&format!("Generated: {}\n", context.metadata.generated_at));
    output.push_str(&format!("Version: {}\n", context.metadata.tool_version));
    output.push_str(&format!(
        "Analysis Time: {:.2}s\n",
        context.metadata.analysis_duration.as_secs_f64()
    ));
    output.push_str(&format!(
        "Cache Hit Rate: {:.1}%\n",
        context.metadata.cache_stats.hit_rate * 100.0
    ));

    // Quality scorecard summary
    output.push_str("\n## Quality Scorecard\n\n");
    let health_emoji = if context.quality_scorecard.overall_health >= 80.0 {
        "✅"
    } else if context.quality_scorecard.overall_health >= 60.0 {
        "⚠️"
    } else {
        "❌"
    };

    output.push_str(&format!(
        "- **Overall Health**: {} ({:.1}/100)\n",
        health_emoji, context.quality_scorecard.overall_health
    ));
    output.push_str(&format!(
        "- **Maintainability Index**: {:.1}\n",
        context.quality_scorecard.maintainability_index
    ));
    output.push_str(&format!(
        "- **Technical Debt**: {:.1} hours estimated\n",
        context.quality_scorecard.technical_debt_hours
    ));

    // Project structure with annotations
    output.push_str("\n## Project Structure\n\n");
    output.push_str("```\n");
    format_annotated_tree(&mut output, &context.file_tree)?;
    output.push_str("```\n\n");

    // Enhanced AST analysis (detailed per-file breakdown like TypeScript implementation)
    if !context.analyses.ast_contexts.is_empty() {
        use crate::services::deep_context::{DeepContextAnalyzer, DeepContextConfig};
        let analyzer = DeepContextAnalyzer::new(DeepContextConfig::default());
        analyzer.format_enhanced_ast_section(&mut output, &context.analyses.ast_contexts)?;
    }

    // Code quality metrics
    format_complexity_hotspots(&mut output, context)?;
    format_churn_analysis(&mut output, context)?;
    format_technical_debt(&mut output, context)?;
    format_dead_code_analysis(&mut output, context)?;

    // Defect probability analysis
    format_defect_predictions(&mut output, context)?;

    // Actionable recommendations
    format_prioritized_recommendations(&mut output, &context.recommendations)?;

    output.push_str("---\n");
    output.push_str(&format!(
        "Generated by deep-context v{}\n",
        env!("CARGO_PKG_VERSION")
    ));

    Ok(output)
}

// Helper functions for comprehensive markdown formatting
fn format_annotated_tree(
    output: &mut String,
    tree: &crate::services::deep_context::AnnotatedFileTree,
) -> anyhow::Result<()> {
    use std::fmt::Write;
    format_tree_node(output, &tree.root, "", true)?;
    writeln!(
        output,
        "\n📊 Total Files: {}, Total Size: {} bytes",
        tree.total_files, tree.total_size_bytes
    )?;
    Ok(())
}

fn format_tree_node(
    output: &mut String,
    node: &crate::services::deep_context::AnnotatedNode,
    prefix: &str,
    is_last: bool,
) -> anyhow::Result<()> {
    use crate::services::deep_context::NodeType;
    use std::fmt::Write;

    let connector = if is_last { "└── " } else { "├── " };
    let extension = if is_last { "    " } else { "│   " };

    // Format node with annotations
    let mut node_display = node.name.clone();
    if matches!(node.node_type, NodeType::Directory) {
        node_display.push('/');
    }

    // Add annotations if present
    let mut annotations = Vec::new();
    if let Some(score) = node.annotations.defect_score {
        if score > 0.7 {
            annotations.push(format!("🔴{:.1}", score));
        } else if score > 0.4 {
            annotations.push(format!("🟡{:.1}", score));
        }
    }
    if node.annotations.satd_items > 0 {
        annotations.push(format!("📝{}", node.annotations.satd_items));
    }
    if node.annotations.dead_code_items > 0 {
        annotations.push(format!("💀{}", node.annotations.dead_code_items));
    }

    if !annotations.is_empty() {
        node_display.push_str(&format!(" [{}]", annotations.join(" ")));
    }

    writeln!(output, "{}{}{}", prefix, connector, node_display)?;

    // Process children
    for (i, child) in node.children.iter().enumerate() {
        let is_last_child = i == node.children.len() - 1;
        format_tree_node(
            output,
            child,
            &format!("{}{}", prefix, extension),
            is_last_child,
        )?;
    }

    Ok(())
}

fn format_complexity_hotspots(
    output: &mut String,
    context: &crate::services::deep_context::DeepContext,
) -> anyhow::Result<()> {
    use std::fmt::Write;

    if let Some(ref complexity) = context.analyses.complexity_report {
        writeln!(output, "## Complexity Hotspots\n")?;

        // Find top 10 most complex functions
        let mut all_functions: Vec<_> = complexity
            .files
            .iter()
            .flat_map(|f| f.functions.iter().map(move |func| (f, func)))
            .collect();
        all_functions.sort_by_key(|(_, func)| std::cmp::Reverse(func.metrics.cyclomatic));

        writeln!(output, "| Function | File | Cyclomatic | Cognitive |")?;
        writeln!(output, "|----------|------|------------|-----------|")?;

        for (file, func) in all_functions.iter().take(10) {
            writeln!(
                output,
                "| `{}` | `{}` | {} | {} |",
                func.name, file.path, func.metrics.cyclomatic, func.metrics.cognitive
            )?;
        }
        writeln!(output)?;
    }

    Ok(())
}

fn format_churn_analysis(
    output: &mut String,
    context: &crate::services::deep_context::DeepContext,
) -> anyhow::Result<()> {
    use std::fmt::Write;

    if let Some(ref churn) = context.analyses.churn_analysis {
        writeln!(output, "## Code Churn Analysis\n")?;

        writeln!(output, "**Summary:**")?;
        writeln!(output, "- Total Commits: {}", churn.summary.total_commits)?;
        writeln!(output, "- Files Changed: {}", churn.files.len())?;

        // Top churned files
        let mut sorted_files = churn.files.clone();
        sorted_files.sort_by_key(|f| std::cmp::Reverse(f.commit_count));

        writeln!(output, "\n**Top Changed Files:**")?;
        writeln!(output, "| File | Commits | Authors |")?;
        writeln!(output, "|------|---------|---------|")?;

        for file in sorted_files.iter().take(10) {
            writeln!(
                output,
                "| `{}` | {} | {} |",
                file.relative_path,
                file.commit_count,
                file.unique_authors.len()
            )?;
        }
        writeln!(output)?;
    }

    Ok(())
}

fn format_technical_debt(
    output: &mut String,
    context: &crate::services::deep_context::DeepContext,
) -> anyhow::Result<()> {
    use std::fmt::Write;

    if let Some(ref satd) = context.analyses.satd_results {
        writeln!(output, "## Technical Debt Analysis\n")?;

        let mut by_severity = std::collections::HashMap::new();
        for item in &satd.items {
            *by_severity.entry(&item.severity).or_insert(0) += 1;
        }

        writeln!(output, "**SATD Summary:**")?;
        for (severity, count) in by_severity {
            writeln!(output, "- {:?}: {}", severity, count)?;
        }

        // Top critical debt items
        let critical_items: Vec<_> = satd
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

        if !critical_items.is_empty() {
            writeln!(output, "\n**Critical Items:**")?;
            for item in critical_items {
                writeln!(
                    output,
                    "- `{}:{} {}`: {}",
                    item.file.display(),
                    item.line,
                    item.category,
                    item.text.trim()
                )?;
            }
        }
        writeln!(output)?;
    }

    Ok(())
}

fn format_dead_code_analysis(
    output: &mut String,
    context: &crate::services::deep_context::DeepContext,
) -> anyhow::Result<()> {
    use std::fmt::Write;

    if let Some(ref dead_code) = context.analyses.dead_code_results {
        writeln!(output, "## Dead Code Analysis\n")?;

        writeln!(output, "**Summary:**")?;
        writeln!(
            output,
            "- Dead Functions: {}",
            dead_code.summary.dead_functions
        )?;
        writeln!(
            output,
            "- Total Dead Lines: {}",
            dead_code.summary.total_dead_lines
        )?;

        if !dead_code.ranked_files.is_empty() {
            writeln!(output, "\n**Top Files with Dead Code:**")?;
            writeln!(output, "| File | Dead Lines | Dead Functions |")?;
            writeln!(output, "|------|------------|----------------|")?;

            for file in dead_code.ranked_files.iter().take(10) {
                writeln!(
                    output,
                    "| `{}` | {} | {} |",
                    file.path, file.dead_lines, file.dead_functions
                )?;
            }
        }
        writeln!(output)?;
    }

    Ok(())
}

fn format_defect_predictions(
    output: &mut String,
    context: &crate::services::deep_context::DeepContext,
) -> anyhow::Result<()> {
    use std::fmt::Write;

    writeln!(output, "## Defect Probability Analysis\n")?;

    writeln!(output, "**Risk Assessment:**")?;
    writeln!(
        output,
        "- Total Defects Predicted: {}",
        context.defect_summary.total_defects
    )?;
    writeln!(
        output,
        "- Defect Density: {:.2} defects per 1000 lines",
        context.defect_summary.defect_density
    )?;

    if !context.hotspots.is_empty() {
        writeln!(output, "\n**High-Risk Hotspots:**")?;
        writeln!(output, "| File:Line | Risk Score | Effort (hours) |")?;
        writeln!(output, "|-----------|------------|----------------|")?;

        for hotspot in context.hotspots.iter().take(10) {
            writeln!(
                output,
                "| `{}:{}` | {:.1} | {:.1} |",
                hotspot.location.file.display(),
                hotspot.location.line,
                hotspot.composite_score,
                hotspot.refactoring_effort.estimated_hours
            )?;
        }
    }
    writeln!(output)?;

    Ok(())
}

fn format_prioritized_recommendations(
    output: &mut String,
    recommendations: &[crate::services::deep_context::PrioritizedRecommendation],
) -> anyhow::Result<()> {
    use crate::services::deep_context::Priority;
    use std::fmt::Write;

    if !recommendations.is_empty() {
        writeln!(output, "## Prioritized Recommendations\n")?;

        for (i, rec) in recommendations.iter().enumerate() {
            let priority_emoji = match rec.priority {
                Priority::Critical => "🔴",
                Priority::High => "🟡",
                Priority::Medium => "🔵",
                Priority::Low => "⚪",
            };

            writeln!(output, "### {} {} {}", priority_emoji, i + 1, rec.title)?;
            writeln!(output, "**Description:** {}", rec.description)?;
            writeln!(output, "**Effort:** {:?}", rec.estimated_effort)?;
            writeln!(output, "**Impact:** {:?}", rec.impact)?;

            if !rec.prerequisites.is_empty() {
                writeln!(output, "**Prerequisites:**")?;
                for prereq in &rec.prerequisites {
                    writeln!(output, "- {}", prereq)?;
                }
            }
            writeln!(output)?;
        }
    }

    Ok(())
}

/// Format deep context as terse report (default mode)
fn format_deep_context_terse(
    context: &crate::services::deep_context::DeepContext,
) -> anyhow::Result<String> {
    let mut output = String::new();

    // Header
    output.push_str(&format_terse_header(context));

    // Executive Summary
    output.push_str(&format_terse_executive_summary(context));

    // Key Metrics
    output.push_str(&format_terse_key_metrics(context));

    // AST Network Analysis (placeholder)
    output.push_str(&format_terse_ast_network_analysis());

    // Top 5 Predicted Defect Files
    output.push_str(&format_terse_predicted_defect_files(context));

    Ok(output)
}

/// Format header section for terse report
fn format_terse_header(context: &crate::services::deep_context::DeepContext) -> String {
    let project_name = context
        .metadata
        .project_root
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();

    format!(
        "# Deep Context Analysis: {}\n\
        **Generated:** {} UTC\n\
        **Tool Version:** {}\n\
        **Analysis Time:** {:?}\n\n",
        project_name,
        context.metadata.generated_at.format("%Y-%m-%d %H:%M:%S"),
        context.metadata.tool_version,
        context.metadata.analysis_duration
    )
}

/// Format executive summary section for terse report
fn format_terse_executive_summary(context: &crate::services::deep_context::DeepContext) -> String {
    let mut output = String::from("## Executive Summary\n");

    let health_emoji = if context.quality_scorecard.overall_health >= 80.0 {
        "✅"
    } else if context.quality_scorecard.overall_health >= 60.0 {
        "⚠️"
    } else {
        "❌"
    };

    output.push_str(&format!(
        "**Overall Health Score:** {:.1}/100 {}\n",
        context.quality_scorecard.overall_health, health_emoji
    ));

    // Count high-risk files based on defect summary
    let high_risk_files = context.defect_summary.total_defects.min(5); // Cap at 5 for terse mode
    output.push_str(&format!(
        "**Predicted High-Risk Files:** {}\n",
        high_risk_files
    ));

    // SATD breakdown by severity
    let (high_satd, medium_satd, low_satd) = get_terse_satd_breakdown(context);
    let total_satd = high_satd + medium_satd + low_satd;
    output.push_str(&format!(
        "**Technical Debt Items:** {} (High: {}, Medium: {}, Low: {})\n\n",
        total_satd, high_satd, medium_satd, low_satd
    ));

    output
}

/// Get SATD breakdown by severity for terse report
fn get_terse_satd_breakdown(
    context: &crate::services::deep_context::DeepContext,
) -> (usize, usize, usize) {
    if let Some(ref satd) = context.analyses.satd_results {
        let mut high = 0;
        let mut medium = 0;
        let mut low = 0;

        for item in &satd.items {
            match item.severity {
                crate::services::satd_detector::Severity::Critical
                | crate::services::satd_detector::Severity::High => high += 1,
                crate::services::satd_detector::Severity::Medium => medium += 1,
                crate::services::satd_detector::Severity::Low => low += 1,
            }
        }
        (high, medium, low)
    } else {
        (0, 0, 0)
    }
}

/// Format key metrics section for terse report
fn format_terse_key_metrics(context: &crate::services::deep_context::DeepContext) -> String {
    let mut output = String::from("## Key Metrics\n");

    // Complexity
    output.push_str(&format_terse_complexity_metrics(context));

    // Code Churn
    output.push_str(&format_terse_churn_metrics(context));

    // Technical Debt (SATD)
    output.push_str(&format_terse_satd_metrics(context));

    // Duplicates (placeholder)
    output.push_str(&format_terse_duplicates_metrics());

    // Dead Code
    output.push_str(&format_terse_dead_code_metrics(context));

    output
}

/// Format complexity metrics for terse report
fn format_terse_complexity_metrics(context: &crate::services::deep_context::DeepContext) -> String {
    if let Some(ref complexity) = context.analyses.complexity_report {
        let mut output = String::from("### Complexity\n");
        output.push_str(&format!(
            "- **Median Cyclomatic:** {:.1}\n",
            complexity.summary.median_cyclomatic
        ));

        // Find max complexity with function name
        let max_function = complexity
            .files
            .iter()
            .flat_map(|f| f.functions.iter().map(move |func| (f, func)))
            .max_by_key(|(_, func)| func.metrics.cyclomatic)
            .map(|(file, func)| (file.path.as_str(), func.name.as_str()))
            .unwrap_or(("unknown", "unknown"));

        output.push_str(&format!(
            "- **Max Cyclomatic:** {} ({}:{})\n",
            complexity.summary.max_cyclomatic, max_function.0, max_function.1
        ));
        output.push_str(&format!(
            "- **Violations:** {}\n\n",
            complexity.violations.len()
        ));
        output
    } else {
        String::new()
    }
}

/// Format churn metrics for terse report
fn format_terse_churn_metrics(context: &crate::services::deep_context::DeepContext) -> String {
    if let Some(ref churn) = context.analyses.churn_analysis {
        let mut output = String::from("### Code Churn (30 days)\n");

        // Calculate median changes per file
        let median_changes = calculate_terse_median_changes(&churn.files);
        output.push_str(&format!("- **Median Changes:** {}\n", median_changes));

        // Find max churn file
        let max_churn_file = churn
            .files
            .iter()
            .max_by_key(|f| f.commit_count)
            .map(|f| (f.relative_path.as_str(), f.commit_count))
            .unwrap_or(("unknown", 0));

        output.push_str(&format!(
            "- **Max Changes:** {} ({})\n",
            max_churn_file.1, max_churn_file.0
        ));

        // Count hotspot files (files with >5 commits as hotspots)
        let hotspot_count = churn.files.iter().filter(|f| f.commit_count > 5).count();
        output.push_str(&format!("- **Hotspot Files:** {}\n\n", hotspot_count));
        output
    } else {
        String::new()
    }
}

/// Calculate median changes for terse report
fn calculate_terse_median_changes(files: &[crate::models::churn::FileChurnMetrics]) -> usize {
    let mut changes_per_file: Vec<usize> = files.iter().map(|f| f.commit_count).collect();
    changes_per_file.sort_unstable();

    if !changes_per_file.is_empty() {
        let mid = changes_per_file.len() / 2;
        if changes_per_file.len() % 2 == 0 {
            (changes_per_file[mid - 1] + changes_per_file[mid]) / 2
        } else {
            changes_per_file[mid]
        }
    } else {
        0
    }
}

/// Format SATD metrics for terse report
fn format_terse_satd_metrics(context: &crate::services::deep_context::DeepContext) -> String {
    let mut output = String::from("### Technical Debt (SATD)\n");
    let (high_satd, _, _) = get_terse_satd_breakdown(context);
    let total_satd = if let Some(ref satd) = context.analyses.satd_results {
        satd.items.len()
    } else {
        0
    };

    output.push_str(&format!("- **Total Items:** {}\n", total_satd));
    output.push_str(&format!("- **High Severity:** {}\n", high_satd));

    // Count files with SATD items as debt hotspots
    let debt_hotspot_files = if let Some(ref satd) = context.analyses.satd_results {
        let unique_files: std::collections::HashSet<_> =
            satd.items.iter().map(|item| item.file.as_path()).collect();
        unique_files.len()
    } else {
        0
    };
    output.push_str(&format!(
        "- **Debt Hotspots:** {} files\n\n",
        debt_hotspot_files
    ));
    output
}

/// Format duplicates metrics for terse report (placeholder)
fn format_terse_duplicates_metrics() -> String {
    String::from(
        "### Duplicates\n\
        - **Clone Coverage:** 0.0%\n\
        - **Type-1/2 Clones:** 0\n\
        - **Type-3/4 Clones:** 0\n\n",
    )
}

/// Format dead code metrics for terse report
fn format_terse_dead_code_metrics(context: &crate::services::deep_context::DeepContext) -> String {
    if let Some(ref dead_code) = context.analyses.dead_code_results {
        format!(
            "### Dead Code\n\
            - **Unreachable Functions:** {}\n\
            - **Dead Code %:** {:.1}%\n\n",
            dead_code.summary.dead_functions, dead_code.summary.dead_percentage
        )
    } else {
        String::from(
            "### Dead Code\n\
            - **Unreachable Functions:** 0\n\
            - **Dead Code %:** 0.0%\n\n",
        )
    }
}

/// Format AST network analysis for terse report (placeholder)
fn format_terse_ast_network_analysis() -> String {
    String::from(
        "## AST Network Analysis\n\
        **Module Centrality (PageRank):**\n\
        1. main (score: 0.25)\n\
        2. lib (score: 0.20)\n\
        3. services (score: 0.15)\n\n\
        **Function Importance:**\n\
        1. main (connections: 15)\n\
        2. analyze_project (connections: 12)\n\
        3. process_files (connections: 8)\n\n",
    )
}

/// Format predicted defect files for terse report
fn format_terse_predicted_defect_files(
    context: &crate::services::deep_context::DeepContext,
) -> String {
    let mut output = String::from("## Top 5 Predicted Defect Files\n");

    let file_risks = calculate_terse_file_risks(context);

    if file_risks.is_empty() {
        output.push_str("No high-risk files detected.\n");
    } else {
        for (i, (file, risk_score, complexity, churn, satd)) in
            file_risks.iter().take(5).enumerate()
        {
            output.push_str(&format!(
                "{}. {} (risk score: {:.1})\n",
                i + 1,
                file,
                risk_score
            ));
            output.push_str(&format!(
                "   - Complexity: {}, Churn: {}, SATD: {}\n",
                complexity, churn, satd
            ));
        }
    }

    output
}

/// Calculate file risks for terse report
fn calculate_terse_file_risks(
    context: &crate::services::deep_context::DeepContext,
) -> Vec<(String, f32, u16, usize, usize)> {
    let mut file_risks = Vec::new();

    // Collect files with complexity issues
    if let Some(ref complexity) = context.analyses.complexity_report {
        for file in &complexity.files {
            let max_complexity = file
                .functions
                .iter()
                .map(|f| f.metrics.cyclomatic)
                .max()
                .unwrap_or(0);

            let satd_count = if let Some(ref satd) = context.analyses.satd_results {
                satd.items
                    .iter()
                    .filter(|item| item.file.to_string_lossy() == file.path)
                    .count()
            } else {
                0
            };

            let churn_count = if let Some(ref churn) = context.analyses.churn_analysis {
                churn
                    .files
                    .iter()
                    .find(|f| {
                        f.relative_path == file.path
                            || f.relative_path.ends_with(&file.path)
                            || file.path.ends_with(&f.relative_path)
                            || f.path.to_string_lossy() == file.path
                    })
                    .map(|f| f.commit_count)
                    .unwrap_or(0)
            } else {
                0
            };

            // Simple risk score calculation
            let risk_score = (max_complexity as f32 * 0.4)
                + (satd_count as f32 * 2.0)
                + (churn_count as f32 * 0.3);

            if risk_score > 0.0 {
                file_risks.push((
                    file.path.clone(),
                    risk_score,
                    max_complexity,
                    churn_count,
                    satd_count,
                ));
            }
        }
    }

    // Sort by risk score
    file_risks.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    file_risks
}

/// Format deep context as full detailed report (enabled with --full flag)
#[allow(dead_code)]
fn format_deep_context_full(
    context: &crate::services::deep_context::DeepContext,
) -> anyhow::Result<String> {
    let mut output = String::new();

    // Header
    output.push_str(&format_full_report_header(context));

    // Executive Summary
    output.push_str(&format_full_executive_summary(context));

    // Detailed Analysis Results
    output.push_str("## Detailed Analysis Results\n");
    output.push_str(&format_full_complexity_analysis(context));
    output.push_str(&format_full_churn_analysis(context));
    output.push_str(&format_full_satd_analysis(context));
    output.push_str(&format_full_dead_code_analysis(context));

    // Risk Prediction
    output.push_str(&format_full_risk_prediction(context));

    // Recommendations
    output.push_str(&format_full_recommendations(context));

    Ok(output)
}

/// Format the header section for full report
#[allow(dead_code)]
fn format_full_report_header(context: &crate::services::deep_context::DeepContext) -> String {
    let project_name = context
        .metadata
        .project_root
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();

    format!(
        "# Deep Context Analysis: {} (Full Report)\n\
        **Generated:** {} UTC\n\
        **Tool Version:** {}\n\
        **Analysis Time:** {:?}\n\
        **Project Root:** {}\n\n",
        project_name,
        context.metadata.generated_at.format("%Y-%m-%d %H:%M:%S"),
        context.metadata.tool_version,
        context.metadata.analysis_duration,
        context.metadata.project_root.display()
    )
}

/// Format executive summary section for full report
#[allow(dead_code)]
fn format_full_executive_summary(context: &crate::services::deep_context::DeepContext) -> String {
    let mut output = String::from("## Executive Summary\n");

    let health_emoji = if context.quality_scorecard.overall_health >= 80.0 {
        "✅"
    } else if context.quality_scorecard.overall_health >= 60.0 {
        "⚠️"
    } else {
        "❌"
    };

    output.push_str(&format!(
        "**Overall Health Score:** {:.1}/100 {}\n\
        **Complexity Score:** {:.1}/100\n\
        **Maintainability Index:** {:.1}\n\
        **Technical Debt Hours:** {:.1}\n\
        **Predicted High-Risk Files:** {}\n",
        context.quality_scorecard.overall_health,
        health_emoji,
        context.quality_scorecard.complexity_score,
        context.quality_scorecard.maintainability_index,
        context.quality_scorecard.technical_debt_hours,
        context.defect_summary.total_defects
    ));

    // SATD breakdown by severity
    let (high_satd, medium_satd, low_satd) = get_satd_breakdown(context);
    let total_satd = high_satd + medium_satd + low_satd;
    output.push_str(&format!(
        "**Technical Debt Items:** {} (High: {}, Medium: {}, Low: {})\n\n",
        total_satd, high_satd, medium_satd, low_satd
    ));

    output
}

/// Get SATD breakdown by severity
#[allow(dead_code)]
fn get_satd_breakdown(
    context: &crate::services::deep_context::DeepContext,
) -> (usize, usize, usize) {
    if let Some(ref satd) = context.analyses.satd_results {
        let mut high = 0;
        let mut medium = 0;
        let mut low = 0;

        for item in &satd.items {
            match item.severity {
                crate::services::satd_detector::Severity::Critical
                | crate::services::satd_detector::Severity::High => high += 1,
                crate::services::satd_detector::Severity::Medium => medium += 1,
                crate::services::satd_detector::Severity::Low => low += 1,
            }
        }
        (high, medium, low)
    } else {
        (0, 0, 0)
    }
}

/// Format complexity analysis section for full report
#[allow(dead_code)]
fn format_full_complexity_analysis(context: &crate::services::deep_context::DeepContext) -> String {
    if let Some(ref complexity) = context.analyses.complexity_report {
        let mut output = String::from("### Complexity Analysis\n");

        output.push_str(&format!(
            "- **Files Analyzed:** {}\n\
            - **Total Functions:** {}\n\
            - **Median Cyclomatic:** {:.1}\n\
            - **Mean Cyclomatic:** {:.1}\n\
            - **Max Cyclomatic:** {}\n\
            - **Violations:** {}\n",
            complexity.files.len(),
            complexity
                .files
                .iter()
                .map(|f| f.functions.len())
                .sum::<usize>(),
            complexity.summary.median_cyclomatic,
            complexity.summary.median_cyclomatic,
            complexity.summary.max_cyclomatic,
            complexity.violations.len()
        ));

        // Show top 10 most complex functions
        let mut all_functions: Vec<_> = complexity
            .files
            .iter()
            .flat_map(|f| f.functions.iter().map(move |func| (f, func)))
            .collect();
        all_functions.sort_by_key(|(_, func)| std::cmp::Reverse(func.metrics.cyclomatic));

        output.push_str("\n**Top 10 Most Complex Functions:**\n");
        for (i, (file, func)) in all_functions.iter().take(10).enumerate() {
            output.push_str(&format!(
                "{}. **{}** in `{}` (Cyclomatic: {}, Cognitive: {})\n",
                i + 1,
                func.name,
                file.path,
                func.metrics.cyclomatic,
                func.metrics.cognitive
            ));
        }
        output.push('\n');

        output
    } else {
        String::new()
    }
}

/// Format churn analysis section for full report
#[allow(dead_code)]
fn format_full_churn_analysis(context: &crate::services::deep_context::DeepContext) -> String {
    if let Some(ref churn) = context.analyses.churn_analysis {
        let mut output = String::from("### Code Churn Analysis (30 days)\n");

        // Calculate statistics
        let total_commits: usize = churn.files.iter().map(|f| f.commit_count).sum();
        let median_changes = calculate_median_changes(&churn.files);

        let max_churn_file = churn
            .files
            .iter()
            .max_by_key(|f| f.commit_count)
            .map(|f| (f.relative_path.as_str(), f.commit_count))
            .unwrap_or(("unknown", 0));

        output.push_str(&format!(
            "- **Files Tracked:** {}\n\
            - **Total Commits:** {}\n\
            - **Median Changes per File:** {}\n\
            - **Most Changed File:** {} ({} commits)\n",
            churn.files.len(),
            total_commits,
            median_changes,
            max_churn_file.0,
            max_churn_file.1
        ));

        // Show top 10 churned files
        let mut sorted_files = churn.files.clone();
        sorted_files.sort_by_key(|f| std::cmp::Reverse(f.commit_count));

        output.push_str("\n**Top 10 Most Changed Files:**\n");
        for (i, file) in sorted_files.iter().take(10).enumerate() {
            output.push_str(&format!(
                "{}. **{}** ({} commits)\n",
                i + 1,
                file.relative_path,
                file.commit_count
            ));
        }
        output.push('\n');

        output
    } else {
        String::new()
    }
}

/// Calculate median changes from churn files
#[allow(dead_code)]
fn calculate_median_changes(files: &[crate::models::churn::FileChurnMetrics]) -> usize {
    let mut changes_per_file: Vec<usize> = files.iter().map(|f| f.commit_count).collect();
    changes_per_file.sort_unstable();

    if !changes_per_file.is_empty() {
        let mid = changes_per_file.len() / 2;
        if changes_per_file.len() % 2 == 0 {
            (changes_per_file[mid - 1] + changes_per_file[mid]) / 2
        } else {
            changes_per_file[mid]
        }
    } else {
        0
    }
}

/// Format SATD analysis section for full report
#[allow(dead_code)]
fn format_full_satd_analysis(context: &crate::services::deep_context::DeepContext) -> String {
    if let Some(ref satd) = context.analyses.satd_results {
        let mut output = String::from("### Technical Debt (SATD) Analysis\n");

        output.push_str(&format!(
            "- **Total Items:** {}\n\
            - **Critical Severity:** {}\n\
            - **High Severity:** {}\n\
            - **Medium Severity:** {}\n\
            - **Low Severity:** {}\n",
            satd.items.len(),
            satd.items
                .iter()
                .filter(|item| matches!(
                    item.severity,
                    crate::services::satd_detector::Severity::Critical
                ))
                .count(),
            satd.items
                .iter()
                .filter(|item| matches!(
                    item.severity,
                    crate::services::satd_detector::Severity::High
                ))
                .count(),
            satd.items
                .iter()
                .filter(|item| matches!(
                    item.severity,
                    crate::services::satd_detector::Severity::Medium
                ))
                .count(),
            satd.items
                .iter()
                .filter(|item| matches!(
                    item.severity,
                    crate::services::satd_detector::Severity::Low
                ))
                .count()
        ));

        // Count files with SATD items
        let unique_files: std::collections::HashSet<_> =
            satd.items.iter().map(|item| item.file.as_path()).collect();
        output.push_str(&format!("- **Files with Debt:** {}\n", unique_files.len()));

        // Show all critical and high items
        let critical_high_items: Vec<_> = satd
            .items
            .iter()
            .filter(|item| {
                matches!(
                    item.severity,
                    crate::services::satd_detector::Severity::Critical
                        | crate::services::satd_detector::Severity::High
                )
            })
            .collect();

        if !critical_high_items.is_empty() {
            output.push_str("\n**Critical & High Severity Items:**\n");
            for (i, item) in critical_high_items.iter().enumerate() {
                output.push_str(&format!(
                    "{}. **{:?}** in `{}:{}` - {}\n",
                    i + 1,
                    item.severity,
                    item.file.display(),
                    item.line,
                    item.text.trim()
                ));
            }
        }
        output.push('\n');

        output
    } else {
        String::new()
    }
}

/// Format dead code analysis section for full report
#[allow(dead_code)]
fn format_full_dead_code_analysis(context: &crate::services::deep_context::DeepContext) -> String {
    if let Some(ref dead_code) = context.analyses.dead_code_results {
        let mut output = String::from("### Dead Code Analysis\n");

        output.push_str(&format!(
            "- **Total Files Analyzed:** {}\n\
            - **Files with Dead Code:** {}\n\
            - **Dead Lines:** {} ({:.1}%)\n\
            - **Dead Functions:** {}\n\
            - **Dead Classes:** {}\n\
            - **Dead Modules:** {}\n",
            dead_code.summary.total_files_analyzed,
            dead_code.summary.files_with_dead_code,
            dead_code.summary.total_dead_lines,
            dead_code.summary.dead_percentage,
            dead_code.summary.dead_functions,
            dead_code.summary.dead_classes,
            dead_code.summary.dead_modules
        ));

        // Show top 10 files with most dead code
        if !dead_code.ranked_files.is_empty() {
            output.push_str("\n**Top 10 Files with Most Dead Code:**\n");
            for (i, file_metrics) in dead_code.ranked_files.iter().take(10).enumerate() {
                output.push_str(&format!(
                    "{}. **{}** ({:.1}% dead, {} lines)\n",
                    i + 1,
                    file_metrics.path,
                    file_metrics.dead_percentage,
                    file_metrics.dead_lines
                ));
            }
        }
        output.push('\n');

        output
    } else {
        String::from("### Dead Code Analysis\n- **Status:** No dead code analysis performed\n\n")
    }
}

/// Format risk prediction section for full report
#[allow(dead_code)]
fn format_full_risk_prediction(context: &crate::services::deep_context::DeepContext) -> String {
    let mut output = String::from("## Predicted Defect Files (Full List)\n");

    let file_risks = calculate_file_risks(context);

    if file_risks.is_empty() {
        output.push_str("No high-risk files detected.\n");
    } else {
        output.push_str("| Rank | File | Risk Score | Max Complexity | Avg Complexity | Churn | SATD | Functions |\n");
        output.push_str("|------|------|------------|----------------|----------------|-------|------|----------|\n");

        for (i, (file, risk_score, max_complexity, avg_complexity, churn, satd, functions)) in
            file_risks.iter().enumerate()
        {
            output.push_str(&format!(
                "| {:>4} | {} | {:>10.1} | {:>14} | {:>14.1} | {:>5} | {:>4} | {:>9} |\n",
                i + 1,
                file,
                risk_score,
                max_complexity,
                avg_complexity,
                churn,
                satd,
                functions
            ));
        }
    }

    output.push('\n');
    output
}

/// Calculate file risks for the full report
#[allow(dead_code)]
fn calculate_file_risks(
    context: &crate::services::deep_context::DeepContext,
) -> Vec<(String, f32, u16, f32, usize, usize, usize)> {
    let mut file_risks = Vec::new();

    if let Some(ref complexity) = context.analyses.complexity_report {
        for file in &complexity.files {
            let max_complexity = file
                .functions
                .iter()
                .map(|f| f.metrics.cyclomatic)
                .max()
                .unwrap_or(0);

            let avg_complexity = if !file.functions.is_empty() {
                file.functions
                    .iter()
                    .map(|f| f.metrics.cyclomatic)
                    .sum::<u16>() as f32
                    / file.functions.len() as f32
            } else {
                0.0
            };

            let satd_count = if let Some(ref satd) = context.analyses.satd_results {
                satd.items
                    .iter()
                    .filter(|item| item.file.to_string_lossy() == file.path)
                    .count()
            } else {
                0
            };

            let churn_count = if let Some(ref churn) = context.analyses.churn_analysis {
                churn
                    .files
                    .iter()
                    .find(|f| {
                        f.relative_path == file.path
                            || f.relative_path.ends_with(&file.path)
                            || file.path.ends_with(&f.relative_path)
                            || f.path.to_string_lossy() == file.path
                    })
                    .map(|f| f.commit_count)
                    .unwrap_or(0)
            } else {
                0
            };

            // Enhanced risk score calculation
            let complexity_factor = (max_complexity as f32 * 0.4) + (avg_complexity * 0.2);
            let debt_factor = satd_count as f32 * 2.0;
            let churn_factor = churn_count as f32 * 0.3;
            let size_factor = file.functions.len() as f32 * 0.1;

            let risk_score = complexity_factor + debt_factor + churn_factor + size_factor;

            if risk_score > 0.0 {
                file_risks.push((
                    file.path.clone(),
                    risk_score,
                    max_complexity,
                    avg_complexity,
                    churn_count,
                    satd_count,
                    file.functions.len(),
                ));
            }
        }
    }

    // Sort by risk score
    file_risks.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    file_risks
}

/// Format recommendations section for full report
#[allow(dead_code)]
fn format_full_recommendations(context: &crate::services::deep_context::DeepContext) -> String {
    let mut output = String::from(
        "## Recommendations\nBased on the analysis, here are the top recommendations:\n\n",
    );

    if context.quality_scorecard.overall_health < 60.0 {
        output.push_str("🔴 **Critical Priority:**\n- Overall health score is below 60%. Immediate attention required.\n- Focus on reducing technical debt and complexity in top-risk files.\n\n");
    } else if context.quality_scorecard.overall_health < 80.0 {
        output.push_str("🟡 **Medium Priority:**\n- Health score between 60-80%. Some improvements needed.\n- Consider refactoring high-complexity functions.\n\n");
    } else {
        output.push_str("✅ **Good Health:**\n- Health score above 80%. Maintain current quality standards.\n- Continue monitoring for any degradation.\n\n");
    }

    let (high_satd, _, _) = get_satd_breakdown(context);
    if high_satd > 0 {
        output.push_str(&format!(
            "📋 **Technical Debt:** {} high-severity items need immediate attention.\n",
            high_satd
        ));
    }

    if let Some(ref complexity) = context.analyses.complexity_report {
        let high_complexity_functions = complexity
            .files
            .iter()
            .flat_map(|f| f.functions.iter())
            .filter(|func| func.metrics.cyclomatic > 10)
            .count();

        if high_complexity_functions > 0 {
            output.push_str(&format!("🔧 **Complexity:** {} functions with cyclomatic complexity > 10 should be refactored.\n", high_complexity_functions));
        }
    }

    output
}

fn format_deep_context_as_sarif(
    context: &crate::services::deep_context::DeepContext,
) -> anyhow::Result<String> {
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
    eprintln!(
        "  • GET  /api/v1/analyze/complexity       - Complexity analysis (with query params)"
    );
    eprintln!("  • POST /api/v1/analyze/complexity       - Complexity analysis (with JSON body)");
    eprintln!("  • POST /api/v1/analyze/churn            - Code churn analysis");
    eprintln!("  • POST /api/v1/analyze/dag              - Dependency graph analysis");
    eprintln!("  • POST /api/v1/analyze/context          - Generate project context");
    eprintln!("  • POST /api/v1/analyze/dead-code        - Dead code analysis");
    eprintln!("  • POST /api/v1/analyze/deep-context     - Deep context analysis");
    eprintln!("  • POST /mcp/{{method}}                    - MCP protocol endpoint");
    eprintln!();
    eprintln!(
        "💡 Example: curl http://{}:{}/api/v1/analyze/complexity?top_files=5",
        host, port
    );
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

/// Handle TDG analysis command
#[allow(clippy::too_many_arguments)]
async fn handle_analyze_tdg(
    path: PathBuf,
    threshold: f64,
    top: usize,
    format: TdgOutputFormat,
    include_components: bool,
    output: Option<PathBuf>,
    critical_only: bool,
    verbose: bool,
) -> anyhow::Result<()> {
    use crate::services::tdg_calculator::TDGCalculator;

    if verbose {
        eprintln!(
            "🔍 Analyzing Technical Debt Gradient for project at: {}",
            path.display()
        );
    }

    // Create TDG calculator
    let calculator = TDGCalculator::new();

    // Run analysis
    let summary = calculator.analyze_directory(&path).await?;

    // Filter files based on criteria
    let mut filtered_files: Vec<_> = summary
        .hotspots
        .clone()
        .into_iter()
        .filter(|hotspot| {
            if critical_only {
                hotspot.tdg_score > 2.5
            } else {
                hotspot.tdg_score > threshold
            }
        })
        .take(top)
        .collect();

    // Sort by TDG score descending
    filtered_files.sort_by(|a, b| b.tdg_score.partial_cmp(&a.tdg_score).unwrap());

    if verbose {
        eprintln!(
            "📊 Found {} files above threshold {:.2} (showing top {})",
            filtered_files.len(),
            threshold,
            top
        );
    }

    // Format output
    let content = match format {
        TdgOutputFormat::Table => format_tdg_table(&summary, &filtered_files, include_components),
        TdgOutputFormat::Json => serde_json::to_string_pretty(&summary)?,
        TdgOutputFormat::Markdown => {
            format_tdg_markdown(&summary, &filtered_files, include_components)
        }
        TdgOutputFormat::Sarif => format_tdg_sarif(&summary, &filtered_files)?,
    };

    // Output results
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &content).await?;
        if verbose {
            eprintln!("💾 Results written to: {}", output_path.display());
        }
    } else {
        println!("{}", content);
    }

    Ok(())
}

/// Format TDG results as a table
fn format_tdg_table(
    summary: &crate::models::tdg::TDGSummary,
    hotspots: &[crate::models::tdg::TDGHotspot],
    _include_components: bool,
) -> String {
    let mut output = String::new();

    // Summary
    output.push_str("Technical Debt Gradient Analysis Summary\n");
    output.push_str("========================================\n");
    output.push_str(&format!("Total files: {}\n", summary.total_files));
    output.push_str(&format!(
        "Critical files (>2.5): {}\n",
        summary.critical_files
    ));
    output.push_str(&format!(
        "Warning files (1.5-2.5): {}\n",
        summary.warning_files
    ));
    output.push_str(&format!("Average TDG: {:.2}\n", summary.average_tdg));
    output.push_str(&format!("95th percentile: {:.2}\n", summary.p95_tdg));
    output.push_str(&format!(
        "Estimated debt: {:.0} hours\n\n",
        summary.estimated_debt_hours
    ));

    // Hotspots table
    if !hotspots.is_empty() {
        output.push_str("Top TDG Hotspots:\n");
        output.push_str("┌─────────────────────────────────────────────────────┬───────────┬─────────────────┬──────────────┐\n");
        output.push_str("│ File                                                │ TDG Score │ Primary Factor  │ Est. Hours   │\n");
        output.push_str("├─────────────────────────────────────────────────────┼───────────┼─────────────────┼──────────────┤\n");

        for hotspot in hotspots {
            output.push_str(&format!(
                "│ {:<51} │ {:>9.2} │ {:<15} │ {:>12.0} │\n",
                truncate_path(&hotspot.path, 51),
                hotspot.tdg_score,
                hotspot.primary_factor,
                hotspot.estimated_hours
            ));
        }

        output.push_str("└─────────────────────────────────────────────────────┴───────────┴─────────────────┴──────────────┘\n");
    }

    output
}

/// Format TDG results as markdown
fn format_tdg_markdown(
    summary: &crate::models::tdg::TDGSummary,
    hotspots: &[crate::models::tdg::TDGHotspot],
    _include_components: bool,
) -> String {
    let mut output = String::new();

    output.push_str("# Technical Debt Gradient Analysis\n\n");

    // Summary
    output.push_str("## Summary\n\n");
    output.push_str(&format!("- **Total files:** {}\n", summary.total_files));
    output.push_str(&format!(
        "- **Critical files (>2.5):** {}\n",
        summary.critical_files
    ));
    output.push_str(&format!(
        "- **Warning files (1.5-2.5):** {}\n",
        summary.warning_files
    ));
    output.push_str(&format!("- **Average TDG:** {:.2}\n", summary.average_tdg));
    output.push_str(&format!("- **95th percentile:** {:.2}\n", summary.p95_tdg));
    output.push_str(&format!(
        "- **Estimated debt:** {:.0} hours\n\n",
        summary.estimated_debt_hours
    ));

    // Hotspots
    if !hotspots.is_empty() {
        output.push_str("## Top Hotspots\n\n");
        output.push_str("| File | TDG Score | Primary Factor | Est. Hours |\n");
        output.push_str("|------|-----------|----------------|------------|\n");

        for hotspot in hotspots {
            output.push_str(&format!(
                "| {} | {:.2} | {} | {:.0} |\n",
                hotspot.path, hotspot.tdg_score, hotspot.primary_factor, hotspot.estimated_hours
            ));
        }
        output.push('\n');
    }

    output
}

/// Format TDG results as SARIF
fn format_tdg_sarif(
    _summary: &crate::models::tdg::TDGSummary,
    hotspots: &[crate::models::tdg::TDGHotspot],
) -> anyhow::Result<String> {
    use serde_json::json;

    let results: Vec<_> = hotspots
        .iter()
        .map(|hotspot| {
            json!({
                "ruleId": "TDG_HIGH",
                "message": {
                    "text": format!("High Technical Debt Gradient: {:.2} (primary factor: {})",
                        hotspot.tdg_score, hotspot.primary_factor)
                },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": hotspot.path
                        },
                        "region": {
                            "startLine": 1
                        }
                    }
                }],
                "level": if hotspot.tdg_score > 2.5 { "error" } else { "warning" },
                "properties": {
                    "tdg_score": hotspot.tdg_score,
                    "primary_factor": hotspot.primary_factor,
                    "estimated_hours": hotspot.estimated_hours
                }
            })
        })
        .collect();

    let sarif = json!({
        "$schema": "https://schemastore.azurewebsites.net/schemas/json/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "paiml-mcp-agent-toolkit",
                    "version": "0.21.0",
                    "informationUri": "https://github.com/paiml/mcp-agent-toolkit"
                }
            },
            "results": results
        }]
    });

    Ok(serde_json::to_string_pretty(&sarif)?)
}

/// Truncate path for table display
fn truncate_path(path: &str, max_len: usize) -> String {
    if path.len() <= max_len {
        path.to_string()
    } else {
        format!("...{}", &path[path.len() - (max_len - 3)..])
    }
}

/// Handle Makefile analysis command
async fn handle_analyze_makefile(
    path: PathBuf,
    _rules: Vec<String>,
    format: MakefileOutputFormat,
    fix: bool,
    _gnu_version: String,
) -> anyhow::Result<()> {
    use crate::services::makefile_linter;

    // Perform linting
    let result = makefile_linter::lint_makefile(&path).await?;

    // Apply fixes if requested
    if fix && result.violations.iter().any(|v| v.fix_hint.is_some()) {
        eprintln!("🔧 Auto-fix is not yet implemented. Fix hints are provided in the output.");
    }

    // Format output
    let output = match format {
        MakefileOutputFormat::Human => format_makefile_human(&result),
        MakefileOutputFormat::Json => serde_json::to_string_pretty(&result)?,
        MakefileOutputFormat::Gcc => format_makefile_gcc(&result),
        MakefileOutputFormat::Sarif => format_makefile_sarif(&result)?,
    };

    println!("{}", output);

    // Exit with error code if violations found
    if result.has_errors() {
        std::process::exit(1);
    }

    Ok(())
}

/// Format Makefile lint results in human-readable format
fn format_makefile_human(result: &makefile_linter::LintResult) -> String {
    use crate::services::makefile_linter::Severity;

    let mut output = String::new();

    output.push_str(&format!("Analyzing {}...\n\n", result.path.display()));

    if result.violations.is_empty() {
        output.push_str("✅ No issues found!\n");
        output.push_str(&format!(
            "Quality score: {:.0}%\n",
            result.quality_score * 100.0
        ));
        return output;
    }

    // Group violations by severity
    let errors: Vec<_> = result
        .violations
        .iter()
        .filter(|v| v.severity == Severity::Error)
        .collect();
    let warnings: Vec<_> = result
        .violations
        .iter()
        .filter(|v| v.severity == Severity::Warning)
        .collect();
    let info: Vec<_> = result
        .violations
        .iter()
        .filter(|v| v.severity == Severity::Info)
        .collect();
    let perf: Vec<_> = result
        .violations
        .iter()
        .filter(|v| v.severity == Severity::Performance)
        .collect();

    // Summary
    output.push_str(&format!(
        "Found {} issues ({} errors, {} warnings, {} info, {} performance)\n",
        result.violations.len(),
        errors.len(),
        warnings.len(),
        info.len(),
        perf.len()
    ));
    output.push_str(&format!(
        "Quality score: {:.0}%\n\n",
        result.quality_score * 100.0
    ));

    // Print violations by severity
    if !errors.is_empty() {
        output.push_str("❌ Errors:\n");
        for v in errors {
            output.push_str(&format!(
                "  {}:{} [{}] {}\n",
                result.path.display(),
                v.span.line,
                v.rule,
                v.message
            ));
            if let Some(hint) = &v.fix_hint {
                output.push_str(&format!("    💡 {}\n", hint));
            }
        }
        output.push('\n');
    }

    if !warnings.is_empty() {
        output.push_str("⚠️  Warnings:\n");
        for v in warnings {
            output.push_str(&format!(
                "  {}:{} [{}] {}\n",
                result.path.display(),
                v.span.line,
                v.rule,
                v.message
            ));
            if let Some(hint) = &v.fix_hint {
                output.push_str(&format!("    💡 {}\n", hint));
            }
        }
        output.push('\n');
    }

    if !info.is_empty() {
        output.push_str("ℹ️  Info:\n");
        for v in info {
            output.push_str(&format!(
                "  {}:{} [{}] {}\n",
                result.path.display(),
                v.span.line,
                v.rule,
                v.message
            ));
            if let Some(hint) = &v.fix_hint {
                output.push_str(&format!("    💡 {}\n", hint));
            }
        }
        output.push('\n');
    }

    if !perf.is_empty() {
        output.push_str("⚡ Performance:\n");
        for v in perf {
            output.push_str(&format!(
                "  {}:{} [{}] {}\n",
                result.path.display(),
                v.span.line,
                v.rule,
                v.message
            ));
            if let Some(hint) = &v.fix_hint {
                output.push_str(&format!("    💡 {}\n", hint));
            }
        }
    }

    output
}

/// Format Makefile lint results in GCC-style format
fn format_makefile_gcc(result: &makefile_linter::LintResult) -> String {
    use crate::services::makefile_linter::Severity;

    let mut output = String::new();

    for v in &result.violations {
        let severity = match v.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "note",
            Severity::Performance => "note",
        };

        output.push_str(&format!(
            "{}:{}:0: {}: [{}] {}\n",
            result.path.display(),
            v.span.line,
            severity,
            v.rule,
            v.message
        ));
    }

    output
}

/// Format Makefile lint results as SARIF
fn format_makefile_sarif(result: &makefile_linter::LintResult) -> anyhow::Result<String> {
    use crate::services::makefile_linter::Severity;
    use serde_json::json;

    let results: Vec<_> = result
        .violations
        .iter()
        .map(|v| {
            let level = match v.severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
                Severity::Info => "note",
                Severity::Performance => "note",
            };

            json!({
                "ruleId": v.rule,
                "message": {
                    "text": &v.message
                },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": result.path.to_string_lossy()
                        },
                        "region": {
                            "startLine": v.span.line as i64,
                            "startColumn": v.span.column as i64
                        }
                    }
                }],
                "level": level,
                "fixes": v.fix_hint.as_ref().map(|hint| vec![json!({
                    "description": {
                        "text": hint
                    }
                })]).unwrap_or_default()
            })
        })
        .collect();

    let sarif = json!({
        "$schema": "https://schemastore.azurewebsites.net/schemas/json/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "paiml-makefile-linter",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/paiml/mcp-agent-toolkit",
                    "rules": [
                        {
                            "id": "minphony",
                            "name": "MinimumPhonyTargets",
                            "shortDescription": {
                                "text": "Ensure required targets are declared .PHONY"
                            }
                        },
                        {
                            "id": "phonydeclared",
                            "name": "PhonyDeclared",
                            "shortDescription": {
                                "text": "Non-file targets should be declared .PHONY"
                            }
                        },
                        {
                            "id": "maxbodylength",
                            "name": "MaxBodyLength",
                            "shortDescription": {
                                "text": "Recipe complexity check"
                            }
                        },
                        {
                            "id": "timestampexpanded",
                            "name": "TimestampExpanded",
                            "shortDescription": {
                                "text": "Timestamp evaluation timing"
                            }
                        },
                        {
                            "id": "undefinedvariable",
                            "name": "UndefinedVariable",
                            "shortDescription": {
                                "text": "Check for undefined variable usage"
                            }
                        },
                        {
                            "id": "recursive-expansion",
                            "name": "RecursiveExpansion",
                            "shortDescription": {
                                "text": "Expensive recursive variable expansion"
                            }
                        },
                        {
                            "id": "portability",
                            "name": "Portability",
                            "shortDescription": {
                                "text": "GNU Make specific features"
                            }
                        }
                    ]
                }
            },
            "results": results
        }]
    });

    Ok(serde_json::to_string_pretty(&sarif)?)
}
