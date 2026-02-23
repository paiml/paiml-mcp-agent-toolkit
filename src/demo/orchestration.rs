#![cfg_attr(coverage_nightly, coverage(off))]
//! Demo orchestration and coordination logic.

use anyhow::Result;

use super::protocols::{
    build_protocol_request, format_and_print_output, print_api_metadata, print_protocol_banner,
    protocol_to_string, DemoArgs, Protocol,
};
use super::runner;
use super::web::run_web_demo;

pub async fn run_demo(
    args: DemoArgs,
    server: std::sync::Arc<crate::stateless_server::StatelessTemplateServer>,
) -> Result<()> {
    let config = load_demo_config(args, server).await?;
    let analyzer = create_analyzer(config.clone())?;
    let results = run_analyses(analyzer, &config).await?;
    let output = generate_output(results, config.args.protocol)?;
    handle_protocol_output(output, &config).await
}

// Configuration loading and validation
async fn load_demo_config(
    args: DemoArgs,
    server: std::sync::Arc<crate::stateless_server::StatelessTemplateServer>,
) -> Result<DemoConfig> {
    let repo_path =
        runner::resolve_repository_async(args.path.clone(), args.url.clone(), args.repo.clone())
            .await?;
    Ok(DemoConfig {
        repo_path,
        args,
        server,
    })
}

// Create the appropriate analyzer based on configuration
fn create_analyzer(config: DemoConfig) -> Result<DemoAnalyzer> {
    use super::adapters::{cli::CliDemoAdapter, http::HttpDemoAdapter, mcp::McpDemoAdapter};
    use super::protocol_harness::DemoEngine;

    let mut engine = DemoEngine::new();
    engine.register_protocol("cli".to_string(), CliDemoAdapter::new());
    engine.register_protocol("http".to_string(), HttpDemoAdapter::new());
    engine.register_protocol("mcp".to_string(), McpDemoAdapter::new());

    Ok(DemoAnalyzer { engine, config })
}

// Run the actual analyses based on protocol
async fn run_analyses(analyzer: DemoAnalyzer, config: &DemoConfig) -> Result<AnalysisResults> {
    if config.args.web {
        return Ok(AnalysisResults::Web);
    }

    #[cfg(feature = "tui")]
    if config.args.protocol == Protocol::Tui {
        return Ok(AnalysisResults::Tui);
    }

    if config.args.protocol == Protocol::All {
        run_all_protocols(analyzer, config).await
    } else {
        run_single_protocol(analyzer, config).await
    }
}

// Generate output based on results and protocol
fn generate_output(results: AnalysisResults, _protocol: Protocol) -> Result<DemoOutput> {
    match results {
        AnalysisResults::Web => Ok(DemoOutput::Web),
        #[cfg(feature = "tui")]
        AnalysisResults::Tui => Ok(DemoOutput::Tui),
        AnalysisResults::Single(trace) => Ok(DemoOutput::Single(trace)),
        AnalysisResults::Multiple(traces) => Ok(DemoOutput::Multiple(traces)),
    }
}

// Handle the final output based on configuration
async fn handle_protocol_output(output: DemoOutput, config: &DemoConfig) -> Result<()> {
    match output {
        DemoOutput::Web => {
            run_web_demo(
                config.repo_path.clone(),
                config.server.clone(),
                config.args.no_browser,
                config.args.port,
            )
            .await
        }
        #[cfg(feature = "tui")]
        DemoOutput::Tui => super::web::run_tui_demo(config.repo_path.clone()).await,
        DemoOutput::Single(trace) => {
            format_and_print_output(&trace.response, &config.args.format)?;
            if config.args.show_api {
                print_api_metadata(&trace.protocol_name).await?;
            }
            Ok(())
        }
        DemoOutput::Multiple(traces) => {
            for trace in traces {
                println!("\n=== {} Protocol ===", trace.protocol_name.to_uppercase());
                format_and_print_output(&trace.response, &config.args.format)?;
            }
            Ok(())
        }
    }
}

// Run demo for all protocols
async fn run_all_protocols(analyzer: DemoAnalyzer, config: &DemoConfig) -> Result<AnalysisResults> {
    println!("🎯 All Protocols Demo");
    let mut traces = Vec::new();

    for protocol_name in analyzer.engine.list_protocols() {
        let request =
            build_protocol_request(&protocol_name, &config.repo_path, config.args.show_api);
        match analyzer.engine.execute_demo(&protocol_name, request).await {
            Ok(trace) => traces.push(ProtocolTrace {
                protocol_name: protocol_name.clone(),
                response: trace.response,
            }),
            Err(e) => eprintln!("Error executing {protocol_name} protocol: {e}"),
        }
    }

    Ok(AnalysisResults::Multiple(traces))
}

