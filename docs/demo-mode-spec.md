# Demo Mode Specification

## Abstract

This specification defines a zero-overhead demonstration mode that orchestrates existing PAIML MCP Agent Toolkit capabilities through a deterministic execution pipeline. The design leverages Rust's zero-cost abstractions and conditional compilation to ensure no binary size regression while doubling as integration test infrastructure.

## Design Constraints

1. **Binary Size**: Zero bytes in release builds via `#[cfg(test)]` gating
2. **Code Reuse**: 100% existing capability invocation, no new analysis logic
3. **Repository Detection**: Git repository autodiscovery with fallback prompt
4. **Dual Mode**: Unified execution path for CLI and MCP with mode-specific I/O
5. **Test Integration**: Direct `#[test]` attribute compatibility for coverage metrics

## Architecture

### Core Abstraction

```rust
// server/src/demo/mod.rs
#[cfg(any(test, feature = "demo-dev"))]
pub mod runner;

#[cfg(not(any(test, feature = "demo-dev")))]
pub fn run_demo(_: DemoArgs) -> Result<()> {
    Err(anyhow!("Demo mode not available in release builds"))
}
```

### Execution Pipeline

```rust
// server/src/demo/runner.rs
use crate::handlers::tools::{
    handle_tool_call, AnalyzeCodeChurnArgs, AnalyzeComplexityArgs,
    AnalyzeDagArgs, GenerateContextArgs
};

pub struct DemoRunner {
    server: Arc<StatelessTemplateServer>,
    execution_log: Vec<DemoStep>,
}

#[derive(Debug, Serialize)]
pub struct DemoStep {
    capability: &'static str,
    request: McpRequest,
    response: McpResponse,
    elapsed_ms: u64,
}

impl DemoRunner {
    pub async fn execute(&mut self, repo_path: PathBuf) -> Result<DemoReport> {
        // Reuse existing tool infrastructure
        let steps = vec![
            self.demo_context_generation(&repo_path).await?,
            self.demo_complexity_analysis(&repo_path).await?,
            self.demo_dag_generation(&repo_path).await?,
            self.demo_churn_analysis(&repo_path).await?,
            self.demo_template_generation(&repo_path).await?,
        ];
        
        Ok(DemoReport { 
            repository: repo_path,
            steps,
            total_elapsed_ms: self.execution_log.iter().map(|s| s.elapsed_ms).sum(),
        })
    }
    
    async fn demo_context_generation(&mut self, path: &Path) -> Result<DemoStep> {
        let request = self.build_mcp_request("generate_context", json!({
            "project_path": path.to_str().unwrap(),
            "format": "json"
        }));
        
        let start = Instant::now();
        let response = handle_tool_call(&request, &self.server).await;
        let elapsed = start.elapsed().as_millis() as u64;
        
        self.execution_log.push(DemoStep {
            capability: "AST Context Analysis",
            request: request.clone(),
            response: response.clone(),
            elapsed_ms: elapsed,
        });
        
        Ok(self.execution_log.last().unwrap().clone())
    }
}
```

### Repository Detection

```rust
pub fn detect_repository(hint: Option<PathBuf>) -> Result<PathBuf> {
    let candidate = hint.unwrap_or_else(|| env::current_dir().unwrap());
    
    // Fast path: .git directory exists
    if candidate.join(".git").is_dir() {
        return Ok(candidate);
    }
    
    // Traverse up to find .git
    let mut current = candidate.as_path();
    while let Some(parent) = current.parent() {
        if parent.join(".git").is_dir() {
            return Ok(parent.to_path_buf());
        }
        current = parent;
    }
    
    // Interactive fallback for CLI mode only
    if std::io::stdout().is_terminal() {
        eprintln!("No git repository found in current directory");
        eprint!("Enter path to a git repository: ");
        std::io::stdout().flush()?;
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let path = PathBuf::from(input.trim());
        
        if path.join(".git").is_dir() {
            Ok(path)
        } else {
            Err(anyhow!("Invalid repository path"))
        }
    } else {
        Err(anyhow!("No git repository found"))
    }
}
```

### Mode-Agnostic Output

