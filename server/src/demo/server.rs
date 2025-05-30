#[cfg(not(feature = "no-demo"))]
use crate::demo::assets::{decompress_asset, get_asset};
use crate::models::dag::DependencyGraph;
use crate::services::mermaid_generator::{MermaidGenerator, MermaidOptions};
use anyhow::Result;
use bytes::Bytes;
use dashmap::DashMap;
use http::{Response, StatusCode};
use serde::Serialize;
use std::sync::Arc;

// Import the validated HTML template
#[cfg(not(feature = "no-demo"))]
use super::templates::HTML_TEMPLATE;
#[allow(unused_imports)]
use super::templates::CSS_DARK_THEME;

#[cfg(not(feature = "no-demo"))]
use parking_lot::RwLock;

#[cfg(not(feature = "no-demo"))]
use bytes::BytesMut;
#[cfg(not(feature = "no-demo"))]
use tokio::io::{AsyncReadExt, AsyncWriteExt};
#[cfg(not(feature = "no-demo"))]
use tokio::net::{TcpListener, TcpStream};
#[cfg(not(feature = "no-demo"))]
use tokio::sync::Semaphore;

#[derive(Debug, Clone, Serialize)]
pub struct DemoContent {
    pub mermaid_diagram: String,
    pub files_analyzed: usize,
    pub avg_complexity: f64,
    pub tech_debt_hours: u32,
    pub hotspots: Vec<Hotspot>,
    pub ast_time_ms: u64,
    pub complexity_time_ms: u64,
    pub churn_time_ms: u64,
    pub dag_time_ms: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Hotspot {
    pub file: String,
    pub complexity: u32,
    pub churn_score: u32,
}

#[derive(Clone)]
pub struct DemoState {
    pub repository: std::path::PathBuf,
    pub analysis_results: AnalysisResults,
    pub mermaid_cache: Arc<DashMap<u64, String>>,
}

#[derive(Clone, Serialize)]
pub struct AnalysisResults {
    pub files_analyzed: usize,
    pub avg_complexity: f64,
    pub tech_debt_hours: u32,
    pub complexity_report: crate::services::complexity::ComplexityReport,
    pub churn_analysis: crate::models::churn::CodeChurnAnalysis,
    pub dependency_graph: DependencyGraph,
}

pub struct LocalDemoServer {
    port: u16,
    shutdown_tx: tokio::sync::oneshot::Sender<()>,
}

impl LocalDemoServer {
    #[cfg(not(feature = "no-demo"))]
    pub async fn spawn(initial_content: DemoContent) -> Result<(Self, u16)> {
        // Bind to ephemeral port
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();

        let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel();

        // Create initial state
        let state = Arc::new(RwLock::new(DemoState {
            repository: std::path::PathBuf::from("."),
            analysis_results: AnalysisResults {
                files_analyzed: initial_content.files_analyzed,
                avg_complexity: initial_content.avg_complexity,
                tech_debt_hours: initial_content.tech_debt_hours,
                complexity_report: Default::default(),
                churn_analysis: Default::default(),
                dependency_graph: Default::default(),
            },
            mermaid_cache: Arc::new(DashMap::new()),
        }));

        // Spawn accept loop with bounded concurrency
        tokio::spawn(async move {
            let semaphore = Arc::new(Semaphore::new(100)); // Max 100 concurrent connections

            loop {
                tokio::select! {
                    accept_result = listener.accept() => {
                        if let Ok((stream, _)) = accept_result {
                            let permit = semaphore.clone().acquire_owned().await.unwrap();
                            let state = Arc::clone(&state);

                            tokio::spawn(async move {
                                if let Err(e) = handle_connection(stream, state).await {
                                    eprintln!("Connection error: {}", e);
                                }
                                drop(permit);
                            });
                        }
                    }
                    _ = &mut shutdown_rx => {
                        break;
                    }
                }
            }
        });

        Ok((Self { port, shutdown_tx }, port))
    }

    #[cfg(feature = "no-demo")]
    pub async fn spawn(_initial_content: DemoContent) -> Result<(Self, u16)> {
        anyhow::bail!("Demo mode not available. Build without --features no-demo")
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn shutdown(self) {
        let _ = self.shutdown_tx.send(());
    }
}

#[cfg(not(feature = "no-demo"))]
async fn handle_connection(mut stream: TcpStream, state: Arc<RwLock<DemoState>>) -> Result<()> {
    let mut buffer = BytesMut::with_capacity(4096);
    stream.read_buf(&mut buffer).await?;

    // Parse HTTP request (minimal parser for demo)
    let request = parse_minimal_request(&buffer)?;

    let response = match request.path.as_str() {
        "/" => serve_dashboard(&state),
        "/api/summary" => serve_summary_json(&state),
        "/api/metrics" => serve_metrics_json(&state),
        "/api/hotspots" => serve_hotspots_table(&state),
        "/api/dag" => serve_dag_mermaid(&state),
        path if path.starts_with("/vendor/") || path.starts_with("/demo.") => {
            serve_static_asset(path)
        }
        _ => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Bytes::from_static(b"404 Not Found"))?,
    };