// Run demo for a single protocol
async fn run_single_protocol(
    analyzer: DemoAnalyzer,
    config: &DemoConfig,
) -> Result<AnalysisResults> {
    let protocol_name = protocol_to_string(&config.args.protocol);
    print_protocol_banner(&config.args.protocol);

    let request = build_protocol_request(&protocol_name, &config.repo_path, config.args.show_api);
    let trace = analyzer
        .engine
        .execute_demo(&protocol_name, request)
        .await?;

    Ok(AnalysisResults::Single(ProtocolTrace {
        protocol_name,
        response: trace.response,
    }))
}

// Helper structures for the refactored code
#[derive(Clone)]
pub(crate) struct DemoConfig {
    pub(crate) repo_path: std::path::PathBuf,
    pub(crate) args: DemoArgs,
    pub(crate) server: std::sync::Arc<crate::stateless_server::StatelessTemplateServer>,
}

pub(crate) struct DemoAnalyzer {
    pub(crate) engine: super::protocol_harness::DemoEngine,
    #[allow(dead_code)]
    pub(crate) config: DemoConfig,
}

pub(crate) enum AnalysisResults {
    Web,
    #[cfg(feature = "tui")]
    Tui,
    Single(ProtocolTrace),
    Multiple(Vec<ProtocolTrace>),
}

#[derive(Clone)]
pub(crate) struct ProtocolTrace {
    pub(crate) protocol_name: String,
    pub(crate) response: serde_json::Value,
}

pub(crate) enum DemoOutput {
    Web,
    #[cfg(feature = "tui")]
    Tui,
    Single(ProtocolTrace),
    Multiple(Vec<ProtocolTrace>),
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // ============================================================
    // generate_output tests
    // ============================================================

    #[test]
    fn test_generate_output_web() {
        let result = generate_output(AnalysisResults::Web, Protocol::Cli);
        assert!(result.is_ok());

        match result.unwrap() {
            DemoOutput::Web => {}
            _ => panic!("Expected DemoOutput::Web"),
        }
    }

    #[test]
    fn test_generate_output_single() {
        let trace = ProtocolTrace {
            protocol_name: "cli".to_string(),
            response: serde_json::json!({"status": "ok"}),
        };
        let result = generate_output(AnalysisResults::Single(trace), Protocol::Cli);
        assert!(result.is_ok());

        match result.unwrap() {
            DemoOutput::Single(t) => {
                assert_eq!(t.protocol_name, "cli");
            }
            _ => panic!("Expected DemoOutput::Single"),
        }
    }

    #[test]
    fn test_generate_output_multiple() {
        let traces = vec![
            ProtocolTrace {
                protocol_name: "cli".to_string(),
                response: serde_json::json!({"protocol": "cli"}),
            },
            ProtocolTrace {
                protocol_name: "http".to_string(),
                response: serde_json::json!({"protocol": "http"}),
            },
        ];
        let result = generate_output(AnalysisResults::Multiple(traces), Protocol::All);
        assert!(result.is_ok());

        match result.unwrap() {
            DemoOutput::Multiple(t) => {
                assert_eq!(t.len(), 2);
                assert_eq!(t[0].protocol_name, "cli");
                assert_eq!(t[1].protocol_name, "http");
            }
            _ => panic!("Expected DemoOutput::Multiple"),
        }
    }

    // ============================================================
    // ProtocolTrace struct tests
    // ============================================================

    #[test]
    fn test_protocol_trace_creation() {
        let trace = ProtocolTrace {
            protocol_name: "test".to_string(),
            response: serde_json::json!({"result": "success"}),
        };

        assert_eq!(trace.protocol_name, "test");
        assert_eq!(trace.response["result"], "success");
    }

    #[test]
    fn test_protocol_trace_clone() {
        let trace = ProtocolTrace {
            protocol_name: "cli".to_string(),
            response: serde_json::json!({"data": [1, 2, 3]}),
        };

        let cloned = trace.clone();
        assert_eq!(trace.protocol_name, cloned.protocol_name);
        assert_eq!(trace.response, cloned.response);
    }

    // ============================================================
    // DemoConfig struct tests
    // ============================================================

