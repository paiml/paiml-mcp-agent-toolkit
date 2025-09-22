use clap::{Parser, Subcommand};
use pmat::agents::registry::AgentRegistry;
use pmat::mcp_integration::server::{McpServer, ServerConfig};
use pmat::workflow::{WorkflowBuilder, DefaultWorkflowExecutor, WorkflowContext};
use pmat::workflow::dsl::DslCompiler;
use std::sync::Arc;
use std::time::Duration;
use tokio::fs;

#[derive(Parser)]
#[command(name = "pmat-agent")]
#[command(about = "PMAT Agent System - Enterprise-grade agent orchestration", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the MCP server
    Serve {
        /// Bind address for TCP server
        #[arg(short, long, default_value = "127.0.0.1:3000")]
        bind: String,
        
        /// Unix socket path
        #[arg(short = 'u', long)]
        socket: Option<String>,
        
        /// Use stdio instead of network
        #[arg(long)]
        stdio: bool,
        
        /// Maximum concurrent connections
        #[arg(long, default_value_t = 100)]
        max_connections: usize,
    },
    
    /// Execute a workflow
    Execute {
        /// Workflow file path (YAML/JSON)
        #[arg(short, long)]
        file: String,
        
        /// Input parameters (JSON)
        #[arg(short, long)]
        params: Option<String>,
        
        /// Timeout in seconds
        #[arg(short, long)]
        timeout: Option<u64>,
    },
    
    /// Validate a workflow
    Validate {
        /// Workflow file path
        #[arg(short, long)]
        file: String,
    },
    
    /// Analyze code quality
    Analyze {
        /// Source code file or directory
        #[arg(short, long)]
        path: String,
        
        /// Programming language
        #[arg(short, long)]
        language: String,
        
        /// Output format (json, text, html)
        #[arg(short, long, default_value = "text")]
        output: String,
    },
    
    /// Run quality gates
    QualityGate {
        /// Source path
        #[arg(short, long)]
        path: String,
        
        /// Language
        #[arg(short, long)]
        language: String,
        
        /// Max complexity threshold
        #[arg(long, default_value_t = 10)]
        max_complexity: u32,
        
        /// Max SATD items
        #[arg(long, default_value_t = 0)]
        max_satd: usize,
        
        /// Fail on violation
        #[arg(long)]
        fail_on_violation: bool,
    },
    
    /// Show system info
    Info,
}

#[actix_rt::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();
    
    let cli = Cli::parse();
    
    // Initialize agent registry
    let registry = Arc::new(AgentRegistry::new());
    initialize_agents(&registry).await?;
    
    match cli.command {
        Commands::Serve { bind, socket, stdio, max_connections } => {
            serve_mcp(registry, bind, socket, stdio, max_connections).await?
        }
        Commands::Execute { file, params, timeout } => {
            execute_workflow(registry, file, params, timeout).await?
        }
        Commands::Validate { file } => {
            validate_workflow(file).await?
        }
        Commands::Analyze { path, language, output } => {
            analyze_code(registry, path, language, output).await?
        }
        Commands::QualityGate { path, language, max_complexity, max_satd, fail_on_violation } => {
            run_quality_gate(registry, path, language, max_complexity, max_satd, fail_on_violation).await?
        }
        Commands::Info => {
            show_info().await?
        }
    }
    
    Ok(())
}

async fn initialize_agents(registry: &Arc<AgentRegistry>) -> Result<(), Box<dyn std::error::Error>> {
    use pmat::agents::*;
    
    // Register core agents
    registry.register("analyzer", Arc::new(AnalyzerActor::new())).await;
    registry.register("transformer", Arc::new(TransformerActor::new())).await;
    registry.register("validator", Arc::new(ValidatorActor::new())).await;
    registry.register("orchestrator", Arc::new(OrchestratorActor::new())).await;
    
    println!("✓ Initialized {} agents", registry.list_agents().await.len());
    
    Ok(())
}

async fn serve_mcp(
    registry: Arc<AgentRegistry>,
    bind: String,
    socket: Option<String>,
    stdio: bool,
    max_connections: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = ServerConfig {
        name: "PMAT Agent Server".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        bind_address: bind.clone(),
        unix_socket: socket.clone(),
        max_connections,
        request_timeout: Duration::from_secs(30),
        enable_logging: true,
    };
    
    let server = McpServer::new(registry, config)?;
    server.register_defaults().await?;
    
    println!("🚀 PMAT Agent Server v{}", env!("CARGO_PKG_VERSION"));
    println!("   Protocol: MCP {}", pmat::mcp_integration::MCP_VERSION);
    
    if stdio {
        println!("📝 Using stdio transport");
        server.run_stdio().await?;
    } else if let Some(socket_path) = socket {
        println!("🔌 Listening on Unix socket: {}", socket_path);
        server.run_unix().await?;
    } else {
        println!("🌐 Listening on TCP: {}", bind);
        server.run_tcp().await?;
    }
    
    Ok(())
}

