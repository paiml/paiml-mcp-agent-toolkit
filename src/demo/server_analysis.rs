#[cfg(feature = "demo")]
pub(crate) fn serve_dag_mermaid(state: &Arc<RwLock<DemoState>>) -> Response<Bytes> {
    let state = state.read();

    // Generate Mermaid diagram with TDG visualization
    let mermaid_generator = MermaidGenerator::new(MermaidOptions {
        show_complexity: true,
        filter_external: true,
        group_by_module: true,
        max_depth: Some(10),
    });

    let mut diagram = mermaid_generator.generate(&state.analysis_results.dependency_graph);

    // If diagram is empty or just "graph TD", provide fallback with TDG info
    if diagram.trim() == "graph TD" || diagram.trim().is_empty() {
        // Check if we have TDG data to show
        let tdg_info = if let Some(tdg_summary) = &state.analysis_results.tdg_summary {
            format!(
                r#"
    subgraph TDG_Legend[" TDG Analysis"]
        TDG_Info["📊 {} files analyzed<br/>🔴 {} critical (TDG>2.5)<br/>🟡 {} warning (1.5-2.5)<br/>🟢 {} normal (<1.5)"]
        TDG_Avg["📈 Average TDG: {:.2}"]
    end"#,
                tdg_summary.total_files,
                tdg_summary.critical_files,
                tdg_summary.warning_files,
                tdg_summary.total_files - tdg_summary.critical_files - tdg_summary.warning_files,
                tdg_summary.average_tdg
            )
        } else {
            String::with_capacity(1024)
        };

        diagram = format!(
            r"graph TD
    A[DemoRunner] -->|uses| B[StatelessTemplateServer]
    A -->|generates| C[DemoReport]
    A -->|creates| D[System Diagram]
    
    E[LocalDemoServer] -->|serves| F[Dashboard]
    E -->|handles| G[API Endpoints]
    
    H[DagBuilder] -->|creates| I[DependencyGraph]
    H -->|processes| J[ProjectContext]
    
    K[TDG Calculator] -->|analyzes| L[Technical Debt]
    K -->|reports| M[TDG Scores]
    
    N[File Discovery] -->|filters| O[Project Files]
    N -->|excludes| P[External Dependencies]
    {tdg_info}
    
    %% TDG-based styling
    style A fill:#90EE90
    style E fill:#FFD700
    style H fill:#FFA500
    style K fill:#87CEEB
    style N fill:#DDA0DD"
        );
    }

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/plain")
        .header("Cache-Control", "max-age=60")
        .body(Bytes::from(diagram))
        .expect("HTTP response construction cannot fail")
}

#[cfg(feature = "demo")]
pub(crate) fn serve_system_diagram_mermaid(state: &Arc<RwLock<DemoState>>) -> Response<Bytes> {
    let state = state.read();

    // Use the actual system diagram from DemoRunner if available
    let system_diagram = if let Some(ref diagram) = state.system_diagram {
        diagram.clone()
    } else {
        // Fallback to hardcoded diagram if no dynamic one available
        r"graph TD
    A[AST Context Analysis] -->|uses| B[File Parser]
    B --> C[Rust AST]
    B --> D[TypeScript AST]
    B --> E[Python AST]

    F[Code Complexity] -->|analyzes| C
    F -->|analyzes| D
    F -->|analyzes| E

    G[DAG Generation] -->|reads| C
    G -->|reads| D
    G -->|reads| E

    H[Code Churn] -->|git history| I[Git Analysis]

    J[Template Generation] -->|renders| K[Handlebars]

    style A fill:#90EE90
    style F fill:#FFD700
    style G fill:#FFA500
    style H fill:#FF6347
    style J fill:#87CEEB"
            .to_string()
    };

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/plain")
        .header("Cache-Control", "max-age=60")
        .body(Bytes::from(system_diagram))
        .expect("HTTP response construction cannot fail")
}

// Enhanced API endpoints following the specification

