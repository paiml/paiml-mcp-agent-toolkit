
/// Builder for creating HTTP responses with common patterns and content types.
///
/// This utility provides a fluent API for creating `UnifiedResponse` objects with
/// common HTTP status codes, content types, and body formats. Essential for
/// maintaining consistent REST API responses across the application.
///
/// # Response Types
///
/// - **JSON**: Structured data responses with automatic serialization
/// - **Text**: Plain text responses with UTF-8 encoding
/// - **HTML**: Web content with proper content-type headers
/// - **Status**: Standard HTTP status responses (200, 404, 500, etc.)
///
/// # Content Type Management
///
/// The builder automatically sets appropriate `Content-Type` headers:
/// - JSON: `application/json; charset=utf-8`
/// - Text: `text/plain; charset=utf-8`
/// - HTML: `text/html; charset=utf-8`
///
/// # Examples
///
/// ```ignore
/// use pmat::unified_protocol::adapters::http::HttpResponseBuilder;
/// use serde_json::json;
///
/// // Success responses
/// let ok_response = HttpResponseBuilder::ok();
/// assert_eq!(ok_response.status, axum::http::StatusCode::OK);
///
/// // JSON API responses
/// let api_response = HttpResponseBuilder::json(&json!({
///     "status": "success",
///     "data": {
///         "user_id": 123,
///         "name": "Alice"
///     },
///     "timestamp": "2024-01-15T10:30:00Z"
/// })).expect("valid json");
/// assert_eq!(api_response.status, axum::http::StatusCode::OK);
/// assert!(api_response.headers.contains_key("content-type"));
///
/// // Error responses
/// let not_found = HttpResponseBuilder::not_found();
/// assert_eq!(not_found.status, axum::http::StatusCode::NOT_FOUND);
///
/// let server_error = HttpResponseBuilder::internal_error();
/// assert_eq!(server_error.status, axum::http::StatusCode::INTERNAL_SERVER_ERROR);
///
/// // Content responses
/// let text_response = HttpResponseBuilder::text("Hello, World!");
/// assert_eq!(text_response.status, axum::http::StatusCode::OK);
///
/// let html_response = HttpResponseBuilder::html(
///     "<html><body><h1>Welcome</h1></body></html>"
/// );
/// assert_eq!(html_response.status, axum::http::StatusCode::OK);
/// ```ignore
///
/// # REST API Usage Examples
///
/// ```ignore
/// use pmat::unified_protocol::adapters::http::HttpResponseBuilder;
/// use serde_json::json;
///
/// // GET /api/users/123 - Success
/// let user_data = json!({
///     "id": 123,
///     "name": "Alice Smith",
///     "email": "alice@example.com",
///     "role": "admin"
/// });
/// let response = HttpResponseBuilder::json(&user_data).expect("valid json");
///
/// // GET /api/users/999 - Not Found
/// let not_found = HttpResponseBuilder::not_found();
///
/// // POST /api/users - Created
/// let created_user = json!({
///     "id": 124,
///     "name": "Bob Johnson",
///     "created_at": "2024-01-15T10:30:00Z"
/// });
/// let created_response = HttpResponseBuilder::json(&created_user).expect("valid json");
///
/// // GET /health - Health Check
/// let health_response = HttpResponseBuilder::json(&json!({
///     "status": "healthy",
///     "version": "1.0.0",
///     "uptime": "2d 5h 23m"
/// })).expect("valid json");
/// ```ignore
pub struct HttpResponseBuilder;

impl HttpResponseBuilder {
    /// Creates a successful HTTP 200 OK response.
    ///
    /// # Returns
    ///
    /// A `UnifiedResponse` with status code 200 and empty body.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::unified_protocol::adapters::http::HttpResponseBuilder;
    /// use axum::http::StatusCode;
    ///
    /// let response = HttpResponseBuilder::ok();
    /// assert_eq!(response.status, StatusCode::OK);
    /// ```
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn ok() -> UnifiedResponse {
        UnifiedResponse::new(StatusCode::OK)
    }

    /// Creates an HTTP 404 Not Found response.
    ///
    /// # Returns
    ///
    /// A `UnifiedResponse` with status code 404 and empty body.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::unified_protocol::adapters::http::HttpResponseBuilder;
    /// use axum::http::StatusCode;
    ///
    /// let response = HttpResponseBuilder::not_found();
    /// assert_eq!(response.status, StatusCode::NOT_FOUND);
    /// ```
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn not_found() -> UnifiedResponse {
        UnifiedResponse::new(StatusCode::NOT_FOUND)
    }

