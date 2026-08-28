#[cfg(feature = "demo")]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn serve_summary_json(state: &Arc<RwLock<DemoState>>) -> Response<Bytes> {
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
        .body(Bytes::from(
            serde_json::to_vec(&summary).expect("JSON serialization cannot fail for DemoContent"),
        ))
        .expect("HTTP response construction cannot fail")
}

#[cfg(feature = "demo")]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn serve_metrics_json(state: &Arc<RwLock<DemoState>>) -> Response<Bytes> {
    let state = state.read();
    let metrics = serde_json::json!({
        "files_analyzed": state.analysis_results.files_analyzed,
        "avg_complexity": state.analysis_results.avg_complexity,
        "tech_debt_hours": state.analysis_results.tech_debt_hours,
    });

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Bytes::from(
            serde_json::to_vec(&metrics).expect("JSON serialization cannot fail for metrics"),
        ))
        .expect("HTTP response construction cannot fail")
}

#[cfg(feature = "demo")]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn serve_recommendations_json(state: &Arc<RwLock<DemoState>>) -> Response<Bytes> {
    let state = state.read();

    // Detect the primary language of the repository
    let language = crate::cli::detect_primary_language(&state.repository)
        .unwrap_or_else(|| "rust".to_string());

    // Generate recommendations based on analysis results
    let recommendation_engine = RecommendationEngine::new();
    let recommendations =
        recommendation_engine.get_recommendations(&language, Some(ComplexityTier::Intermediate));

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Bytes::from(
            serde_json::to_vec(&recommendations)
                .expect("JSON serialization cannot fail for recommendations"),
        ))
        .expect("HTTP response construction cannot fail")
}

#[cfg(feature = "demo")]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn serve_polyglot_analysis(state: &Arc<RwLock<DemoState>>) -> Response<Bytes> {
    let state = state.read();

    // Detect primary language as a simple alternative to full polyglot analysis
    let primary_language = crate::cli::detect_primary_language(&state.repository)
        .unwrap_or_else(|| "rust".to_string());

    // Create a simplified polyglot response based on detected language
    // This avoids the thread safety issues with the async polyglot analyzer
    let polyglot_data = serde_json::json!({
        "languages": [
            {
                "language": primary_language,
                "file_count": 25, // These would be calculated in a full implementation
                "line_count": 2500,
                "complexity_score": 6.5,
                "test_coverage": 0.85,
                "primary_frameworks": match primary_language.as_str() {
                    "rust" => vec!["Tokio", "Serde", "Clap"],
                    "python" | "python-uv" => vec!["FastAPI", "Pydantic", "SQLAlchemy"],
                    "javascript" | "typescript" => vec!["React", "Express", "Jest"],
                    _ => vec![]
                }
            }
        ],
        "cross_language_dependencies": [],
        "architecture_pattern": "Monolithic",
        "integration_points": [],
        "recommendation_score": 0.8,
        "detected_primary": primary_language
    });

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Bytes::from(
            serde_json::to_vec(&polyglot_data)
                .expect("JSON serialization cannot fail for polyglot data"),
        ))
        .expect("HTTP response construction cannot fail")
}

#[cfg(feature = "demo")]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn serve_showcase_gallery(_state: &Arc<RwLock<DemoState>>) -> Response<Bytes> {
    let gallery = ShowcaseGallery::new();
    let showcase_data = serde_json::json!({
        "repositories": gallery.get_all_repositories(),
        "summary": gallery.generate_showcase_summary(),
        "featured": gallery.get_featured_repositories(),
        "quickStart": gallery.get_quick_start_recommendations(),
        "categories": gallery.get_categories()
    });

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Bytes::from(
            serde_json::to_vec(&showcase_data)
                .expect("JSON serialization cannot fail for showcase data"),
        ))
        .expect("HTTP response construction cannot fail")
}

#[cfg(feature = "demo")]
#[derive(Serialize)]
struct HotspotEntry {
    rank: usize,
    function: String,
    complexity: u32,
    loc: usize,
    path: String,
}

#[cfg(feature = "demo")]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn serve_hotspots_table(state: &Arc<RwLock<DemoState>>) -> Response<Bytes> {
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
                complexity: u32::from(func.metrics.cyclomatic),
                loc: func.metrics.lines as usize,
                path: file.path.clone(),
            })
        })
        .collect();

    // If no hotspots found, provide fallback data
    if hotspots.is_empty() {
        hotspots = vec![
            HotspotEntry {
                rank: 1,
                function: "serve_dashboard".to_string(),
                complexity: 12,
                loc: 35,
                path: "server/src/demo/server.rs".to_string(),
            },
            HotspotEntry {
                rank: 2,
                function: "execute_with_diagram".to_string(),
                complexity: 11,
                loc: 45,
                path: "server/src/demo/runner.rs".to_string(),
            },
            HotspotEntry {
                rank: 3,
                function: "handle_connection".to_string(),
                complexity: 9,
                loc: 28,
                path: "server/src/demo/server.rs".to_string(),
            },
            HotspotEntry {
                rank: 4,
                function: "render_system_mermaid".to_string(),
                complexity: 8,
                loc: 30,
                path: "server/src/demo/runner.rs".to_string(),
            },
            HotspotEntry {
                rank: 5,
                function: "build_from_project".to_string(),
                complexity: 7,
                loc: 22,
                path: "server/src/services/dag_builder.rs".to_string(),
            },
        ];
    } else {
        // Sort by complexity descending
        hotspots.sort_unstable_by_key(|h| std::cmp::Reverse(h.complexity));

        // Assign ranks and take top 10
        for (idx, entry) in hotspots.iter_mut().enumerate() {
            entry.rank = idx + 1;
        }
        hotspots.truncate(10);
    }

    // Serialize with minimal allocations
    let json = serde_json::to_vec(&hotspots).expect("JSON serialization cannot fail for hotspots");

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .header("Cache-Control", "max-age=60")
        .body(Bytes::from(json))
        .expect("HTTP response construction cannot fail")
}

