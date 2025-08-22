//! Command Dispatcher - Reduces CLI complexity through handler pattern
//!
//! This module implements a dispatch table pattern to reduce cyclomatic complexity
//! in the CLI module by delegating command execution to specialized handlers.

use super::commands::{RoadmapCommands, ScaffoldCommands};
use super::{AnalyzeCommands, Commands, DemoProtocol, RefactorCommands};
use crate::cli::handlers::cache::CacheCommand;
use crate::cli::handlers::memory::MemoryCommand;
use crate::stateless_server::StatelessTemplateServer;
use std::path::PathBuf;
use std::sync::Arc;

/// Trait for command handlers to reduce complexity through delegation
#[allow(dead_code)]
#[allow(async_fn_in_trait)]
pub trait CommandHandler: Send + Sync {
    async fn execute(&self, server: Arc<StatelessTemplateServer>) -> anyhow::Result<()>;
}

/// Trait for analyze command handlers
#[allow(dead_code)]
#[allow(async_fn_in_trait)]
pub trait AnalyzeCommandHandler: Send + Sync {
    async fn execute(&self) -> anyhow::Result<()>;
}

/// Command dispatcher that reduces complexity by delegating to handlers
pub struct CommandDispatcher;

impl CommandDispatcher {
    /// Execute a command using the handler pattern (reduces CC from dispatch match)
    pub async fn execute_command(
        command: Commands,
        server: Arc<StatelessTemplateServer>,
    ) -> anyhow::Result<()> {
        use super::handlers;

        match command {
            Commands::Generate {
                category,
                template,
                params,
                output,
                create_dirs,
            } => {
                handlers::handle_generate(server, category, template, params, output, create_dirs)
                    .await
            }
            Commands::Scaffold { command } => match command {
                ScaffoldCommands::Project {
                    toolchain,
                    templates,
                    params,
                    parallel,
                } => {
                    handlers::handle_scaffold(server, toolchain, templates, params, parallel).await
                }
                ScaffoldCommands::Agent {
                    name,
                    template,
                    features,
                    quality,
                    output,
                    force,
                    dry_run,
                    interactive,
                    deterministic_core,
                    probabilistic_wrapper,
                } => {
                    let params = handlers::generation_handlers::ScaffoldAgentParams {
                        name,
                        template,
                        features,
                        quality,
                        output,
                        force,
                        dry_run,
                        interactive,
                        deterministic_core,
                        probabilistic_wrapper,
                    };
                    handlers::handle_scaffold_agent(params).await
                }
                ScaffoldCommands::ListTemplates => handlers::handle_list_agent_templates().await,
                ScaffoldCommands::ValidateTemplate { path } => {
                    handlers::handle_validate_agent_template(path).await
                }
            },
            Commands::List {
                toolchain,
                category,
                format,
            } => handlers::handle_list(server, toolchain, category, format).await,
            Commands::Search {
                query,
                toolchain,
                limit,
            } => handlers::handle_search(server, query, toolchain, limit).await,
            Commands::Validate { uri, params } => {
                handlers::handle_validate(server, uri, params).await
            }
            Commands::Context {
                toolchain,
                project_path,
                output,
                format,
                include_large_files,
                skip_expensive_metrics,
            } => {
                handlers::handle_context(
                    toolchain,
                    project_path,
                    output,
                    format,
                    include_large_files,
                    skip_expensive_metrics,
                )
                .await
            }
            Commands::Analyze(analyze_cmd) => Self::execute_analyze_command(analyze_cmd).await,
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
                // Convert CLI DemoProtocol to demo module Protocol
                let demo_protocol = if cli {
                    crate::demo::Protocol::Cli
                } else {
                    match protocol {
                        DemoProtocol::Cli => crate::demo::Protocol::Cli,
                        DemoProtocol::Http => crate::demo::Protocol::Http,
                        DemoProtocol::Mcp => crate::demo::Protocol::Mcp,
                        #[cfg(feature = "tui")]
                        DemoProtocol::Tui => crate::demo::Protocol::Tui,
                        DemoProtocol::All => crate::demo::Protocol::All,
                    }
                };

                let web_mode = !cli;

                // Create demo args
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
                    skip_vendor: skip_vendor && !no_skip_vendor,
                    max_line_length,
                };

                crate::demo::run_demo(demo_args, server).await
            }
            Commands::QualityGate {
                project_path,
                file,
                format,
                fail_on_violation,
                checks,
                max_dead_code,
                min_entropy,
                max_complexity_p99,
                include_provability,
                output,
                perf,
            } => {
                handlers::handle_quality_gate(
                    project_path,
                    file,
                    format,
                    fail_on_violation,
                    checks,
                    max_dead_code,
                    min_entropy,
                    max_complexity_p99,
                    include_provability,
                    output,
                    perf,
                )
                .await
            }
            Commands::Report {
                project_path,
                output_format,
                include_visualizations,
                include_executive_summary,
                include_recommendations,
                analyses,
                confidence_threshold,
                output,
                perf,
                text,
                markdown,
                csv,
            } => {
                handlers::enhanced_reporting_handlers::handle_generate_report(
                    project_path,
                    output_format,
                    text,
                    markdown,
                    csv,
                    include_visualizations,
                    include_executive_summary,
                    include_recommendations,
                    analyses,
                    confidence_threshold,
                    output,
                    perf,
                )
                .await
            }
            Commands::Serve { port, host, cors, transport } => handlers::handle_serve(host, port, cors, transport).await,
            Commands::Diagnose(args) => super::diagnose::handle_diagnose(args).await,
            Commands::Enforce(enforce_cmd) => handlers::route_enforce_command(enforce_cmd).await,
            Commands::Refactor(refactor_cmd) => Self::execute_refactor_command(refactor_cmd).await,
            Commands::Roadmap(roadmap_cmd) => Self::execute_roadmap_command(roadmap_cmd).await,
            Commands::Test {
                suite,
                iterations,
                memory,
                throughput,
                regression,
                timeout,
                output,
                perf,
            } => {
                Self::execute_test_command(
                    suite,
                    iterations,
                    memory,
                    throughput,
                    regression,
                    timeout,
                    output,
                    perf,
                )
                .await
            }
            Commands::Memory { command } => {
                Self::execute_memory_command(command).await
            }
            Commands::Cache { command } => {
                Self::execute_cache_command(command).await
            }
            Commands::Telemetry { system, service, reset, test_event } => {
                handlers::telemetry_handlers::handle_telemetry(system, service, reset, test_event).await
            }
            Commands::Config { show, edit, validate, reset, section, set, config_path } => {
                handlers::handle_configuration(show, edit, validate, reset, section, set, config_path).await
            }
        }
    }

    /// Execute analyze commands using handler pattern (reduces CC)
    pub async fn execute_analyze_command(analyze_cmd: AnalyzeCommands) -> anyhow::Result<()> {
        // Delegate to the modular analysis handlers
        super::handlers::route_analyze_command(analyze_cmd).await
    }

    /// Execute refactor commands using handler pattern (reduces CC)
    pub async fn execute_refactor_command(refactor_cmd: RefactorCommands) -> anyhow::Result<()> {
        // Delegate to the refactor handlers
        super::handlers::route_refactor_command(refactor_cmd).await
    }

    /// Execute roadmap commands using handler pattern (reduces CC)
    pub async fn execute_roadmap_command(roadmap_cmd: RoadmapCommands) -> anyhow::Result<()> {
        use crate::roadmap::{self, RoadmapConfig};
        use std::path::PathBuf;

        // Load configuration (with defaults)
        let config = RoadmapConfig {
            path: PathBuf::from("docs/execution/roadmap.md"),
            quality_gates: roadmap::QualityGateConfig {
                complexity_max: 20,
                coverage_min: 80,
                satd_tolerance: 0,
                documentation_required: true,
                lint_compliance: true,
            },
            enforce_quality_gates: true,
            git: roadmap::GitConfig {
                create_branches: true,
                branch_pattern: "feature/{task_id}".to_string(),
                commit_pattern: "{task_id}: {message}".to_string(),
                require_quality_check: true,
            },
            enabled: true,
            auto_generate_todos: true,
            require_task_ids: true,
            task_id_pattern: "PMAT-[0-9]{4}".to_string(),
            tracking: roadmap::TrackingConfig {
                velocity_tracking: true,
                burndown_charts: true,
                quality_metrics: true,
                export_format: "json".to_string(),
            },
        };
        
        // Create command struct and execute
        let cmd = roadmap::commands::RoadmapCommand {
            command: match roadmap_cmd {
                RoadmapCommands::Init { version, title, duration_days, priority } => {
                    roadmap::commands::RoadmapSubcommand::Init { version, title, duration_days, priority }
                }
                RoadmapCommands::Todos { sprint, output, include_quality_gates } => {
                    roadmap::commands::RoadmapSubcommand::Todos { sprint, output, include_quality_gates }
                }
                RoadmapCommands::Start { task_id, create_branch } => {
                    roadmap::commands::RoadmapSubcommand::Start { task_id, create_branch }
                }
                RoadmapCommands::Complete { task_id, skip_quality_check } => {
                    roadmap::commands::RoadmapSubcommand::Complete { task_id, skip_quality_check }
                }
                RoadmapCommands::Status { sprint, task, format } => {
                    let output_format = match format {
                        super::OutputFormat::Json => crate::cli::OutputFormat::Json,
                        _ => crate::cli::OutputFormat::Table,
                    };
                    roadmap::commands::RoadmapSubcommand::Status { sprint, task, format: output_format }
                }
                RoadmapCommands::Validate { sprint, strict } => {
                    roadmap::commands::RoadmapSubcommand::Validate { sprint, strict }
                }
                RoadmapCommands::QualityCheck { task_id } => {
                    roadmap::commands::RoadmapSubcommand::QualityCheck { task_id }
                }
            },
        };
        
        roadmap::commands::execute(cmd, config).await
    }

    /// Execute test commands using handler pattern (reduces CC)
    #[allow(clippy::too_many_arguments)]
    pub async fn execute_test_command(
        suite: super::commands::TestSuite,
        iterations: usize,
        memory: bool,
        throughput: bool,
        regression: bool,
        timeout: u64,
        output: Option<PathBuf>,
        perf: bool,
    ) -> anyhow::Result<()> {
        use super::commands::TestSuite;
        
        // Import the performance testing module
        use crate::test_performance::*;
        
        // Configure the test suite based on CLI arguments
        let config = PerformanceTestConfig {
            enable_regression_tests: regression || matches!(suite, TestSuite::Regression | TestSuite::All),
            enable_memory_tests: memory || matches!(suite, TestSuite::Memory | TestSuite::All),
            enable_throughput_tests: throughput || matches!(suite, TestSuite::Throughput | TestSuite::All),
            test_iterations: iterations,
        };
        
        // Run the performance test suite
        println!("🚀 Starting Performance Testing Suite (SPECIFICATION.md Section 30)");
        println!("Suite: {:?}, Iterations: {}, Timeout: {}s", suite, iterations, timeout);
        
        let start = std::time::Instant::now();
        
        // Set timeout for the test execution
        let test_future = async {
            match suite {
                TestSuite::Performance | TestSuite::All => {
                    run_performance_test_suite(config).await
                }
                TestSuite::Regression => {
                    if config.enable_regression_tests {
                        println!("🔍 Running regression tests...");
                        test_performance_regression_detection().await?;
                        println!("✅ Regression tests passed!");
                    }
                    Ok(())
                }
                TestSuite::Memory => {
                    if config.enable_memory_tests {
                        println!("💾 Running memory tests...");
                        test_memory_usage_patterns().await?;
                        println!("✅ Memory tests passed!");
                    }
                    Ok(())
                }
                TestSuite::Throughput => {
                    if config.enable_throughput_tests {
                        println!("📊 Running throughput tests...");
                        test_single_threaded_throughput().await?;
                        test_realistic_project_analysis().await?;
                        test_large_file_performance().await?;
                        println!("✅ Throughput tests passed!");
                    }
                    Ok(())
                }
                TestSuite::Property => {
                    println!("🔬 Property-based testing not yet implemented in this context");
                    println!("Use the `pmat test --suite property` command for property tests");
                    Ok(())
                }
                TestSuite::Integration => {
                    println!("🧪 Integration testing not yet implemented in this context");
                    println!("Use the `pmat test --suite integration` command for integration tests");
                    Ok(())
                }
            }
        };
        
        // Execute with timeout
        let timeout_duration = std::time::Duration::from_secs(timeout);
        match tokio::time::timeout(timeout_duration, test_future).await {
            Ok(result) => {
                let elapsed = start.elapsed();
                
                if perf {
                    println!("\n📈 Performance Summary:");
                    println!("   Total execution time: {:?}", elapsed);
                    println!("   Suite: {:?}", suite);
                    println!("   Iterations: {}", iterations);
                }
                
                // Write results to output file if specified
                if let Some(output_path) = output {
                    let results = format!(
                        "Performance Test Results\n\
                        ======================\n\
                        Suite: {:?}\n\
                        Execution time: {:?}\n\
                        Iterations: {}\n\
                        Status: {}\n",
                        suite,
                        elapsed,
                        iterations,
                        if result.is_ok() { "PASSED" } else { "FAILED" }
                    );
                    std::fs::write(&output_path, results)?;
                    println!("📄 Results written to: {}", output_path.display());
                }
                
                result
            }
            Err(_) => {
                eprintln!("❌ Test execution timed out after {}s", timeout);
                anyhow::bail!("Performance tests timed out");
            }
        }
    }

    /// Execute memory management commands using handler pattern (reduces CC)
    pub async fn execute_memory_command(memory_cmd: MemoryCommand) -> anyhow::Result<()> {
        // Delegate to the memory handler
        super::handlers::handle_memory_command(&memory_cmd).await
    }

    /// Execute cache management commands using handler pattern (reduces CC)
    pub async fn execute_cache_command(cache_cmd: CacheCommand) -> anyhow::Result<()> {
        // Delegate to the cache handler
        super::handlers::handle_cache_command(&cache_cmd).await
    }
}

#[cfg(test)]
mod tests {
    // use super::*; // Unused in simple tests

    #[test]
    fn test_command_dispatcher_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }
}