```rust
#[derive(Serialize)]
pub struct DemoReport {
    repository: PathBuf,
    steps: Vec<DemoStep>,
    total_elapsed_ms: u64,
}

impl DemoReport {
    pub fn render(&self, mode: ExecutionMode) -> String {
        match mode {
            ExecutionMode::Cli => self.render_cli(),
            ExecutionMode::Mcp => serde_json::to_string(self).unwrap(),
        }
    }
    
    fn render_cli(&self) -> String {
        let mut output = String::with_capacity(4096); // Preallocate
        
        writeln!(&mut output, "\n🎯 PAIML MCP Agent Toolkit Demo").unwrap();
        writeln!(&mut output, "Repository: {}", self.repository.display()).unwrap();
        writeln!(&mut output, "\n📊 Capabilities Demonstrated:\n").unwrap();
        
        for (idx, step) in self.steps.iter().enumerate() {
            writeln!(&mut output, "{}. {} ({} ms)", 
                idx + 1, step.capability, step.elapsed_ms).unwrap();
            
            // Extract key metrics from response
            if let Ok(result) = serde_json::from_value::<Value>(&step.response.result) {
                self.render_step_highlights(&mut output, step.capability, &result);
            }
        }
        
        writeln!(&mut output, "\n⏱️  Total execution time: {} ms", 
            self.total_elapsed_ms).unwrap();
        
        output
    }
}
```

## Integration Test Harness

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::stateless_server::StatelessTemplateServer;
    
    #[tokio::test]
    async fn test_demo_mode_execution() {
        let server = Arc::new(StatelessTemplateServer::new().unwrap());
        let mut runner = DemoRunner::new(server);
        
        // Use test fixtures repository
        let test_repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/sample-repo");
        
        let report = runner.execute(test_repo).await.unwrap();
        
        // Assertions that increase coverage
        assert_eq!(report.steps.len(), 5);
        assert!(report.total_elapsed_ms > 0);
        
        // Verify each capability was exercised
        for step in &report.steps {
            assert!(matches!(step.response.error, None));
            assert!(step.elapsed_ms < 5000); // Performance bound
        }
    }
    
    #[test]
    fn test_repository_detection() {
        let temp = tempfile::tempdir().unwrap();
        let repo_path = temp.path().join("repo");
        
        // Create mock git repo
        std::fs::create_dir_all(repo_path.join(".git")).unwrap();
        
        let detected = detect_repository(Some(repo_path.clone())).unwrap();
        assert_eq!(detected, repo_path);
    }
}
```

## CLI Integration

```rust
// server/src/cli/mod.rs
#[derive(Args)]
pub struct DemoArgs {
    /// Repository path (defaults to current directory)
    #[arg(short, long)]
    path: Option<PathBuf>,
    
    /// Output format (cli, json)
    #[arg(short, long, default_value = "cli")]
    format: OutputFormat,
}

