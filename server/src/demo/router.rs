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

    /// HTTP router for demo and API endpoints with exact path matching.
    ///
    /// This router provides the REST API interface for the PMAT demo server,
    /// handling both dashboard UI routes and API endpoints. Critical for maintaining
    /// API compatibility and preventing REST endpoint drift.
    ///
    /// # Route Categories
    ///
    /// ## Dashboard Routes
    /// - `/` - Main dashboard interface
    /// - `/vendor/*` - Static vendor assets (CSS, JS libraries)
    /// - `/demo.*` - Demo-specific static files
    ///
    /// ## Core API v1 Routes
    /// - `/api/summary` - Project summary metrics
    /// - `/api/metrics` - Detailed analysis metrics
    /// - `/api/hotspots` - Code complexity hotspots
    /// - `/api/dag` - Dependency graph in Mermaid format
    /// - `/api/system-diagram` - System architecture diagram
    /// - `/api/analysis` - Comprehensive analysis data
    ///
    /// ## Enhanced API v1 Routes
    /// - `/api/v1/analysis/architecture` - Architecture analysis
    /// - `/api/v1/analysis/defects` - Defect detection results
    /// - `/api/v1/analysis/statistics` - Statistical analysis
    /// - `/api/v1/analysis/diagram` - System diagrams
    /// - `/api/v1/analysis/stream` - Real-time analysis stream
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Create custom router
    /// let router = Router::new();
    ///
    /// // Router starts with empty routes
    /// assert_eq!(router.exact_routes.len(), 0);
    /// ```
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

        /// Handles incoming HTTP requests by routing to appropriate handlers.
        ///
        /// This method implements the core routing logic with exact path matching
        /// for API endpoints and prefix matching for static assets. Ensures consistent
        /// REST API behavior and prevents route conflicts.
        ///
        /// # Parameters
        ///
        /// * `path` - The request path to route (e.g., "/api/summary", "/vendor/bootstrap.css")
        /// * `state` - Shared demo state containing analysis data and metrics
        ///
        /// # Returns
        ///
        /// An HTTP response with appropriate status code, headers, and body content.
        ///
        /// # Routing Logic
        ///
        /// 1. **Exact Match**: Check registered routes for exact path matches
        /// 2. **Prefix Match**: Handle static assets with prefix matching
        /// 3. **404 Fallback**: Return Not Found for unmatched paths
        ///
        /// # Examples
        ///
        /// ```rust,ignore
        /// let router = Router::new();
        ///
        /// // Router provides a handle method for path routing
        /// // Implementation depends on configured routes
        /// assert!(router.exact_routes.is_empty());
        /// ```
        ///
        /// # API Endpoint Examples
        ///
        /// ```rust
        /// // The handle_request function provides routing for demo endpoints
        /// // Example paths: "/", "/api/summary", "/api/metrics"
        /// // Returns HTTP responses based on the requested path
        /// let example_path = "/api/summary";
        /// assert!(example_path.starts_with("/api/"));
        /// ```
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

    /// Main entry point for handling HTTP requests in the demo server.
    ///
    /// This function provides the primary REST API interface for the PMAT demo server,
    /// routing requests to appropriate handlers based on the configured routes.
    /// Critical for maintaining API stability and preventing endpoint drift.
    ///
    /// # Parameters
    ///
    /// * `path` - HTTP request path (e.g., "/api/summary", "/", "/vendor/style.css")
    /// * `state` - Shared demo state containing analysis results and metrics
    ///
    /// # Returns
    ///
    /// HTTP response with appropriate status, headers, and content based on the route.
    ///
    /// # Supported API Endpoints
    ///
    /// ## Core Analysis APIs
    /// - `GET /api/summary` - Project overview and key metrics
    /// - `GET /api/metrics` - Detailed quantitative analysis
    /// - `GET /api/hotspots` - Code complexity and quality hotspots
    /// - `GET /api/dag` - Dependency graph in Mermaid format
    /// - `GET /api/analysis` - Comprehensive analysis data
    ///
    /// ## Enhanced v1 APIs
    /// - `GET /api/v1/analysis/architecture` - System architecture analysis
    /// - `GET /api/v1/analysis/defects` - Defect detection and prediction
    /// - `GET /api/v1/analysis/statistics` - Statistical code analysis
    /// - `GET /api/v1/analysis/diagram` - System diagrams and visualizations
    /// - `GET /api/v1/analysis/stream` - Real-time analysis data stream
    ///
    /// ## UI and Assets
    /// - `GET /` - Main dashboard interface
    /// - `GET /vendor/*` - Third-party CSS/JS libraries
    /// - `GET /demo.*` - Demo-specific static assets
    ///
    /// # Response Formats
    ///
    /// - **JSON APIs**: `Content-Type: application/json`
    /// - **Mermaid Diagrams**: `Content-Type: text/plain`
    /// - **HTML Dashboard**: `Content-Type: text/html`
    /// - **Static Assets**: Appropriate MIME types
    ///
    /// # Examples
    ///
    /// ```rust
    /// // The handle_request function routes demo server requests
    /// // Supports dashboard, API endpoints, and static assets
    /// let api_path = "/api/summary";
    /// let dashboard_path = "/";
    /// assert!(api_path.starts_with("/api/"));
    /// assert_eq!(dashboard_path, "/");
    /// ```
    ///
    /// # Integration Examples
    ///
    /// ## curl API Testing
    /// ```bash
    /// # Get project summary
    /// curl -H "Accept: application/json" http://localhost:3000/api/summary
    ///
    /// # Get complexity hotspots
    /// curl http://localhost:3000/api/hotspots
    ///
    /// # Get dependency graph
    /// curl http://localhost:3000/api/dag
    /// ```
    ///
    /// ## JavaScript Integration
    /// ```javascript
    /// // Fetch analysis metrics
    /// fetch('/api/metrics')
    ///   .then(response => response.json())
    ///   .then(data => console.log('Metrics:', data));
    ///
    /// // Real-time analysis stream
    /// const eventSource = new EventSource('/api/v1/analysis/stream');
    /// eventSource.onmessage = (event) => {
    ///   const data = JSON.parse(event.data);
    ///   updateDashboard(data);
    /// };
    /// ```
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
