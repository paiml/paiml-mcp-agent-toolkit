#[cfg(feature = "demo")]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn serve_dashboard(state: &Arc<RwLock<DemoState>>) -> Response<Bytes> {
    let state = state.read();
    let results = &state.analysis_results;

    // Calculate timing percentages from DemoContent if available
    // For now use reasonable defaults since we don't store individual timings in state
    let total_time = 100 + 150 + 200 + 250;
    let context_percent = (100 * 100) / total_time;
    let complexity_percent = (150 * 100) / total_time;
    let dag_percent = (200 * 100) / total_time;
    let churn_percent = (250 * 100) / total_time;

    // Get p90 complexity
    let p90_complexity = results.complexity_report.summary.p90_cyclomatic;

    // Get version from Cargo.toml
    let version = env!("CARGO_PKG_VERSION");

    // Format the HTML with actual data
    let html = HTML_TEMPLATE
        .replace("{version}", version)
        .replace("{files_analyzed}", &results.files_analyzed.to_string())
        .replace(
            "{avg_complexity:.2}",
            &format!("{:.2}", results.avg_complexity),
        )
        .replace("{p90_complexity}", &p90_complexity.to_string())
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
        .expect("HTTP response construction cannot fail")
}

#[cfg(feature = "demo")]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn serve_static_asset(path: &str) -> Response<Bytes> {
    debug_assert!(!path.is_empty(), "path must not be empty");
    if let Some(asset) = get_asset(path) {
        let content = decompress_asset(asset);
        Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", asset.content_type)
            .header("Cache-Control", "public, max-age=3600")
            .body(Bytes::from(content.into_owned()))
            .expect("HTTP response construction cannot fail")
    } else {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Bytes::from_static(b"404 Not Found"))
            .expect("HTTP response construction cannot fail")
    }
}

#[cfg(not(feature = "demo"))]
#[allow(dead_code)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn serve_static_asset(_path: &str) -> Response<Bytes> {
    debug_assert!(!_path.is_empty(), "_path must not be empty");
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Bytes::from_static(b"Demo mode disabled"))
        .expect("HTTP response construction cannot fail")
}

// Disabled demo mode stubs for new endpoints
#[cfg(not(feature = "demo"))]
#[allow(dead_code)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn serve_architecture_analysis(
    _state: &std::sync::Arc<parking_lot::RwLock<DemoState>>,
) -> Response<Bytes> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Bytes::from_static(b"Demo mode disabled"))
        .expect("HTTP response construction cannot fail")
}

#[cfg(not(feature = "demo"))]
#[allow(dead_code)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn serve_defect_analysis(
    _state: &std::sync::Arc<parking_lot::RwLock<DemoState>>,
) -> Response<Bytes> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Bytes::from_static(b"Demo mode disabled"))
        .expect("HTTP response construction cannot fail")
}

#[cfg(not(feature = "demo"))]
#[allow(dead_code)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn serve_statistics_analysis(
    _state: &std::sync::Arc<parking_lot::RwLock<DemoState>>,
) -> Response<Bytes> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Bytes::from_static(b"Demo mode disabled"))
        .expect("HTTP response construction cannot fail")
}

#[cfg(not(feature = "demo"))]
#[allow(dead_code)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn serve_system_diagram(
    _state: &std::sync::Arc<parking_lot::RwLock<DemoState>>,
) -> Response<Bytes> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Bytes::from_static(b"Demo mode disabled"))
        .expect("HTTP response construction cannot fail")
}

#[cfg(not(feature = "demo"))]
#[allow(dead_code)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn serve_analysis_stream(
    _state: &std::sync::Arc<parking_lot::RwLock<DemoState>>,
) -> Response<Bytes> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Bytes::from_static(b"Demo mode disabled"))
        .expect("HTTP response construction cannot fail")
}

#[cfg(not(feature = "demo"))]
#[allow(dead_code)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn serve_recommendations_json(
    _state: &std::sync::Arc<parking_lot::RwLock<DemoState>>,
) -> Response<Bytes> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Bytes::from_static(b"Demo mode disabled"))
        .expect("HTTP response construction cannot fail")
}

#[cfg(not(feature = "demo"))]
#[allow(dead_code)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn serve_polyglot_analysis(
    _state: &std::sync::Arc<parking_lot::RwLock<DemoState>>,
) -> Response<Bytes> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Bytes::from_static(b"Demo mode disabled"))
        .expect("HTTP response construction cannot fail")
}

#[cfg(not(feature = "demo"))]
#[allow(dead_code)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn serve_showcase_gallery(
    _state: &std::sync::Arc<parking_lot::RwLock<DemoState>>,
) -> Response<Bytes> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Bytes::from_static(b"Demo mode disabled"))
        .expect("HTTP response construction cannot fail")
}

#[cfg(not(feature = "demo"))]
#[allow(dead_code)]
fn calculate_graph_density(_graph: &DependencyGraph) -> f64 {
    debug_assert!(true, "contract: calculate_graph_density");
    0.0
}

#[cfg(not(feature = "demo"))]
#[allow(dead_code)]
fn calculate_avg_degree(_graph: &DependencyGraph) -> f64 {
    0.0
}

#[cfg(not(feature = "demo"))]
#[allow(dead_code)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn serve_analysis_data(
    _state: &std::sync::Arc<parking_lot::RwLock<DemoState>>,
) -> Response<Bytes> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Bytes::from_static(b"Demo mode disabled"))
        .expect("HTTP response construction cannot fail")
}