    // Write response with zero-copy
    let response_bytes = serialize_response(response);
    stream.write_all(&response_bytes).await?;
    stream.flush().await?;

    Ok(())
}

#[cfg(not(feature = "no-demo"))]
#[derive(Debug)]
struct MinimalRequest {
    path: String,
}

#[cfg(not(feature = "no-demo"))]
fn parse_minimal_request(buffer: &[u8]) -> Result<MinimalRequest> {
    let request_str = std::str::from_utf8(buffer)?;
    let first_line = request_str
        .lines()
        .next()
        .ok_or_else(|| anyhow::anyhow!("Empty request"))?;

    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 2 {
        return Err(anyhow::anyhow!("Invalid request line"));
    }

    Ok(MinimalRequest {
        path: parts[1].to_string(),
    })
}

#[cfg(not(feature = "no-demo"))]
fn serialize_response(response: Response<Bytes>) -> Vec<u8> {
    let mut output = Vec::new();

    // Status line
    output.extend_from_slice(b"HTTP/1.1 ");
    output.extend_from_slice(response.status().as_str().as_bytes());
    output.extend_from_slice(b" ");
    output.extend_from_slice(
        response
            .status()
            .canonical_reason()
            .unwrap_or("Unknown")
            .as_bytes(),
    );
    output.extend_from_slice(b"\r\n");

    // Headers
    for (name, value) in response.headers() {
        output.extend_from_slice(name.as_str().as_bytes());
        output.extend_from_slice(b": ");
        output.extend_from_slice(value.as_bytes());
        output.extend_from_slice(b"\r\n");
    }

    // Content-Length
    output.extend_from_slice(b"Content-Length: ");
    output.extend_from_slice(response.body().len().to_string().as_bytes());
    output.extend_from_slice(b"\r\n");

    // End of headers
    output.extend_from_slice(b"\r\n");

    // Body
    output.extend_from_slice(response.body());

    output
}

#[cfg(not(feature = "no-demo"))]
fn serve_dashboard(state: &Arc<RwLock<DemoState>>) -> Response<Bytes> {
    let state = state.read();
    let results = &state.analysis_results;
    
    // Calculate timing percentages
    let total_time = 100 + 150 + 200 + 250; // TODO: Get actual timings
    let context_percent = (100 * 100) / total_time;
    let complexity_percent = (150 * 100) / total_time;
    let dag_percent = (200 * 100) / total_time;
    let churn_percent = (250 * 100) / total_time;
    
    // Get p90 complexity
    let p90_complexity = results.complexity_report.summary.p90_cyclomatic;
    
    // Format the HTML with actual data
    let html = HTML_TEMPLATE
        .replace("{files_analyzed}", &results.files_analyzed.to_string())
        .replace("{avg_complexity:.2}", &format!("{:.2}", results.avg_complexity))
        .replace("{p90_complexity}", &p90_complexity.to_string())
        .replace("{tech_debt_hours}", &results.tech_debt_hours.to_string())
        .replace("{time_context}", "100")
        .replace("{time_complexity}", "150")
        .replace("{time_dag}", "200")
        .replace("{time_churn}", "250")
        .replace("{context_percent}", &context_percent.to_string())
        .replace("{complexity_percent}", &complexity_percent.to_string())
        .replace("{dag_percent}", &dag_percent.to_string())
        .replace("{churn_percent}", &churn_percent.to_string());

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html; charset=utf-8")
        .header("Cache-Control", "no-cache")
        .body(Bytes::from(html))
        .unwrap()
}

#[cfg(not(feature = "no-demo"))]
fn serve_static_asset(path: &str) -> Response<Bytes> {
    if let Some(asset) = get_asset(path) {
        let content = decompress_asset(asset);
        Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", asset.content_type)
            .header("Cache-Control", "public, max-age=3600")
            .body(Bytes::from(content.into_owned()))
            .unwrap()
    } else {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Bytes::from_static(b"404 Not Found"))
            .unwrap()
    }
}

#[cfg(feature = "no-demo")]
#[allow(dead_code)]
fn serve_static_asset(_path: &str) -> Response<Bytes> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Bytes::from_static(b"Demo mode disabled"))
        .unwrap()
}

// API endpoints
#[cfg(not(feature = "no-demo"))]
fn serve_summary_json(state: &Arc<RwLock<DemoState>>) -> Response<Bytes> {
    let state = state.read();
    let results = &state.analysis_results;
    
    let summary = serde_json::json!({
        "files_analyzed": results.files_analyzed,
        "avg_complexity": results.avg_complexity,
        "p90_complexity": results.complexity_report.summary.p90_cyclomatic,
        "tech_debt_hours": results.tech_debt_hours,
        "time_context": 100,
        "time_complexity": 150,
        "time_dag": 200,
        "time_churn": 250,
    });

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Bytes::from(serde_json::to_vec(&summary).unwrap()))
        .unwrap()
}

