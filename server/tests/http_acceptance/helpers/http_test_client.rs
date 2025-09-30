//! HTTP Test Client - Framework for HTTP API acceptance testing
//!
//! Provides a comprehensive testing framework for HTTP REST API interfaces.
//! Implements HTTP client with proper error handling, performance validation, and content negotiation.

use anyhow::{Context, Result};
use reqwest::{Client, Method};
use serde_json::Value;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tempfile::{tempdir, TempDir};

/// HTTP test client for REST API testing
pub struct HttpTestClient {
    pub client: Client,
    pub base_url: String,
    pub auth_token: Option<String>,
    pub default_headers: HashMap<String, String>,
    pub test_workspace: TempDir,
}

/// HTTP test result with performance metrics
#[derive(Debug, Clone)]
pub struct HttpTestResult {
    pub response: HttpResponse,
    pub execution_time: Duration,
    pub request_url: String,
    pub method: String,
    pub status_code: u16,
    pub success: bool,
}

/// HTTP response wrapper
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub json: Option<Value>,
}

impl HttpTestClient {
    /// Create a new HTTP test client
    pub fn new(base_url: &str) -> Result<Self> {
        let workspace = tempdir().context("Failed to create test workspace")?;

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        let mut default_headers = HashMap::new();
        default_headers.insert("Content-Type".to_string(), "application/json".to_string());
        default_headers.insert("Accept".to_string(), "application/json".to_string());
        default_headers.insert("User-Agent".to_string(), "pmat-test-client/1.0".to_string());

        Ok(Self {
            client,
            base_url: base_url.to_string(),
            auth_token: None,
            default_headers,
            test_workspace: workspace,
        })
    }

    /// GET request
    pub async fn get(&self, path: &str) -> Result<HttpTestResult> {
        self.request(Method::GET, path, None, None).await
    }

    /// POST request with JSON body
    pub async fn post(&self, path: &str, body: Option<Value>) -> Result<HttpTestResult> {
        self.request(Method::POST, path, body, None).await
    }

    /// PUT request with JSON body
    pub async fn put(&self, path: &str, body: Option<Value>) -> Result<HttpTestResult> {
        self.request(Method::PUT, path, body, None).await
    }

    /// DELETE request
    pub async fn delete(&self, path: &str) -> Result<HttpTestResult> {
        self.request(Method::DELETE, path, None, None).await
    }

    /// HEAD request
    pub async fn head(&self, path: &str) -> Result<HttpTestResult> {
        self.request(Method::HEAD, path, None, None).await
    }

    /// OPTIONS request (for CORS)
    pub async fn options(&self, path: &str) -> Result<HttpTestResult> {
        self.request(Method::OPTIONS, path, None, None).await
    }

    /// Generic request method
    pub async fn request(
        &self,
        method: Method,
        path: &str,
        body: Option<Value>,
        additional_headers: Option<HashMap<String, String>>,
    ) -> Result<HttpTestResult> {
        let start_time = Instant::now();
        let url = format!("{}{}", self.base_url, path);

        let mut request_builder = self.client.request(method.clone(), &url);

        // Add default headers
        for (key, value) in &self.default_headers {
            request_builder = request_builder.header(key, value);
        }

        // Add additional headers
        if let Some(headers) = additional_headers {
            for (key, value) in headers {
                request_builder = request_builder.header(key, value);
            }
        }

        // Add authentication if available
        if let Some(ref token) = self.auth_token {
            request_builder = request_builder.bearer_auth(token);
        }

        // Add body for POST/PUT requests
        if let Some(json_body) = body {
            request_builder = request_builder.json(&json_body);
        }

        // Execute request
        let response = request_builder
            .send()
            .await
            .context("Failed to send HTTP request")?;

        let execution_time = start_time.elapsed();
        let status_code = response.status().as_u16();
        let success = response.status().is_success();

        // Extract headers
        let mut headers = HashMap::new();
        for (key, value) in response.headers() {
            if let Ok(value_str) = value.to_str() {
                headers.insert(key.to_string(), value_str.to_string());
            }
        }

        // Extract body
        let body_text = response
            .text()
            .await
            .context("Failed to read response body")?;

        // Try to parse as JSON
        let json_value = if body_text.trim().starts_with('{') || body_text.trim().starts_with('[') {
            serde_json::from_str(&body_text).ok()
        } else {
            None
        };

        let http_response = HttpResponse {
            status: status_code,
            headers,
            body: body_text,
            json: json_value,
        };

        Ok(HttpTestResult {
            response: http_response,
            execution_time,
            request_url: url,
            method: method.to_string(),
            status_code,
            success,
        })
    }