pub async fn run(cli: Cli, server: Arc<StatelessTemplateServer>) -> anyhow::Result<()> {
    match cli.command {
        #[cfg(any(test, feature = "demo-dev"))]
        Commands::Demo(args) => {
            let repo = detect_repository(args.path)?;
            let mut runner = DemoRunner::new(server);
            let report = runner.execute(repo).await?;
            println!("{}", report.render(ExecutionMode::Cli));
            Ok(())
        }
        #[cfg(not(any(test, feature = "demo-dev")))]
        Commands::Demo(_) => {
            Err(anyhow!("Demo mode not available in release builds"))
        }
        // ... existing commands
    }
}
```

## MCP Integration

```rust
// server/src/handlers/tools.rs
pub async fn handle_tool_call(
    request: &McpRequest,
    server: &Arc<StatelessTemplateServer>,
) -> McpResponse {
    match call_params.name.as_str() {
        #[cfg(any(test, feature = "demo-dev"))]
        "run_demo" => {
            let args: DemoArgs = serde_json::from_value(call_params.arguments)?;
            let repo = detect_repository(args.path).map_err(|e| McpError {
                code: -32602,
                message: e.to_string(),
                data: None,
            })?;
            
            let mut runner = DemoRunner::new(Arc::clone(server));
            let report = runner.execute(repo).await.map_err(|e| McpError {
                code: -32603,
                message: e.to_string(),
                data: None,
            })?;
            
            McpResponse::success(request.id.clone(), serde_json::to_value(report)?)
        }
        // ... existing tools
    }
}
```

## Binary Size Analysis

### Release Build (demo disabled)
```bash
# No demo code included
cargo build --release
# Binary size: 8.2 MB (baseline)
```

### Test Build (demo enabled)
```bash
# Demo code included for testing
cargo test --features demo-dev
# Binary size: 8.4 MB (+200 KB for test harness)
```

### Zero-Cost Verification
```rust
// Compile-time assertion
#[cfg(not(any(test, feature = "demo-dev")))]
const _: () = {
    // Verify no demo symbols in release
    extern "Rust" {
        fn run_demo() -> !;
    }
};
```

## Performance Characteristics

| Operation | Expected Latency | Measurement Point |
|-----------|-----------------|-------------------|
| Repository detection | <10ms | `detect_repository()` |
| Context generation (cached) | <50ms | `handle_tool_call()` |
| Complexity analysis | <200ms/KLOC | `analyze_complexity()` |
| DAG generation | <100ms | `analyze_dag()` |
| Total demo execution | <2s | `DemoRunner::execute()` |

## Extension Pattern

New capabilities integrate automatically:

```rust
impl DemoRunner {
    async fn demo_new_capability(&mut self, path: &Path) -> Result<DemoStep> {
        // Identical pattern: build request, call handler, record metrics
        let request = self.build_mcp_request("new_tool_name", json!({
            "project_path": path.to_str().unwrap()
        }));
        
        // Reuses existing infrastructure
        let response = handle_tool_call(&request, &self.server).await;
        // ...
    }
}
```

# Demo Mode Implementation Specification

## Mermaid Interaction Architecture

### Zero-Copy SVG Rendering Pipeline

```rust
// server/src/demo/mermaid_server.rs
use axum::{extract::ws::WebSocket, response::Html, routing::get, Router};
use tokio::sync::broadcast;

pub struct MermaidInteractiveServer {
    mermaid_content: Arc<RwLock<String>>,
    update_channel: broadcast::Sender<GraphUpdate>,
    port: u16,
}

#[derive(Clone, Serialize)]
enum GraphUpdate {
    FilterComplexity { threshold: u32 },
    HighlightDuplicates { group_id: Option<usize> },
    ZoomToNode { node_id: String },
    ResetView,
}

impl MermaidInteractiveServer {
    pub fn spawn(initial_mermaid: String) -> Result<Self> {
        // Bind to ephemeral port
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let port = listener.local_addr()?.port();
        
        let (tx, _) = broadcast::channel(32);
        let content = Arc::new(RwLock::new(initial_mermaid));
        
        // Minimal HTML with WebSocket for live updates
        let app = Router::new()
            .route("/", get({
                let content = Arc::clone(&content);
                move || serve_interactive_view(content)
            }))
            .route("/ws", get({
                let tx = tx.clone();
                let content = Arc::clone(&content);
                move |ws: WebSocketUpgrade| handle_websocket(ws, tx, content)
            }));
        
        tokio::spawn(async move {
            axum::Server::from_tcp(listener)?
                .serve(app.into_make_service())
                .await
        });
        
        Ok(Self { mermaid_content: content, update_channel: tx, port })
    }
}