    /// Creates an HTTP 500 Internal Server Error response.
    ///
    /// # Returns
    ///
    /// A `UnifiedResponse` with status code 500 and empty body.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::unified_protocol::adapters::http::HttpResponseBuilder;
    /// use axum::http::StatusCode;
    ///
    /// let response = HttpResponseBuilder::internal_error();
    /// assert_eq!(response.status, StatusCode::INTERNAL_SERVER_ERROR);
    /// ```
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn internal_error() -> UnifiedResponse {
        UnifiedResponse::new(StatusCode::INTERNAL_SERVER_ERROR)
    }

    /// Creates a JSON response with automatic serialization and content-type header.
    ///
    /// Serializes the provided data to JSON and sets the appropriate content-type
    /// header. Essential for REST API endpoints returning structured data.
    ///
    /// # Parameters
    ///
    /// * `data` - Any serializable data structure implementing `Serialize`
    ///
    /// # Returns
    ///
    /// * `Ok(UnifiedResponse)` - HTTP 200 response with JSON body and headers
    /// * `Err(serde_json::Error)` - Serialization failed
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::unified_protocol::adapters::http::HttpResponseBuilder;
    /// use serde_json::json;
    /// use axum::http::StatusCode;
    ///
    /// // Simple JSON response
    /// let response = HttpResponseBuilder::json(&json!({
    ///     "message": "Hello, World!",
    ///     "status": "success"
    /// })).expect("valid json");
    ///
    /// assert_eq!(response.status, StatusCode::OK);
    /// assert!(response.headers.contains_key("content-type"));
    ///
    /// // API data response
    /// let user_data = json!({
    ///     "id": 123,
    ///     "name": "Alice",
    ///     "email": "alice@example.com"
    /// });
    /// let user_response = HttpResponseBuilder::json(&user_data).expect("valid json");
    /// ```
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn json<T: Serialize>(data: &T) -> Result<UnifiedResponse, serde_json::Error> {
        UnifiedResponse::ok().with_json(data)
    }

    /// Creates a plain text response with UTF-8 encoding.
    ///
    /// Sets the content-type to `text/plain; charset=utf-8` and includes
    /// the provided text content in the response body.
    ///
    /// # Parameters
    ///
    /// * `content` - Text content to include in the response body
    ///
    /// # Returns
    ///
    /// A `UnifiedResponse` with status code 200, text body, and appropriate headers.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::unified_protocol::adapters::http::HttpResponseBuilder;
    /// use axum::http::StatusCode;
    ///
    /// // Simple text response
    /// let response = HttpResponseBuilder::text("Hello, World!");
    /// assert_eq!(response.status, StatusCode::OK);
    ///
    /// // Multi-line text response
    /// let multi_line = HttpResponseBuilder::text(
    ///     "Line 1\nLine 2\nLine 3"
    /// );
    ///
    /// // Log output response
    /// let log_response = HttpResponseBuilder::text(
    ///     "[INFO] Server started successfully\n[DEBUG] Listening on port 3000"
    /// );
    /// ```
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn text(content: &str) -> UnifiedResponse {
        debug_assert!(!content.is_empty(), "content must not be empty");
        UnifiedResponse::ok()
            .with_body(Body::from(content.to_string()))
            .with_header("content-type", "text/plain")
    }

    /// Creates an HTML response with proper content-type header.
    ///
    /// Sets the content-type to `text/html; charset=utf-8` and includes
    /// the provided HTML content in the response body.
    ///
    /// # Parameters
    ///
    /// * `content` - HTML content to include in the response body
    ///
    /// # Returns
    ///
    /// A `UnifiedResponse` with status code 200, HTML body, and appropriate headers.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::unified_protocol::adapters::http::HttpResponseBuilder;
    /// use axum::http::StatusCode;
    ///
    /// // Simple HTML page
    /// let response = HttpResponseBuilder::html(
    ///     "<html><body><h1>Welcome</h1></body></html>"
    /// );
    /// assert_eq!(response.status, StatusCode::OK);
    ///
    /// // Dashboard HTML response
    /// let dashboard = HttpResponseBuilder::html(r#"
    ///     <!DOCTYPE html>
    ///     <html>
    ///     <head><title>PMAT Dashboard</title></head>
    ///     <body>
    ///         <h1>Project Analysis Dashboard</h1>
    ///         <div id="metrics">Loading...</div>
    ///     </body>
    ///     </html>
    /// "#);
    /// ```
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn html(content: &str) -> UnifiedResponse {
        debug_assert!(!content.is_empty(), "content must not be empty");
        UnifiedResponse::ok()
            .with_body(Body::from(content.to_string()))
            .with_header("content-type", "text/html")
    }
}