    /// Set authentication token
    pub fn with_auth_token(mut self, token: String) -> Self {
        self.auth_token = Some(token);
        self
    }

    /// Add custom header
    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.default_headers
            .insert(key.to_string(), value.to_string());
        self
    }

    /// Set content type
    pub fn with_content_type(mut self, content_type: &str) -> Self {
        self.default_headers
            .insert("Content-Type".to_string(), content_type.to_string());
        self
    }

    /// Set accept header
    pub fn with_accept(mut self, accept: &str) -> Self {
        self.default_headers
            .insert("Accept".to_string(), accept.to_string());
        self
    }

    /// Create sample project for testing
    pub fn create_sample_project(&self) -> Result<std::path::PathBuf> {
        let project_path = self.test_workspace.path().join("sample_project");
        std::fs::create_dir_all(&project_path)?;

        // Create sample Rust project structure
        std::fs::create_dir_all(project_path.join("src"))?;
        std::fs::write(
            project_path.join("Cargo.toml"),
            r#"[package]
name = "sample-project"
version = "0.1.0"
edition = "2021"

[dependencies]
"#,
        )?;

        std::fs::write(
            project_path.join("src/main.rs"),
            r#"fn main() {
    println!("Hello, world!");
}

fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn complex_function(data: &[i32]) -> Vec<i32> {
    let mut result = Vec::new();
    for item in data {
        if *item > 0 {
            if *item % 2 == 0 {
                result.push(*item * 2);
            } else {
                result.push(*item * 3);
            }
        } else if *item < 0 {
            result.push(item.abs());
        }
    }
    result.sort();
    result.dedup();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
    }
}
"#,
        )?;

        Ok(project_path)
    }

    /// Get workspace path
    pub fn workspace_path(&self) -> &std::path::Path {
        self.test_workspace.path()
    }
}

/// Validation helpers for HTTP test results
pub struct HttpValidators;

impl HttpValidators {
    /// Validate HTTP status code
    pub fn assert_status_code(result: &HttpTestResult, expected_status: u16) -> Result<()> {
        if result.status_code != expected_status {
            anyhow::bail!(
                "Expected status code {} but got {} for {} {}",
                expected_status,
                result.status_code,
                result.method,
                result.request_url
            );
        }
        Ok(())
    }

    /// Validate successful response (2xx)
    pub fn assert_success(result: &HttpTestResult) -> Result<()> {
        if !result.success {
            anyhow::bail!(
                "Expected successful response but got status {} for {} {}",
                result.status_code,
                result.method,
                result.request_url
            );
        }
        Ok(())
    }

    /// Validate response contains expected headers
    pub fn assert_headers(
        result: &HttpTestResult,
        expected_headers: &[(&str, &str)],
    ) -> Result<()> {
        for (key, expected_value) in expected_headers {
            if let Some(actual_value) = result.response.headers.get(*key) {
                if actual_value != expected_value {
                    anyhow::bail!(
                        "Expected header '{}' to be '{}' but got '{}'",
                        key,
                        expected_value,
                        actual_value
                    );
                }
            } else {
                anyhow::bail!("Missing expected header '{}'", key);
            }
        }
        Ok(())
    }

    /// Validate response performance
    pub fn assert_performance(result: &HttpTestResult, max_duration: Duration) -> Result<()> {
        if result.execution_time > max_duration {
            anyhow::bail!(
                "Request took too long: {:?} > {:?} for {} {}",
                result.execution_time,
                max_duration,
                result.method,
                result.request_url
            );
        }
        Ok(())
    }

    /// Validate JSON response structure
    pub fn assert_json_structure(result: &HttpTestResult, expected_fields: &[&str]) -> Result<()> {
        if let Some(ref json) = result.response.json {
            for field in expected_fields {
                if json.get(field).is_none() {
                    anyhow::bail!("Missing expected JSON field '{}'", field);
                }
            }
        } else {
            anyhow::bail!("Response is not valid JSON");
        }
        Ok(())
    }

    /// Validate content type
    pub fn assert_content_type(result: &HttpTestResult, expected_type: &str) -> Result<()> {
        if let Some(content_type) = result.response.headers.get("content-type") {
            if !content_type.starts_with(expected_type) {
                anyhow::bail!(
                    "Expected content-type to start with '{}' but got '{}'",
                    expected_type,
                    content_type
                );
            }
        } else {
            anyhow::bail!("Missing Content-Type header");
        }
        Ok(())
    }