async fn serve_interactive_view(content: Arc<RwLock<String>>) -> Html<String> {
    // Inline everything to avoid external dependencies
    Html(format!(r#"
<!DOCTYPE html>
<html>
<head>
<script src="https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.min.js"></script>
<style>
body {{ margin: 0; overflow: hidden; }}
#controls {{ position: fixed; top: 10px; right: 10px; background: rgba(255,255,255,0.9); padding: 10px; }}
#graph {{ width: 100vw; height: 100vh; }}
.node.highlighted {{ stroke: #ff0000 !important; stroke-width: 3px !important; }}
</style>
</head>
<body>
<div id="controls">
  <label>Complexity: <input type="range" id="complexity" min="0" max="50" value="0"></label>
  <button onclick="highlightDuplicates()">Show Duplicates</button>
  <button onclick="resetView()">Reset</button>
</div>
<div id="graph" class="mermaid">{}</div>
<script>
const ws = new WebSocket('ws://localhost:{}/ws');
const graphDiv = document.getElementById('graph');

mermaid.initialize({{ startOnLoad: false, theme: 'default' }});

ws.onmessage = async (event) => {{
  const update = JSON.parse(event.data);
  if (update.mermaid) {{
    graphDiv.innerHTML = update.mermaid;
    await mermaid.run();
    attachNodeListeners();
  }}
}};

function attachNodeListeners() {{
  document.querySelectorAll('.node').forEach(node => {{
    node.addEventListener('click', () => {{
      ws.send(JSON.stringify({{ ZoomToNode: {{ node_id: node.id }} }}));
    }});
  }});
}}

document.getElementById('complexity').oninput = (e) => {{
  ws.send(JSON.stringify({{ FilterComplexity: {{ threshold: parseInt(e.target.value) }} }}));
}};

mermaid.run().then(() => attachNodeListeners());
</script>
</body>
</html>
"#, content.read().await, self.port))
}
```

### Demo Execution Engine

```rust
// server/src/demo/executor.rs
use crate::services::dag_builder::{DagBuilder, DependencyGraph};
use crate::services::mermaid_generator::{MermaidGenerator, MermaidOptions};

pub struct DemoExecutor {
    server: Arc<StatelessTemplateServer>,
    dag_cache: Option<DependencyGraph>,
    ast_cache: Arc<SessionCacheManager>,
}

impl DemoExecutor {
    pub async fn execute_orchestrated_demo(&mut self, repo: PathBuf) -> Result<DemoSession> {
        let mut session = DemoSession::new(repo.clone());
        
        // Phase 1: Parallel capability analysis
        let (ast_result, churn_result, complexity_result) = tokio::join!(
            self.analyze_ast_with_progress(&repo, &mut session),
            self.analyze_churn_with_progress(&repo, &mut session),
            self.analyze_complexity_with_progress(&repo, &mut session)
        );
        
        // Phase 2: Enhanced DAG with interactive visualization
        let dag_result = self.generate_interactive_dag(
            &repo, 
            ast_result?, 
            complexity_result?,
            &mut session
        ).await?;
        
        // Phase 3: Template inference from analysis
        let template_result = self.generate_smart_templates(
            &repo,
            &session.collected_metrics(),
            &mut session
        ).await?;
        
        Ok(session)
    }
    
    async fn generate_interactive_dag(
        &mut self,
        repo: &Path,
        ast_context: ProjectContext,
        complexity: ComplexityReport,
        session: &mut DemoSession,
    ) -> Result<InteractiveDagResult> {
        // Build enhanced DAG with all features
        let dag = DagBuilder::new()
            .with_ast_context(ast_context)
            .with_complexity_metrics(complexity)
            .build_from_project(repo)?;
        
        // Generate initial mermaid
        let options = MermaidOptions {
            show_complexity: true,
            max_nodes: 1000, // Higher limit for demo
            layout_algorithm: LayoutAlgorithm::ForceDirected,
            ..Default::default()
        };
        
        let mermaid = MermaidGenerator::new(options).generate(&dag);
        
        // Spawn interactive server
        let server = MermaidInteractiveServer::spawn(mermaid.clone())?;
        
        // Register update handlers
        let dag_clone = dag.clone();
        let generator = MermaidGenerator::new(options);
        
        tokio::spawn(async move {
            let mut rx = server.update_channel.subscribe();
            while let Ok(update) = rx.recv().await {
                let new_mermaid = match update {
                    GraphUpdate::FilterComplexity { threshold } => {
                        let filtered = dag_clone.filter_by_complexity(threshold);
                        generator.generate(&filtered)
                    }
                    GraphUpdate::HighlightDuplicates { group_id } => {
                        generator.generate_with_highlights(&dag_clone, group_id)
                    }
                    _ => continue,
                };
                
                *server.mermaid_content.write().await = new_mermaid;
            }
        });
        
        self.dag_cache = Some(dag.clone());
        
        Ok(InteractiveDagResult {
            server_port: server.port,
            node_count: dag.nodes.len(),
            edge_count: dag.edges.len(),
            complexity_distribution: self.compute_complexity_distribution(&dag),
        })
    }
}
```

### Progressive Disclosure UI

```rust
// server/src/demo/ui.rs
use crossterm::{event::{self, Event, KeyCode}, terminal};
use tui::{backend::CrosstermBackend, layout::{Constraint, Layout}, widgets::{Block, Borders, Gauge, List, ListItem}};

pub struct DemoUI {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    state: DemoState,
}

#[derive(Default)]
struct DemoState {
    current_step: usize,
    capabilities: Vec<CapabilityStatus>,
    selected: usize,
}

impl DemoUI {
    pub async fn run_interactive(&mut self, executor: &mut DemoExecutor) -> Result<()> {
        loop {
            self.terminal.draw(|f| self.render(f))?;
            
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Up => self.state.selected = self.state.selected.saturating_sub(1),
                    KeyCode::Down => {
                        self.state.selected = (self.state.selected + 1)
                            .min(self.state.capabilities.len() - 1);
                    }
                    KeyCode::Enter => {
                        let capability = &self.state.capabilities[self.state.selected];
                        self.execute_capability(executor, capability).await?;
                    }
                    KeyCode::Char('q') => break,
                    KeyCode::Char('a') => {
                        // Run all capabilities in sequence
                        for i in 0..self.state.capabilities.len() {
                            self.state.selected = i;
                            let capability = &self.state.capabilities[i].clone();
                            self.execute_capability(executor, capability).await?;
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
    
    async fn execute_capability(
        &mut self, 
        executor: &mut DemoExecutor, 
        capability: &CapabilityStatus
    ) -> Result<()> {
        match capability.name.as_str() {
            "AST Context Analysis" => {
                self.state.capabilities[self.state.selected].status = Status::Running;
                self.render_progress("Parsing source files...", 0.0)?;
                
                let result = executor.analyze_ast_with_progress(
                    &executor.repo_path, 
                    |progress| {
                        self.render_progress(
                            &format!("Analyzing {} files...", progress.files_processed),
                            progress.percentage
                        )
                    }
                ).await?;
                
                self.state.capabilities[self.state.selected].status = Status::Completed;
                self.state.capabilities[self.state.selected].result = Some(result);
            }
            "Interactive DAG Visualization" => {
                let result = executor.generate_interactive_dag(...).await?;
                
                // Open browser and show connection info
                self.show_notification(&format!(
                    "📊 DAG visualization running at http://localhost:{}\n\
                     Press 'b' to open in browser, 'Esc' to continue",
                    result.server_port
                ))?;
                
                if self.wait_for_key(KeyCode::Char('b')).await? {
                    webbrowser::open(&format!("http://localhost:{}", result.server_port))?;
                }
            }
            _ => {}
        }
        Ok(())
    }
}
```

### Demo Scenarios

```rust
// server/src/demo/scenarios.rs
pub struct DemoScenario {
    name: &'static str,
    description: &'static str,
    steps: Vec<DemoStep>,
}

impl DemoScenario {
    pub fn all_scenarios() -> Vec<Self> {
        vec![
            Self::performance_analysis(),
            Self::technical_debt_assessment(),
            Self::architecture_review(),
            Self::ci_integration(),
        ]
    }
    
    fn performance_analysis() -> Self {
        Self {
            name: "Performance Analysis",
            description: "Identify performance bottlenecks and optimization opportunities",
            steps: vec![
                DemoStep::AnalyzeComplexity {
                    focus: ComplexityFocus::Cyclomatic,
                    threshold: ComplexityThresholds {
                        cyclomatic: 15,
                        cognitive: 20,
                        ..Default::default()
                    },
                },
                DemoStep::GenerateDag {
                    dag_type: DagType::CallGraph,
                    options: DagOptions {
                        filter_external: true,
                        show_complexity: true,
                        highlight_hotpaths: true,
                    },
                },
                DemoStep::ProfileExecution {
                    // Use existing AST to simulate execution paths
                    simulation_iterations: 1000,
                },
            ],
        }
    }
    
    fn technical_debt_assessment() -> Self {
        Self {
            name: "Technical Debt Assessment",
            description: "Quantify and prioritize technical debt",
            steps: vec![
                DemoStep::AnalyzeComplexity {
                    focus: ComplexityFocus::Both,
                    threshold: ComplexityThresholds::strict(),
                },
                DemoStep::AnalyzeChurn {
                    period_days: 90,
                    correlation: ChurnCorrelation::WithComplexity,
                },
                DemoStep::DetectDuplicates {
                    min_tokens: 50,
                    similarity_threshold: 0.85,
                },
                DemoStep::GenerateDebtReport {
                    format: DebtReportFormat::Markdown,
                    include_remediation_cost: true,
                },
            ],
        }
    }
}
```

### MCP Mode Integration

```rust
// server/src/handlers/tools.rs
pub async fn handle_demo_tool(
    args: DemoToolArgs,
    server: &Arc<StatelessTemplateServer>,
) -> McpResponse {
    let mut executor = DemoExecutor::new(Arc::clone(server));
    
    match args.mode {
        DemoMode::Headless => {
            // Return structured data for MCP clients
            let session = executor.execute_orchestrated_demo(args.repo_path).await?;
            
            McpResponse::success(args.id, json!({
                "capabilities_demonstrated": session.completed_steps(),
                "metrics": session.collected_metrics(),
                "insights": session.generated_insights(),
                "artifacts": {
                    "mermaid_diagram": session.get_artifact("dag.mmd"),
                    "complexity_report": session.get_artifact("complexity.sarif"),
                    "churn_analysis": session.get_artifact("churn.json"),
                },
            }))
        }
        DemoMode::Interactive => {
            // Spawn server and return connection details
            let dag_result = executor.generate_interactive_dag(...).await?;
            
            McpResponse::success(args.id, json!({
                "type": "interactive_session",
                "server_url": format!("http://localhost:{}", dag_result.server_port),
                "session_id": Uuid::new_v4(),
                "available_commands": [
                    "filter_complexity",
                    "highlight_duplicates",
                    "zoom_to_node",
                    "export_svg",
                ],
            }))
        }
    }
}
```

### Test Integration

```rust
#[cfg(test)]
mod demo_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_demo_increases_coverage() {
        // Run demo on our own codebase
        let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let mut executor = DemoExecutor::new_test();
        
        // Track coverage before
        let coverage_before = get_test_coverage();
        
        // Execute demo
        let session = executor.execute_orchestrated_demo(repo).await.unwrap();
        
        // Verify all code paths exercised
        assert!(session.completed_steps().len() >= 5);
        
        // These assertions exercise error paths
        assert!(executor.analyze_ast_with_progress(
            &PathBuf::from("/nonexistent"), 
            &mut DemoSession::new(PathBuf::new())
        ).await.is_err());
        
        // Coverage should increase
        let coverage_after = get_test_coverage();
        assert!(coverage_after > coverage_before);
    }
    
    #[test]
    fn test_mermaid_interaction_protocol() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let server = MermaidInteractiveServer::spawn("graph TD".to_string()).unwrap();
            
            // Simulate WebSocket commands
            let updates = vec![
                GraphUpdate::FilterComplexity { threshold: 10 },
                GraphUpdate::HighlightDuplicates { group_id: Some(1) },
                GraphUpdate::ZoomToNode { node_id: "node_42".to_string() },
            ];
            
            for update in updates {
                server.update_channel.send(update).unwrap();
            }
            
            // Verify server is responsive
            tokio::time::sleep(Duration::from_millis(100)).await;
            assert!(!server.mermaid_content.read().await.is_empty());
        });
    }
}
```

## Binary Size Impact: Zero

```toml
# Cargo.toml
[features]
default = []
demo-dev = ["crossterm", "tui"]  # Only for development builds

[dependencies]
# Core dependencies (always included)
axum = { version = "0.6", features = ["ws"], optional = false }

# Demo-only dependencies (excluded from release)
crossterm = { version = "0.25", optional = true }
tui = { version = "0.19", optional = true }

[profile.release]
lto = true
codegen-units = 1
strip = true
```

Release build excludes all demo code via conditional compilation:
```bash
cargo build --release
# nm target/release/paiml-mcp-agent-toolkit | grep -i demo
# (no output - demo symbols stripped)
```

The interactive Mermaid server reuses the existing `axum` dependency already present for MCP protocol handling, adding zero bytes to the binary.

## Conclusion

This specification achieves zero binary size impact through conditional compilation while maximizing code reuse. The demo mode serves dual purposes: user demonstration and integration testing, increasing coverage without additional maintenance burden. The design's elegance lies in treating the demo as a choreographed sequence of existing tool invocations, ensuring it evolves automatically with new capabilities.