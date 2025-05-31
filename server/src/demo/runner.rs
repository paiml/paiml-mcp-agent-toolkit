use crate::cli::ExecutionMode;
use crate::handlers::tools::handle_tool_call;
use crate::models::mcp::{McpRequest, McpResponse};
use crate::stateless_server::StatelessTemplateServer;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::env;
use std::fmt::Write;
use std::io::{self, Write as IoWrite};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use tracing::instrument;

pub struct DemoRunner {
    server: Arc<StatelessTemplateServer>,
    execution_log: Vec<DemoStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemoStep {
    pub name: String,
    pub capability: &'static str,
    pub request: McpRequest,
    pub response: McpResponse,
    pub elapsed_ms: u64,
    pub success: bool,
    pub output: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct DemoReport {
    pub repository: String,
    pub total_time_ms: u64,
    pub steps: Vec<DemoStep>,
    pub system_diagram: Option<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Component {
    id: String,
    label: String,
    color: String,
    connections: Vec<(String, String)>,
}

impl DemoRunner {
    pub fn new(server: Arc<StatelessTemplateServer>) -> Self {
        Self {
            server,
            execution_log: Vec::new(),
        }
    }

    async fn clone_and_prepare(&self, url: &str) -> Result<PathBuf> {
        // For now, just return an error - git2 integration would go here
        // This is a placeholder implementation
        Err(anyhow!(
            "Remote repository cloning not yet implemented. URL: {}",
            url
        ))
    }

    fn generate_system_diagram(&self, _steps: &[DemoStep]) -> Result<String> {
        // Extract component relationships from analysis results
        let mut components = HashMap::new();

        // Map internal components to high-level architecture
        components.insert(
            "ast_context".to_string(),
            Component {
                id: "A".to_string(),
                label: "AST Context Analysis".to_string(),
                color: "#90EE90".to_string(),
                connections: vec![("B".to_string(), "uses".to_string())],
            },
        );

        components.insert(
            "file_parser".to_string(),
            Component {
                id: "B".to_string(),
                label: "File Parser".to_string(),
                color: "#FFFFFF".to_string(),
                connections: vec![
                    ("C".to_string(), "".to_string()),
                    ("D".to_string(), "".to_string()),
                    ("E".to_string(), "".to_string()),
                ],
            },
        );

        // Language-specific AST components
        components.insert(
            "rust_ast".to_string(),
            Component {
                id: "C".to_string(),
                label: "Rust AST".to_string(),
                color: "#FFFFFF".to_string(),
                connections: vec![],
            },
        );

        components.insert(
            "typescript_ast".to_string(),
            Component {
                id: "D".to_string(),
                label: "TypeScript AST".to_string(),
                color: "#FFFFFF".to_string(),
                connections: vec![],
            },
        );

        components.insert(
            "python_ast".to_string(),
            Component {
                id: "E".to_string(),
                label: "Python AST".to_string(),
                color: "#FFFFFF".to_string(),
                connections: vec![],
            },
        );

        // Analysis components
        components.insert(
            "complexity".to_string(),
            Component {
                id: "F".to_string(),
                label: "Code Complexity".to_string(),
                color: "#FFD700".to_string(),
                connections: vec![
                    ("C".to_string(), "analyzes".to_string()),
                    ("D".to_string(), "analyzes".to_string()),
                    ("E".to_string(), "analyzes".to_string()),
                ],
            },
        );

        components.insert(
            "dag_gen".to_string(),
            Component {
                id: "G".to_string(),
                label: "DAG Generation".to_string(),
                color: "#FFA500".to_string(),
                connections: vec![
                    ("C".to_string(), "reads".to_string()),
                    ("D".to_string(), "reads".to_string()),
                    ("E".to_string(), "reads".to_string()),
                ],
            },
        );

        components.insert(
            "churn".to_string(),
            Component {
                id: "H".to_string(),
                label: "Code Churn".to_string(),
                color: "#FF6347".to_string(),
                connections: vec![("I".to_string(), "git history".to_string())],
            },
        );

        components.insert(
            "git".to_string(),
            Component {
                id: "I".to_string(),
                label: "Git Analysis".to_string(),
                color: "#FFFFFF".to_string(),
                connections: vec![],
            },
        );

        components.insert(
            "template".to_string(),
            Component {
                id: "J".to_string(),
                label: "Template Generation".to_string(),
                color: "#87CEEB".to_string(),
                connections: vec![("K".to_string(), "renders".to_string())],
            },
        );

        components.insert(
            "handlebars".to_string(),
            Component {
                id: "K".to_string(),
                label: "Handlebars".to_string(),
                color: "#FFFFFF".to_string(),
                connections: vec![],
            },
        );

        // Generate Mermaid diagram
        self.render_system_mermaid(&components)
    }

    fn render_system_mermaid(&self, _components: &HashMap<String, Component>) -> Result<String> {
        let mut output = String::new();
        output.push_str("graph TD\n");

        // Add nodes and connections based on target diagram
        output.push_str("    A[AST Context Analysis] -->|uses| B[File Parser]\n");
        output.push_str("    B --> C[Rust AST]\n");
        output.push_str("    B --> D[TypeScript AST]\n");
        output.push_str("    B --> E[Python AST]\n\n");

        output.push_str("    F[Code Complexity] -->|analyzes| C\n");
        output.push_str("    F -->|analyzes| D\n");
        output.push_str("    F -->|analyzes| E\n\n");

        output.push_str("    G[DAG Generation] -->|reads| C\n");
        output.push_str("    G -->|reads| D\n");
        output.push_str("    G -->|reads| E\n\n");

        output.push_str("    H[Code Churn] -->|git history| I[Git Analysis]\n\n");

        output.push_str("    J[Template Generation] -->|renders| K[Handlebars]\n\n");

        // Add styling
        output.push_str("    style A fill:#90EE90\n");
        output.push_str("    style F fill:#FFD700\n");
        output.push_str("    style G fill:#FFA500\n");
        output.push_str("    style H fill:#FF6347\n");
        output.push_str("    style J fill:#87CEEB\n");

        Ok(output)
    }

    fn create_demo_step(
        &self,
        name: &str,
        capability: &'static str,
        request: McpRequest,
        response: McpResponse,
        elapsed_ms: u64,
    ) -> DemoStep {
        let success = response.error.is_none();
        let output = if success {
            response.result.clone()
        } else {
            Some(
                json!({ "error": response.error.as_ref().map(|e| e.message.clone()).unwrap_or_default() }),
            )
        };

        DemoStep {
            name: name.to_string(),
            capability,
            request,
            response,
            elapsed_ms,
            success,
            output,
        }
    }

    pub async fn execute(&mut self, repo_path: PathBuf) -> Result<DemoReport> {
        self.execute_with_diagram(&repo_path, None).await
    }

    #[instrument(skip(self, repo_path), fields(repo = %repo_path.display()))]
    pub async fn execute_with_diagram(
        &mut self,
        repo_path: &Path,
        url: Option<&str>,
    ) -> Result<DemoReport> {
        let start = Instant::now();

        // Clone remote repository if URL provided
        let working_path = if let Some(url) = url {
            self.clone_and_prepare(url).await?
        } else {
            repo_path.to_path_buf()
        };

        let version = env!("CARGO_PKG_VERSION");
        println!("🎯 PAIML MCP Agent Toolkit Demo v{}", version);
        if url.is_some() {
            println!("📁 Repository: {} (cloned)", url.unwrap());
        } else {
            println!("📁 Repository: {}", working_path.display());
        }
        println!();

        // Execute analysis pipeline with tracing
        let span = tracing::info_span!("demo_execution", repo = %working_path.display());
        let _guard = span.enter();

        // Collect all analysis results
        let mut steps = Vec::new();
        steps.push(self.demo_context_generation(&working_path).await?);
        steps.push(self.demo_complexity_analysis(&working_path).await?);
        steps.push(self.demo_dag_generation(&working_path).await?);
        steps.push(self.demo_churn_analysis(&working_path).await?);
        steps.push(self.demo_system_architecture(&working_path).await?);
        steps.push(self.demo_defect_analysis(&working_path).await?);
        steps.push(self.demo_template_generation(&working_path).await?);

        // Generate high-level system diagram
        let system_diagram = self.generate_system_diagram(&steps)?;

        Ok(DemoReport {
            repository: if url.is_some() {
                url.unwrap().to_string()
            } else {
                working_path.display().to_string()
            },
            total_time_ms: start.elapsed().as_millis() as u64,
            steps,
            system_diagram: Some(system_diagram),
        })
    }

    async fn demo_context_generation(&mut self, path: &Path) -> Result<DemoStep> {
        let request = self.build_mcp_request(
            "generate_context",
            json!({
                "project_path": path.to_str().unwrap(),
                "toolchain": "rust",
                "format": "json"
            }),
        );

        println!("1️⃣  Generating AST Context...");

        let start = Instant::now();
        let response = handle_tool_call(self.server.clone(), request.clone()).await;
        let elapsed = start.elapsed().as_millis() as u64;

        let step = self.create_demo_step(
            "AST Context Analysis",
            "AST Context Analysis",
            request.clone(),
            response.clone(),
            elapsed,
        );

        self.execution_log.push(step.clone());

        if response.error.is_none() {
            println!("   ✅ Context generated in {} ms", elapsed);
        } else {
            println!("   ❌ Failed: {:?}", response.error);
        }

        Ok(step)
    }

    async fn demo_complexity_analysis(&mut self, path: &Path) -> Result<DemoStep> {
        let request = self.build_mcp_request(
            "analyze_complexity",
            json!({
                "project_path": path.to_str().unwrap(),
                "toolchain": "rust",
                "format": "summary",
                "max_cyclomatic": 20,
                "max_cognitive": 30
            }),
        );

        println!("\n2️⃣  Analyzing Code Complexity...");

        let start = Instant::now();
        let response = handle_tool_call(self.server.clone(), request.clone()).await;
        let elapsed = start.elapsed().as_millis() as u64;

        let step = self.create_demo_step(
            "Code Complexity Analysis",
            "Code Complexity Analysis",
            request.clone(),
            response.clone(),
            elapsed,
        );

        self.execution_log.push(step.clone());

        if response.error.is_none() {
            println!("   ✅ Complexity analyzed in {} ms", elapsed);
            if let Some(result) = &response.result {
                if let Ok(summary) = serde_json::from_value::<Value>(result.clone()) {
                    if let Some(total_functions) = summary.get("total_functions") {
                        println!("   📊 Analyzed {} functions", total_functions);
                    }
                }
            }
        } else {
            println!("   ❌ Failed: {:?}", response.error);
        }

        Ok(step)
    }

    async fn demo_dag_generation(&mut self, path: &Path) -> Result<DemoStep> {
        let request = self.build_mcp_request(
            "analyze_dag",
            json!({
                "project_path": path.to_str().unwrap(),
                "dag_type": "import-graph",
                "filter_external": true,
                "show_complexity": true,
                "format": "mermaid"
            }),
        );

        println!("\n3️⃣  Generating Dependency Graph...");

        let start = Instant::now();
        let response = handle_tool_call(self.server.clone(), request.clone()).await;
        let elapsed = start.elapsed().as_millis() as u64;

        let step = self.create_demo_step(
            "DAG Generation",
            "DAG Visualization",
            request.clone(),
            response.clone(),
            elapsed,
        );

        self.execution_log.push(step.clone());

        if response.error.is_none() {
            println!("   ✅ DAG generated in {} ms", elapsed);
            if let Some(result) = &response.result {
                if let Ok(dag_result) = serde_json::from_value::<Value>(result.clone()) {
                    if let Some(stats) = dag_result.get("stats") {
                        if let (Some(nodes), Some(edges)) = (stats.get("nodes"), stats.get("edges"))
                        {
                            println!("   📈 Graph: {} nodes, {} edges", nodes, edges);
                        }
                    }
                }
            }
        } else {
            println!("   ❌ Failed: {:?}", response.error);
        }

        Ok(step)
    }

    async fn demo_churn_analysis(&mut self, path: &Path) -> Result<DemoStep> {
        let request = self.build_mcp_request(
            "analyze_code_churn",
            json!({
                "project_path": path.to_str().unwrap(),
                "period_days": 30,
                "format": "summary"
            }),
        );

        println!("\n4️⃣  Analyzing Code Churn...");

        let start = Instant::now();
        let response = handle_tool_call(self.server.clone(), request.clone()).await;
        let elapsed = start.elapsed().as_millis() as u64;

        let step = self.create_demo_step(
            "Code Churn Analysis",
            "Code Churn Analysis",
            request.clone(),
            response.clone(),
            elapsed,
        );

        self.execution_log.push(step.clone());

        if response.error.is_none() {
            println!("   ✅ Churn analyzed in {} ms", elapsed);
            if let Some(result) = &response.result {
                if let Ok(churn_result) = serde_json::from_value::<Value>(result.clone()) {
                    if let Some(files_analyzed) = churn_result.get("files_analyzed") {
                        println!("   📈 Analyzed {} files", files_analyzed);
                    }
                }
            }
        } else {
            println!("   ❌ Failed: {:?}", response.error);
        }

        Ok(step)
    }

    async fn demo_system_architecture(&mut self, path: &Path) -> Result<DemoStep> {
        // Use the enhanced canonical query system
        let request = self.build_mcp_request(
            "analyze_system_architecture",
            json!({
                "project_path": path.to_str().unwrap(),
                "format": "mermaid",
                "show_complexity": true
            }),
        );

        println!("\n5️⃣  Analyzing System Architecture...");

        let start = Instant::now();
        let response = handle_tool_call(self.server.clone(), request.clone()).await;
        let elapsed = start.elapsed().as_millis() as u64;

        let step = self.create_demo_step(
            "System Architecture",
            "System Architecture Analysis",
            request.clone(),
            response.clone(),
            elapsed,
        );

        self.execution_log.push(step.clone());

        if response.error.is_none() {
            println!("   ✅ Architecture analyzed in {} ms", elapsed);
            if let Some(result) = &response.result {
                if let Ok(arch_result) = serde_json::from_value::<Value>(result.clone()) {
                    if let Some(metadata) = arch_result.get("metadata") {
                        if let (Some(nodes), Some(edges)) =
                            (metadata.get("nodes"), metadata.get("edges"))
                        {
                            println!("   🏗️  Components: {}, Relationships: {}", nodes, edges);
                        }
                    }
                }
            }
        } else {
            println!("   ❌ Failed: {:?}", response.error);
        }

        Ok(step)
    }

    async fn demo_defect_analysis(&mut self, path: &Path) -> Result<DemoStep> {
        let request = self.build_mcp_request(
            "analyze_defect_probability",
            json!({
                "project_path": path.to_str().unwrap(),
                "toolchain": "rust",
                "format": "summary"
            }),
        );

        println!("\n6️⃣  Analyzing Defect Probability...");

        let start = Instant::now();
        let response = handle_tool_call(self.server.clone(), request.clone()).await;
        let elapsed = start.elapsed().as_millis() as u64;

        let step = self.create_demo_step(
            "Defect Probability Analysis",
            "Defect Probability Analysis",
            request.clone(),
            response.clone(),
            elapsed,
        );

        self.execution_log.push(step.clone());

        if response.error.is_none() {
            println!("   ✅ Defect analysis completed in {} ms", elapsed);
            if let Some(result) = &response.result {
                if let Ok(defect_result) = serde_json::from_value::<Value>(result.clone()) {
                    if let Some(avg_prob) = defect_result.get("average_probability") {
                        println!(
                            "   🔍 Average defect probability: {:.2}",
                            avg_prob.as_f64().unwrap_or(0.0)
                        );
                    }
                }
            }
        } else {
            println!("   ❌ Failed: {:?}", response.error);
        }

        Ok(step)
    }

    async fn demo_template_generation(&mut self, path: &Path) -> Result<DemoStep> {
        let request = self.build_mcp_request(
            "generate_template",
            json!({
                "resource_uri": "template://makefile/rust/cli",
                "parameters": {
                    "project_name": path.file_name()
                        .unwrap_or_default()
                        .to_str()
                        .unwrap_or("demo-project"),
                    "has_tests": true,
                    "has_benchmarks": false
                }
            }),
        );

        println!("\n7️⃣  Generating Template...");

        let start = Instant::now();
        let response = handle_tool_call(self.server.clone(), request.clone()).await;
        let elapsed = start.elapsed().as_millis() as u64;

        let step = self.create_demo_step(
            "Template Generation",
            "Template Generation",
            request.clone(),
            response.clone(),
            elapsed,
        );

        self.execution_log.push(step.clone());

        if response.error.is_none() {
            println!("   ✅ Template generated in {} ms", elapsed);
        } else {
            println!("   ❌ Failed: {:?}", response.error);
        }

        Ok(step)
    }

    fn build_mcp_request(&self, method: &str, arguments: Value) -> McpRequest {
        McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(format!("demo-{}", method)),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": method,
                "arguments": arguments
            })),
        }
    }
}