    /// Validate CORS headers
    pub fn assert_cors_headers(result: &HttpTestResult) -> Result<()> {
        let cors_headers = [
            "Access-Control-Allow-Origin",
            "Access-Control-Allow-Methods",
            "Access-Control-Allow-Headers",
        ];

        for header in &cors_headers {
            if !result.response.headers.contains_key(&header.to_lowercase()) {
                anyhow::bail!("Missing CORS header '{}'", header);
            }
        }
        Ok(())
    }

    /// Validate security headers
    pub fn assert_security_headers(result: &HttpTestResult) -> Result<()> {
        let security_headers = [
            ("x-content-type-options", "nosniff"),
            ("x-frame-options", "DENY"),
        ];

        for (header, expected_value) in &security_headers {
            if let Some(actual_value) = result.response.headers.get(*header) {
                if actual_value != expected_value {
                    anyhow::bail!(
                        "Security header '{}' should be '{}' but got '{}'",
                        header,
                        expected_value,
                        actual_value
                    );
                }
            } else {
                // Security headers are recommended but not always required
                println!("Warning: Missing security header '{}'", header);
            }
        }
        Ok(())
    }

    /// Validate error response structure
    pub fn assert_error_response(
        result: &HttpTestResult,
        expected_error_code: Option<u16>,
    ) -> Result<()> {
        if let Some(expected_code) = expected_error_code {
            Self::assert_status_code(result, expected_code)?;
        } else if result.success {
            anyhow::bail!("Expected error response but got success");
        }

        // Error responses should have meaningful error messages
        if let Some(ref json) = result.response.json {
            let has_error_info = json.get("error").is_some()
                || json.get("message").is_some()
                || json.get("details").is_some();

            if !has_error_info {
                anyhow::bail!("Error response should contain error information");
            }
        } else if result.response.body.trim().is_empty() {
            anyhow::bail!("Error response should not be empty");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_http_client_creation() {
        let client = HttpTestClient::new("http://localhost:3000");
        assert!(client.is_ok());

        let client = client.unwrap();
        assert_eq!(client.base_url, "http://localhost:3000");
        assert!(client.test_workspace.path().exists());
    }

    #[test]
    fn test_sample_project_creation() {
        let client = HttpTestClient::new("http://localhost:3000").unwrap();
        let project_path = client.create_sample_project().unwrap();

        assert!(project_path.join("Cargo.toml").exists());
        assert!(project_path.join("src/main.rs").exists());
    }

    #[test]
    fn test_client_configuration() {
        let client = HttpTestClient::new("http://localhost:3000")
            .unwrap()
            .with_auth_token("test-token".to_string())
            .with_content_type("application/x-www-form-urlencoded")
            .with_accept("text/html")
            .with_header("X-Custom-Header", "custom-value");

        assert_eq!(client.auth_token, Some("test-token".to_string()));
        assert_eq!(
            client.default_headers.get("Content-Type"),
            Some(&"application/x-www-form-urlencoded".to_string())
        );
        assert_eq!(
            client.default_headers.get("Accept"),
            Some(&"text/html".to_string())
        );
        assert_eq!(
            client.default_headers.get("X-Custom-Header"),
            Some(&"custom-value".to_string())
        );
    }

    #[test]
    fn test_http_validators() {
        let mock_result = HttpTestResult {
            response: HttpResponse {
                status: 200,
                headers: {
                    let mut h = HashMap::new();
                    h.insert("content-type".to_string(), "application/json".to_string());
                    h
                },
                body: r#"{"status": "ok", "data": {"id": 1}}"#.to_string(),
                json: Some(json!({"status": "ok", "data": {"id": 1}})),
            },
            execution_time: Duration::from_millis(100),
            request_url: "http://localhost:3000/api/test".to_string(),
            method: "GET".to_string(),
            status_code: 200,
            success: true,
        };

        // Test status validation
        assert!(HttpValidators::assert_status_code(&mock_result, 200).is_ok());
        assert!(HttpValidators::assert_status_code(&mock_result, 404).is_err());

        // Test success validation
        assert!(HttpValidators::assert_success(&mock_result).is_ok());

        // Test performance validation
        assert!(HttpValidators::assert_performance(&mock_result, Duration::from_secs(1)).is_ok());
        assert!(
            HttpValidators::assert_performance(&mock_result, Duration::from_millis(50)).is_err()
        );

        // Test JSON structure validation
        assert!(HttpValidators::assert_json_structure(&mock_result, &["status"]).is_ok());
        assert!(HttpValidators::assert_json_structure(&mock_result, &["nonexistent"]).is_err());

        // Test content type validation
        assert!(HttpValidators::assert_content_type(&mock_result, "application/json").is_ok());
        assert!(HttpValidators::assert_content_type(&mock_result, "text/html").is_err());
    }
}
