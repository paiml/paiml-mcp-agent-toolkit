pub mod args;

use crate::{
    models::{churn::ChurnOutputFormat, template::*},
    services::template_service::*,
    stateless_server::StatelessTemplateServer,
};
use clap::{Parser, Subcommand, ValueEnum};
use serde_json::Value;
use std::{path::PathBuf, sync::Arc};
use tokio::io::AsyncWriteExt;

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
    mode: Option<Mode>,

    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Clone, Debug, ValueEnum)]
enum Mode {
    Cli,
    Mcp,
}

#[derive(Subcommand)]
#[cfg_attr(test, derive(Debug))]
pub(crate) enum Commands {
    /// Generate a single template
    #[command(visible_aliases = &["gen", "g"])]
    Generate {
        /// Template category
        category: String,

        /// Template path (e.g., rust/cli)
        template: String,

        /// Parameters as key=value pairs
        #[arg(short = 'p', long = "param", value_parser = parse_key_val)]
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
        #[arg(short = 'p', long = "param", value_parser = parse_key_val)]
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
        #[arg(short = 'p', long = "param", value_parser = parse_key_val)]
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
}

#[derive(Subcommand)]
#[cfg_attr(test, derive(Debug))]
pub(crate) enum AnalyzeCommands {
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
}

#[derive(Clone, Debug, ValueEnum)]
pub(crate) enum ContextFormat {
    Markdown,
    Json,
}

#[derive(Clone, Debug, ValueEnum)]
pub(crate) enum OutputFormat {
    Table,
    Json,
    Yaml,
}

pub async fn run(server: Arc<StatelessTemplateServer>) -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Handle forced mode
    if let Some(Mode::Mcp) = cli.mode {
        return crate::run_mcp_server(server).await;
    }

    match cli.command {
        Commands::Generate {
            category,
            template,
            params,
            output,
            create_dirs,
        } => {
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
        }

        Commands::Scaffold {
            toolchain,
            templates,
            params,
            parallel,
        } => {
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
        }

        Commands::List {
            toolchain,
            category,
            format,
        } => {
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
        }

        Commands::Search {
            query,
            toolchain,
            limit,
        } => {
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
        }

        Commands::Validate { uri, params } => {
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
        }

        Commands::Context {
            toolchain,
            project_path,
            output,
            format,
        } => {
            use crate::services::context::{analyze_project, format_context_as_markdown};

            // Analyze the project
            let context = analyze_project(&project_path, &toolchain).await?;

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
        }

        Commands::Analyze(analyze_cmd) => match analyze_cmd {
            AnalyzeCommands::Churn {
                project_path,
                days,
                format,
                output,
            } => {
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
            }
        },
    }

    Ok(())
}

// Zero-allocation parameter parsing for common types
fn parse_key_val(s: &str) -> Result<(String, Value), String> {
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{s}`"))?;

    let key = &s[..pos];
    let val = &s[pos + 1..];

    // Type inference with fast paths
    let value = if val.is_empty() {
        Value::Bool(true) // Treat bare flags as true
    } else if val == "true" || val == "false" {
        Value::Bool(val.parse().unwrap())
    } else if let Ok(n) = val.parse::<i64>() {
        Value::Number(n.into())
    } else if let Ok(f) = val.parse::<f64>() {
        Value::Number(serde_json::Number::from_f64(f).unwrap())
    } else {
        Value::String(val.to_string())
    };

    Ok((key.to_string(), value))
}

fn params_to_json(params: Vec<(String, Value)>) -> serde_json::Map<String, Value> {
    let mut map = serde_json::Map::new();
    for (k, v) in params {
        map.insert(k, v);
    }
    map
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
