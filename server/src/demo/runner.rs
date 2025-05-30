use crate::cli::ExecutionMode;
use crate::handlers::tools::handle_tool_call;
use crate::models::mcp::{McpRequest, McpResponse};
use crate::stateless_server::StatelessTemplateServer;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use std::fmt::Write;
use std::io::{self, Write as IoWrite};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

pub struct DemoRunner {
    server: Arc<StatelessTemplateServer>,
    execution_log: Vec<DemoStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemoStep {
    pub capability: &'static str,
    pub request: McpRequest,
    pub response: McpResponse,
    pub elapsed_ms: u64,
}

#[derive(Debug, Serialize)]
pub struct DemoReport {
    pub repository: PathBuf,
    pub steps: Vec<DemoStep>,
    pub total_elapsed_ms: u64,
}

impl DemoRunner {
    pub fn new(server: Arc<StatelessTemplateServer>) -> Self {
        Self {
            server,
            execution_log: Vec::new(),
        }
    }

    pub async fn execute(&mut self, repo_path: PathBuf) -> Result<DemoReport> {
        println!("🎯 Starting PAIML MCP Agent Toolkit Demo");
        println!("📁 Repository: {}", repo_path.display());
        println!();

        // Execute demo steps
        let steps = vec![
            self.demo_context_generation(&repo_path).await?,
            self.demo_complexity_analysis(&repo_path).await?,
            self.demo_dag_generation(&repo_path).await?,
            self.demo_churn_analysis(&repo_path).await?,
            self.demo_template_generation(&repo_path).await?,
        ];

        let total_elapsed_ms = self.execution_log.iter().map(|s| s.elapsed_ms).sum();

        Ok(DemoReport {
            repository: repo_path,
            steps,
            total_elapsed_ms,
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

        let step = DemoStep {
            capability: "AST Context Analysis",
            request: request.clone(),
            response: response.clone(),
            elapsed_ms: elapsed,
        };

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

        let step = DemoStep {
            capability: "Code Complexity Analysis",
            request: request.clone(),
            response: response.clone(),
            elapsed_ms: elapsed,
        };

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
                "dag_type": "call-graph",
                "filter_external": true,
                "show_complexity": true,
                "format": "mermaid"
            }),
        );

        println!("\n3️⃣  Generating Dependency Graph...");

        let start = Instant::now();
        let response = handle_tool_call(self.server.clone(), request.clone()).await;
        let elapsed = start.elapsed().as_millis() as u64;

        let step = DemoStep {
            capability: "DAG Visualization",
            request: request.clone(),
            response: response.clone(),
            elapsed_ms: elapsed,
        };

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

        let step = DemoStep {
            capability: "Code Churn Analysis",
            request: request.clone(),
            response: response.clone(),
            elapsed_ms: elapsed,
        };

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

        println!("\n5️⃣  Generating Template...");

        let start = Instant::now();
        let response = handle_tool_call(self.server.clone(), request.clone()).await;
        let elapsed = start.elapsed().as_millis() as u64;

        let step = DemoStep {
            capability: "Template Generation",
            request: request.clone(),
            response: response.clone(),
            elapsed_ms: elapsed,
        };

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
        writeln!(&mut output, "Repository: {}", self.repository.display()).unwrap();
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
            self.total_elapsed_ms
        )
        .unwrap();

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