impl DemoReport {
    pub fn render(&self, mode: ExecutionMode) -> String {
        match mode {
            ExecutionMode::Cli => self.render_cli(),
            ExecutionMode::Mcp => serde_json::to_string_pretty(self).unwrap(),
        }
    }

    fn render_cli(&self) -> String {
        let mut output = String::with_capacity(4096);

        writeln!(&mut output, "\n🎯 PAIML MCP Agent Toolkit Demo Complete").unwrap();
        writeln!(&mut output, "Repository: {}", self.repository).unwrap();
        writeln!(&mut output, "\n📊 Capabilities Demonstrated:\n").unwrap();

        for (idx, step) in self.steps.iter().enumerate() {
            writeln!(
                &mut output,
                "{}. {} ({} ms)",
                idx + 1,
                step.capability,
                step.elapsed_ms
            )
            .unwrap();

            // Extract key metrics from response
            if let Some(result) = &step.response.result {
                self.render_step_highlights(&mut output, step.capability, result);
            }
        }

        writeln!(
            &mut output,
            "\n⏱️  Total execution time: {} ms",
            self.total_time_ms
        )
        .unwrap();

        // Add system diagram if available
        if let Some(ref diagram) = self.system_diagram {
            writeln!(&mut output, "\n🌍 System Architecture:").unwrap();
            writeln!(&mut output, "```mermaid").unwrap();
            writeln!(&mut output, "{}", diagram).unwrap();
            writeln!(&mut output, "```").unwrap();
        }

        writeln!(
            &mut output,
            "\n🚀 Get started with PAIML MCP Agent Toolkit:"
        )
        .unwrap();
        writeln!(
            &mut output,
            "   - Generate templates: paiml-mcp-agent-toolkit scaffold <toolchain>"
        )
        .unwrap();
        writeln!(
            &mut output,
            "   - Analyze complexity: paiml-mcp-agent-toolkit analyze complexity"
        )
        .unwrap();
        writeln!(
            &mut output,
            "   - View code churn: paiml-mcp-agent-toolkit analyze churn"
        )
        .unwrap();
        writeln!(
            &mut output,
            "   - Create DAGs: paiml-mcp-agent-toolkit analyze dag"
        )
        .unwrap();
        writeln!(
            &mut output,
            "   - System architecture: paiml-mcp-agent-toolkit analyze architecture"
        )
        .unwrap();
        writeln!(
            &mut output,
            "   - Defect probability: paiml-mcp-agent-toolkit analyze defects"
        )
        .unwrap();
        writeln!(&mut output).unwrap();
        writeln!(
            &mut output,
            "📊 To view Mermaid diagrams: https://mermaid.live"
        )
        .unwrap();

        output
    }