#[cfg(not(feature = "no-demo"))]
fn serve_metrics_json(state: &Arc<RwLock<DemoState>>) -> Response<Bytes> {
    let state = state.read();
    let metrics = serde_json::json!({
        "files_analyzed": state.analysis_results.files_analyzed,
        "avg_complexity": state.analysis_results.avg_complexity,
        "tech_debt_hours": state.analysis_results.tech_debt_hours,
    });

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Bytes::from(serde_json::to_vec(&metrics).unwrap()))
        .unwrap()
}

#[cfg(not(feature = "no-demo"))]
#[derive(Serialize)]
struct HotspotEntry {
    rank: usize,
    function: String,
    complexity: u32,
    loc: usize,
    path: String,
}

#[cfg(not(feature = "no-demo"))]
fn serve_hotspots_table(state: &Arc<RwLock<DemoState>>) -> Response<Bytes> {
    let state = state.read();

    // Extract top 10 complex functions
    let mut hotspots: Vec<_> = state
        .analysis_results
        .complexity_report
        .files
        .iter()
        .flat_map(|file| {
            file.functions.iter().map(move |func| HotspotEntry {
                rank: 0, // Will be set after sorting
                function: func.name.clone(),
                complexity: func.metrics.cyclomatic as u32,
                loc: func.metrics.lines as usize,
                path: file.path.clone(),
            })
        })
        .collect();

    // Sort by complexity descending
    hotspots.sort_unstable_by(|a, b| b.complexity.cmp(&a.complexity));

    // Assign ranks and take top 10
    for (idx, entry) in hotspots.iter_mut().enumerate() {
        entry.rank = idx + 1;
    }
    hotspots.truncate(10);

    // Serialize with minimal allocations
    let json = serde_json::to_vec(&hotspots).unwrap();

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .header("Cache-Control", "max-age=60")
        .body(Bytes::from(json))
        .unwrap()
}

#[cfg(not(feature = "no-demo"))]
fn serve_dag_mermaid(state: &Arc<RwLock<DemoState>>) -> Response<Bytes> {
    let state = state.read();

    // Generate Mermaid diagram
    let mermaid_generator = MermaidGenerator::new(MermaidOptions {
        show_complexity: true,
        filter_external: true,
        ..Default::default()
    });

    let diagram = mermaid_generator.generate(&state.analysis_results.dependency_graph);

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/plain")
        .header("Cache-Control", "max-age=60")
        .body(Bytes::from(diagram))
        .unwrap()
}

// Helper implementation moved here
impl DemoContent {
    #[allow(clippy::too_many_arguments)]
    pub fn from_analysis_results(
        dag: &DependencyGraph,
        files_analyzed: usize,
        avg_complexity: f64,
        tech_debt_hours: u32,
        hotspots: Vec<Hotspot>,
        ast_time_ms: u64,
        complexity_time_ms: u64,
        churn_time_ms: u64,
        dag_time_ms: u64,
    ) -> Self {
        let mermaid_generator = MermaidGenerator::new(MermaidOptions {
            show_complexity: true,
            filter_external: true,
            ..Default::default()
        });

        let mermaid_diagram = mermaid_generator.generate(dag);

        Self {
            mermaid_diagram,
            files_analyzed,
            avg_complexity,
            tech_debt_hours,
            hotspots,
            ast_time_ms,
            complexity_time_ms,
            churn_time_ms,
            dag_time_ms,
        }
    }
}

// For backwards compatibility with synchronous API
#[cfg(not(feature = "no-demo"))]
pub fn spawn_sync(initial_content: DemoContent) -> Result<LocalDemoServer> {
    // Create a tokio runtime for the synchronous API
    let runtime = tokio::runtime::Runtime::new()?;
    let (server, _port) = runtime.block_on(LocalDemoServer::spawn(initial_content))?;
    Ok(server)
}

impl Default for crate::services::complexity::ComplexityReport {
    fn default() -> Self {
        Self {
            summary: crate::services::complexity::ComplexitySummary {
                total_files: 0,
                total_functions: 0,
                avg_cyclomatic: 0.0,
                avg_cognitive: 0.0,
                p90_cyclomatic: 0,
                p90_cognitive: 0,
                technical_debt_hours: 0.0,
            },
            violations: vec![],
            hotspots: vec![],
            files: vec![],
        }
    }
}

impl Default for crate::models::churn::CodeChurnAnalysis {
    fn default() -> Self {
        Self {
            generated_at: chrono::Utc::now(),
            period_days: 0,
            repository_root: std::path::PathBuf::new(),
            files: vec![],
            summary: crate::models::churn::ChurnSummary {
                total_commits: 0,
                total_files_changed: 0,
                hotspot_files: vec![],
                stable_files: vec![],
                author_contributions: std::collections::HashMap::new(),
            },
        }
    }
}