async fn execute_workflow(
    registry: Arc<AgentRegistry>,
    file: String,
    params: Option<String>,
    timeout: Option<u64>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("📋 Loading workflow: {}", file);
    
    let content = fs::read_to_string(&file).await?;
    let mut workflow = DslCompiler::compile(&content)?;
    
    if let Some(timeout_secs) = timeout {
        workflow.timeout = Some(Duration::from_secs(timeout_secs));
    }
    
    println!("▶️  Executing workflow: {}", workflow.name);
    println!("   Steps: {}", workflow.steps.len());
    
    let context = WorkflowContext::new(workflow.id, registry.clone());
    
    // Set initial parameters
    if let Some(params_json) = params {
        let params: serde_json::Value = serde_json::from_str(&params_json)?;
        for (key, value) in params.as_object().unwrap_or(&serde_json::Map::new()) {
            context.set_variable(key.clone(), value.clone());
        }
    }
    
    let executor = DefaultWorkflowExecutor::new(registry);
    let start = std::time::Instant::now();
    
    match executor.execute(&workflow, &context).await {
        Ok(result) => {
            let elapsed = start.elapsed();
            println!("✅ Workflow completed in {:.2}s", elapsed.as_secs_f64());
            println!("   Result: {}", serde_json::to_string_pretty(&result)?);
        }
        Err(e) => {
            println!("❌ Workflow failed: {}", e);
            std::process::exit(1);
        }
    }
    
    Ok(())
}

async fn validate_workflow(file: String) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Validating workflow: {}", file);
    
    let content = fs::read_to_string(&file).await?;
    
    match DslCompiler::compile(&content) {
        Ok(workflow) => {
            println!("✅ Valid workflow: {}", workflow.name);
            println!("   Version: {}", workflow.version);
            println!("   Steps: {}", workflow.steps.len());
            
            for (i, step) in workflow.steps.iter().enumerate() {
                println!("   {}. {} ({})", i + 1, step.name, step.id);
            }
        }
        Err(e) => {
            println!("❌ Invalid workflow: {}", e);
            std::process::exit(1);
        }
    }
    
    Ok(())
}

async fn analyze_code(
    registry: Arc<AgentRegistry>,
    path: String,
    language: String,
    output: String,
) -> Result<(), Box<dyn std::error::Error>> {
    use pmat::quality::complexity::ComplexityAnalyzer;
    use pmat::quality::satd::SatdDetector;
    use pmat::quality::entropy::EntropyCalculator;
    
    println!("🔬 Analyzing: {}", path);
    
    let code = fs::read_to_string(&path).await?;
    
    // Run analyzers
    let analyzer = ComplexityAnalyzer::default();
    let complexity = analyzer.analyze_code(&code, &language);
    
    let detector = SatdDetector::new();
    let satd_items = detector.detect(&code);
    
    let calculator = EntropyCalculator::new();
    let entropy = calculator.calculate(code.as_bytes());
    
    match output.as_str() {
        "json" => {
            let result = serde_json::json!({
                "file": path,
                "language": language,
                "complexity": {
                    "cyclomatic": complexity.cyclomatic,
                    "cognitive": complexity.cognitive,
                },
                "satd": satd_items,
                "entropy": entropy,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        "html" => {
            // Would generate HTML report
            println!("HTML output not yet implemented");
        }
        _ => {
            // Text output
            println!("📊 Analysis Results:");
            println!("   Cyclomatic Complexity: {}", complexity.cyclomatic);
            println!("   Cognitive Complexity: {}", complexity.cognitive);
            println!("   Shannon Entropy: {:.2}", entropy);
            println!("   SATD Items: {}", satd_items.len());
            
            if !satd_items.is_empty() {
                println!("\n⚠️  Self-Admitted Technical Debt:");
                for item in satd_items {
                    println!("   - {} (line {}): {}", item.satd_type, item.line, item.comment);
                }
            }
        }
    }
    
    Ok(())
}

async fn run_quality_gate(
    registry: Arc<AgentRegistry>,
    path: String,
    language: String,
    max_complexity: u32,
    max_satd: usize,
    fail_on_violation: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use pmat::quality::gate::{QualityGate, QualityThresholds};
    use pmat::quality::complexity::ComplexityAnalyzer;
    use pmat::quality::satd::SatdDetector;
    
    println!("🚦 Running quality gates on: {}", path);
    
    let code = fs::read_to_string(&path).await?;
    
    let gate = QualityGate::new(
        vec![
            Box::new(ComplexityAnalyzer::default()),
            Box::new(SatdDetector::new()),
        ],
        QualityThresholds {
            max_complexity,
            max_satd_items: max_satd,
            min_test_coverage: 0.0, // Not checking coverage
            max_duplication: 1.0,   // Not checking duplication
        },
    );
    
    let result = gate.check(&code, &language).await;
    
    if result.passed {
        println!("✅ Quality gates PASSED");
    } else {
        println!("❌ Quality gates FAILED");
        
        for violation in &result.violations {
            println!("   ⚠️  {}", violation);
        }
        
        if fail_on_violation {
            std::process::exit(1);
        }
    }
    
    println!("\n📊 Metrics:");
    for (key, value) in &result.metrics {
        println!("   {}: {}", key, value);
    }
    
    Ok(())
}

async fn show_info() -> Result<(), Box<dyn std::error::Error>> {
    println!("PMAT Agent System v{}", env!("CARGO_PKG_VERSION"));
    println!("═══════════════════════════════════════");
    println!("MCP Protocol: {}", pmat::mcp_integration::MCP_VERSION);
    println!("Build: {} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    println!();
    println!("Features:");
    println!("  ✓ Actix Actor System");
    println!("  ✓ Zero-copy Message Passing");
    println!("  ✓ Event Sourcing with Snapshots");
    println!("  ✓ Raft Consensus");
    println!("  ✓ Resource Control (CPU/Memory/GPU/Network/IO)");
    println!("  ✓ MCP Protocol Integration");
    println!("  ✓ Workflow Orchestration");
    println!("  ✓ Quality Gates (Complexity/SATD/Entropy)");
    println!();
    println!("Repository: https://github.com/paiml/pmat-agent-toolkit");
    
    Ok(())
}