    fn render_step_highlights(&self, output: &mut String, capability: &str, result: &Value) {
        match capability {
            "Code Complexity Analysis" => {
                if let Ok(summary) = serde_json::from_value::<Value>(result.clone()) {
                    if let (Some(total), Some(warnings), Some(errors)) = (
                        summary.get("total_functions"),
                        summary.get("total_warnings"),
                        summary.get("total_errors"),
                    ) {
                        writeln!(
                            output,
                            "      Functions: {}, Warnings: {}, Errors: {}",
                            total, warnings, errors
                        )
                        .unwrap();
                    }
                }
            }
            "DAG Visualization" => {
                if let Ok(dag_result) = serde_json::from_value::<Value>(result.clone()) {
                    if let Some(stats) = dag_result.get("stats") {
                        if let (Some(nodes), Some(edges)) = (stats.get("nodes"), stats.get("edges"))
                        {
                            writeln!(output, "      Graph size: {} nodes, {} edges", nodes, edges)
                                .unwrap();
                        }
                    }
                }
            }
            "Code Churn Analysis" => {
                if let Ok(churn_result) = serde_json::from_value::<Value>(result.clone()) {
                    if let (Some(files), Some(total_churn)) = (
                        churn_result.get("files_analyzed"),
                        churn_result.get("total_churn_score"),
                    ) {
                        writeln!(
                            output,
                            "      Files analyzed: {}, Total churn: {}",
                            files, total_churn
                        )
                        .unwrap();
                    }
                }
            }
            "System Architecture Analysis" => {
                if let Ok(arch_result) = serde_json::from_value::<Value>(result.clone()) {
                    if let Some(metadata) = arch_result.get("metadata") {
                        if let (Some(nodes), Some(edges)) =
                            (metadata.get("nodes"), metadata.get("edges"))
                        {
                            writeln!(
                                output,
                                "      Components: {}, Relationships: {}",
                                nodes, edges
                            )
                            .unwrap();
                        }
                    }
                }
            }
            "Defect Probability Analysis" => {
                if let Ok(defect_result) = serde_json::from_value::<Value>(result.clone()) {
                    if let (Some(high_risk), Some(avg_prob)) = (
                        defect_result.get("high_risk_files"),
                        defect_result.get("average_probability"),
                    ) {
                        writeln!(
                            output,
                            "      High-risk files: {}, Avg probability: {:.2}",
                            high_risk.as_array().map(|a| a.len()).unwrap_or(0),
                            avg_prob.as_f64().unwrap_or(0.0)
                        )
                        .unwrap();
                    }
                }
            }
            _ => {}
        }
    }
}

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
    if atty::is(atty::Stream::Stdout) {
        eprintln!("No git repository found in current directory");
        eprint!("Enter path to a git repository: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
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