    #[test]
    fn test_demo_config_clone() {
        // Create a mock StatelessTemplateServer
        let server =
            std::sync::Arc::new(crate::stateless_server::StatelessTemplateServer::new().unwrap());

        let config = DemoConfig {
            repo_path: PathBuf::from("/test/repo"),
            args: DemoArgs {
                path: Some(PathBuf::from("/test")),
                url: None,
                repo: None,
                format: crate::cli::OutputFormat::Json,
                no_browser: false,
                port: None,
                web: false,
                target_nodes: 10,
                centrality_threshold: 0.5,
                merge_threshold: 3,
                protocol: Protocol::Cli,
                show_api: false,
                debug: false,
                debug_output: None,
                skip_vendor: false,
                max_line_length: None,
            },
            server: server.clone(),
        };

        let cloned = config.clone();
        assert_eq!(config.repo_path, cloned.repo_path);
        assert_eq!(config.args.target_nodes, cloned.args.target_nodes);
    }

    // ============================================================
    // create_analyzer tests
    // ============================================================

    #[test]
    fn test_create_analyzer_registers_protocols() {
        let server =
            std::sync::Arc::new(crate::stateless_server::StatelessTemplateServer::new().unwrap());

        let config = DemoConfig {
            repo_path: PathBuf::from("/test"),
            args: DemoArgs {
                path: None,
                url: None,
                repo: None,
                format: crate::cli::OutputFormat::Json,
                no_browser: false,
                port: None,
                web: false,
                target_nodes: 10,
                centrality_threshold: 0.5,
                merge_threshold: 3,
                protocol: Protocol::All,
                show_api: false,
                debug: false,
                debug_output: None,
                skip_vendor: false,
                max_line_length: None,
            },
            server,
        };

        let result = create_analyzer(config);
        assert!(result.is_ok());

        let analyzer = result.unwrap();
        let protocols = analyzer.engine.list_protocols();

        assert!(protocols.contains(&"cli".to_string()));
        assert!(protocols.contains(&"http".to_string()));
        assert!(protocols.contains(&"mcp".to_string()));
    }

    // ============================================================
    // AnalysisResults enum tests
    // ============================================================

    #[test]
    fn test_analysis_results_web_variant() {
        let result = AnalysisResults::Web;
        match result {
            AnalysisResults::Web => {}
            _ => panic!("Expected Web variant"),
        }
    }

    #[test]
    fn test_analysis_results_single_variant() {
        let trace = ProtocolTrace {
            protocol_name: "test".to_string(),
            response: serde_json::json!({}),
        };
        let result = AnalysisResults::Single(trace);

        match result {
            AnalysisResults::Single(t) => {
                assert_eq!(t.protocol_name, "test");
            }
            _ => panic!("Expected Single variant"),
        }
    }

    #[test]
    fn test_analysis_results_multiple_variant() {
        let traces = vec![
            ProtocolTrace {
                protocol_name: "a".to_string(),
                response: serde_json::json!({}),
            },
            ProtocolTrace {
                protocol_name: "b".to_string(),
                response: serde_json::json!({}),
            },
        ];
        let result = AnalysisResults::Multiple(traces);

        match result {
            AnalysisResults::Multiple(t) => {
                assert_eq!(t.len(), 2);
            }
            _ => panic!("Expected Multiple variant"),
        }
    }

    // ============================================================
    // DemoOutput enum tests
    // ============================================================

    #[test]
    fn test_demo_output_web_variant() {
        let output = DemoOutput::Web;
        match output {
            DemoOutput::Web => {}
            _ => panic!("Expected Web variant"),
        }
    }

    #[test]
    fn test_demo_output_single_variant() {
        let trace = ProtocolTrace {
            protocol_name: "demo".to_string(),
            response: serde_json::json!({"key": "value"}),
        };
        let output = DemoOutput::Single(trace);

        match output {
            DemoOutput::Single(t) => {
                assert_eq!(t.protocol_name, "demo");
            }
            _ => panic!("Expected Single variant"),
        }
    }

    #[test]
    fn test_demo_output_multiple_variant() {
        let traces = vec![ProtocolTrace {
            protocol_name: "test".to_string(),
            response: serde_json::json!({}),
        }];
        let output = DemoOutput::Multiple(traces);

        match output {
            DemoOutput::Multiple(t) => {
                assert_eq!(t.len(), 1);
            }
            _ => panic!("Expected Multiple variant"),
        }
    }
}