#[cfg(feature = "demo")]
pub(crate) fn serve_architecture_analysis(state: &Arc<RwLock<DemoState>>) -> Response<Bytes> {
    use crate::services::canonical_query::{
        AnalysisContext, CallGraph, CanonicalQuery, SystemArchitectureQuery,
    };
    use rustc_hash::FxHashMap;

    let state = state.read();

    // Build minimal context for architecture analysis
    let context = AnalysisContext {
        project_path: state.repository.clone(),
        ast_dag: state.analysis_results.dependency_graph.clone(),
        call_graph: CallGraph::default(),
        complexity_map: FxHashMap::default(),
        churn_analysis: Some(state.analysis_results.churn_analysis.clone()),
    };

    let query = SystemArchitectureQuery;
    match query.execute(&context) {
        Ok(result) => Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .header("Cache-Control", "max-age=60")
            .body(Bytes::from(
                serde_json::to_vec(&result).expect("JSON serialization cannot fail for result"),
            ))
            .expect("HTTP response construction cannot fail"),
        Err(_) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Bytes::from_static(b"Architecture analysis failed"))
            .expect("HTTP response construction cannot fail"),
    }
}

#[cfg(feature = "demo")]
pub(crate) fn serve_defect_analysis(state: &Arc<RwLock<DemoState>>) -> Response<Bytes> {
    // Return defect analysis data
    let state = state.read();

    let placeholder = serde_json::json!({
        "summary": {
            "total_files": state.analysis_results.files_analyzed,
            "critical_files": 0,
            "warning_files": 0,
            "average_tdg": 1.2,
            "p95_tdg": 2.1,
            "p99_tdg": 3.0,
            "estimated_debt_hours": 24.5,
            "hotspots": []
        },
        "recommendations": []
    });

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .header("Cache-Control", "max-age=60")
        .body(Bytes::from(
            serde_json::to_vec(&placeholder)
                .expect("JSON serialization cannot fail for placeholder"),
        ))
        .expect("HTTP response construction cannot fail")
}

#[cfg(feature = "demo")]
pub(crate) fn serve_statistics_analysis(state: &Arc<RwLock<DemoState>>) -> Response<Bytes> {
    let state = state.read();

    // Calculate comprehensive project statistics
    let stats = serde_json::json!({
        "structural_metrics": {
            "node_count": state.analysis_results.dependency_graph.nodes.len(),
            "edge_count": state.analysis_results.dependency_graph.edges.len(),
            "density": calculate_graph_density(&state.analysis_results.dependency_graph),
            "avg_degree": calculate_avg_degree(&state.analysis_results.dependency_graph),
        },
        "code_metrics": {
            "total_files": state.analysis_results.complexity_report.summary.total_files,
            "total_functions": state.analysis_results.complexity_report.summary.total_functions,
            "median_complexity": state.analysis_results.complexity_report.summary.median_cyclomatic,
            "complexity_p90": state.analysis_results.complexity_report.summary.p90_cyclomatic,
            "tech_debt_hours": state.analysis_results.complexity_report.summary.technical_debt_hours,
        },
        "temporal_metrics": {
            "total_commits": state.analysis_results.churn_analysis.summary.total_commits,
            "total_files_changed": state.analysis_results.churn_analysis.summary.total_files_changed,
            "active_authors": state.analysis_results.churn_analysis.summary.author_contributions.len(),
        }
    });

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .header("Cache-Control", "max-age=60")
        .body(Bytes::from(
            serde_json::to_vec(&stats).expect("JSON serialization cannot fail for stats"),
        ))
        .expect("HTTP response construction cannot fail")
}

#[cfg(feature = "demo")]
pub(crate) fn serve_system_diagram(state: &Arc<RwLock<DemoState>>) -> Response<Bytes> {
    // This endpoint could support content negotiation in the future
    serve_architecture_analysis(state)
}

#[cfg(feature = "demo")]
pub(crate) fn serve_analysis_stream(_state: &Arc<RwLock<DemoState>>) -> Response<Bytes> {
    // Placeholder for Server-Sent Events streaming
    // Return streaming response placeholder
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/event-stream")
        .body(Bytes::from_static(
            b"data: {\"event\":\"complete\",\"result\":\"Analysis complete\"}\n\n",
        ))
        .expect("HTTP response construction cannot fail")
}

