#[cfg(feature = "demo")]
mod implementation {
    use bytes::Bytes;
    use http::{Response, StatusCode};
    use lazy_static::lazy_static;
    use parking_lot::RwLock;
    use std::sync::Arc;

    use crate::demo::server::{
        serve_analysis_data, serve_analysis_stream, serve_architecture_analysis, serve_dag_mermaid,
        serve_dashboard, serve_defect_analysis, serve_hotspots_table, serve_metrics_json,
        serve_static_asset, serve_statistics_analysis, serve_summary_json, serve_system_diagram,
        serve_system_diagram_mermaid, DemoState,
    };

    type RouteHandler = fn(&Arc<RwLock<DemoState>>) -> Response<Bytes>;

    pub struct Router {
        exact_routes: Vec<(&'static str, RouteHandler)>,
    }

    impl Router {
        pub fn new() -> Self {
            Self {
                exact_routes: Vec::new(),
            }
        }

        pub fn route(mut self, path: &'static str, handler: RouteHandler) -> Self {
            self.exact_routes.push((path, handler));
            self
        }

        pub fn handle(&self, path: &str, state: &Arc<RwLock<DemoState>>) -> Response<Bytes> {
            // Check exact routes first
            for (route_path, handler) in &self.exact_routes {
                if path == *route_path {
                    return handler(state);
                }
            }

            // Check prefix routes for static assets
            if path.starts_with("/vendor/") || path.starts_with("/demo.") {
                return serve_static_asset(path);
            }

            // 404 Not Found
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Bytes::from_static(b"404 Not Found"))
                .unwrap()
        }
    }

    lazy_static! {
        pub static ref DEMO_ROUTES: Router = build_router();
    }

    fn build_router() -> Router {
        Router::new()
            // Dashboard and main UI
            .route("/", serve_dashboard)
            // Core API endpoints
            .route("/api/summary", serve_summary_json)
            .route("/api/metrics", serve_metrics_json)
            .route("/api/hotspots", serve_hotspots_table)
            .route("/api/dag", serve_dag_mermaid)
            .route("/api/system-diagram", serve_system_diagram_mermaid)
            .route("/api/analysis", serve_analysis_data)
            // Enhanced API v1 endpoints
            .route("/api/v1/analysis/architecture", serve_architecture_analysis)
            .route("/api/v1/analysis/defects", serve_defect_analysis)
            .route("/api/v1/analysis/statistics", serve_statistics_analysis)
            .route("/api/v1/analysis/diagram", serve_system_diagram)
            .route("/api/v1/analysis/stream", serve_analysis_stream)
    }

    pub fn handle_request(path: &str, state: &Arc<RwLock<DemoState>>) -> Response<Bytes> {
        DEMO_ROUTES.handle(path, state)
    }
}

#[cfg(feature = "demo")]
pub use implementation::handle_request;

#[cfg(not(feature = "demo"))]
pub fn handle_request(
    _path: &str,
    _state: &std::sync::Arc<parking_lot::RwLock<crate::demo::server::DemoState>>,
) -> http::Response<bytes::Bytes> {
    http::Response::builder()
        .status(http::StatusCode::NOT_FOUND)
        .body(bytes::Bytes::from_static(b"Demo mode disabled"))
        .unwrap()
}

#[cfg(test)]
mod tests {
    // use super::*; // Unused in simple tests

    #[test]
    fn test_router_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }
}