// Grid.js API endpoint for file analysis data
#[cfg(feature = "demo")]
pub(crate) fn serve_analysis_data(state: &Arc<RwLock<DemoState>>) -> Response<Bytes> {
    let state = state.read();

    // Prepare data in the format expected by Grid.js
    let mut analysis_data = Vec::new();

    // Extract file-level metrics from complexity report
    for file in &state.analysis_results.complexity_report.files {
        // Calculate aggregate metrics for the file
        let total_complexity: u32 = file
            .functions
            .iter()
            .map(|f| u32::from(f.metrics.cognitive))
            .sum();

        let max_complexity = file
            .functions
            .iter()
            .map(|f| u32::from(f.metrics.cognitive))
            .max()
            .unwrap_or(0);

        let total_loc: usize = file
            .functions
            .iter()
            .map(|f| f.metrics.lines as usize)
            .sum();

        // Find churn data for this file
        let churn_data = state
            .analysis_results
            .churn_analysis
            .files
            .iter()
            .find(|f| f.relative_path == file.path);

        let commit_count = churn_data.map_or(0, |c| c.commit_count);
        let churn_score = churn_data.map_or(0.0, |c| c.churn_score);

        // Calculate TDG score instead of defect probability
        let cognitive_normalized = (f64::from(max_complexity) / 20.0).min(3.0);
        let churn_normalized = (f64::from(churn_score) / 10.0).min(3.0);
        let tdg_value = (cognitive_normalized * 0.3 + churn_normalized * 0.3 + 0.4).min(3.0);
        let tdg_severity = match tdg_value {
            v if v > 2.5 => "Critical",
            v if v > 1.5 => "Warning",
            _ => "Normal",
        };

        analysis_data.push(serde_json::json!({
            "path": file.path.replace("./server/src/", ""),
            "complexity_metrics": {
                "cognitive": max_complexity,
                "cyclomatic": file.total_complexity.cyclomatic,
                "total": total_complexity
            },
            "churn_metrics": {
                "commit_count": commit_count,
                "churn_score": churn_score
            },
            "tdg_score": tdg_value,
            "tdg_severity": tdg_severity,
            "lines_of_code": total_loc
        }));
    }

    // If no complexity data, provide fallback
    if analysis_data.is_empty() {
        analysis_data = vec![
            serde_json::json!({
                "path": "demo/server.rs",
                "complexity_metrics": { "cognitive": 12, "cyclomatic": 10, "total": 45 },
                "churn_metrics": { "commit_count": 5, "churn_score": 3.2 },
                "tdg_score": 0.89,
                "tdg_severity": "Low",
                "lines_of_code": 789
            }),
            serde_json::json!({
                "path": "services/dag_builder.rs",
                "complexity_metrics": { "cognitive": 18, "cyclomatic": 15, "total": 67 },
                "churn_metrics": { "commit_count": 8, "churn_score": 5.1 },
                "tdg_score": 1.67,
                "tdg_severity": "Medium",
                "lines_of_code": 456
            }),
            serde_json::json!({
                "path": "handlers/tools.rs",
                "complexity_metrics": { "cognitive": 9, "cyclomatic": 7, "total": 28 },
                "churn_metrics": { "commit_count": 3, "churn_score": 1.8 },
                "tdg_score": 0.65,
                "tdg_severity": "Low",
                "lines_of_code": 234
            }),
        ];
    }

    let response_data = serde_json::json!({
        "ast_contexts": analysis_data,
        "total_files": state.analysis_results.files_analyzed,
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .header("Cache-Control", "max-age=60")
        .body(Bytes::from(
            serde_json::to_vec(&response_data)
                .expect("JSON serialization cannot fail for response data"),
        ))
        .expect("HTTP response construction cannot fail")
}

// Helper function removed - using TDG scores instead of defect probability

// Helper functions for statistics calculation

#[cfg(feature = "demo")]
fn calculate_graph_density(graph: &DependencyGraph) -> f64 {
    let n = graph.nodes.len() as f64;
    if n <= 1.0 {
        0.0
    } else {
        graph.edges.len() as f64 / (n * (n - 1.0))
    }
}

#[cfg(feature = "demo")]
fn calculate_avg_degree(graph: &DependencyGraph) -> f64 {
    let n = graph.nodes.len() as f64;
    if n == 0.0 {
        0.0
    } else {
        2.0 * graph.edges.len() as f64 / n
    }
}

// Helper implementation moved here
