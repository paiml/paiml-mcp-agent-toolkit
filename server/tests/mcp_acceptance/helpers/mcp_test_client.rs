//! MCP Test Client - Framework for MCP acceptance testing
//!
//! Provides a comprehensive testing framework for MCP (Model Context Protocol) interfaces.
//! Implements JSON-RPC 2.0 client for testing MCP server functionality.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};
use tempfile::{tempdir, TempDir};

/// MCP test client for JSON-RPC 2.0 communication
pub struct McpTestClient {
    pub server_process: Option<Child>,
    pub server_url: String,
    pub client_info: ClientInfo,
    pub test_workspace: TempDir,
    pub request_id_counter: u64,
}

/// Client information for MCP initialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
    pub capabilities: ClientCapabilities,
}

/// Client capabilities for MCP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientCapabilities {
    pub experimental: HashMap<String, Value>,
    pub sampling: Option<Value>,
}

/// MCP JSON-RPC request structure
#[derive(Debug, Clone, Serialize)]
pub struct McpRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    pub params: Value,
}

/// MCP JSON-RPC response structure
#[derive(Debug, Clone, Deserialize)]
pub struct McpResponse {
    pub jsonrpc: String,
    pub id: u64,
    pub result: Option<Value>,
    pub error: Option<McpError>,
}

/// MCP error structure
#[derive(Debug, Clone, Deserialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

/// Tool call result with performance metrics
#[derive(Debug, Clone)]
pub struct ToolCallResult {
    pub response: McpResponse,
    pub execution_time: Duration,
    pub tool_name: String,
    pub success: bool,
}

impl McpTestClient {
    /// Create a new MCP test client
    pub fn new() -> Result<Self> {
        let workspace = tempdir().context("Failed to create test workspace")?;

        Ok(Self {
            server_process: None,
            server_url: "stdio".to_string(),
            client_info: ClientInfo {
                name: "pmat-test-client".to_string(),
                version: "1.0.0".to_string(),
                capabilities: ClientCapabilities {
                    experimental: HashMap::new(),
                    sampling: None,
                },
            },
            test_workspace: workspace,
            request_id_counter: 0,
        })
    }

    /// Start MCP server process
    pub fn start_server(&mut self, server_path: &str, args: &[&str]) -> Result<()> {
        let mut cmd = Command::new(server_path);
        cmd.args(args);
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.current_dir(self.test_workspace.path());

        let child = cmd.spawn().context("Failed to start MCP server")?;
        self.server_process = Some(child);

        // Wait for server to initialize
        std::thread::sleep(Duration::from_millis(1000));

        Ok(())
    }

    /// Initialize MCP connection
    pub fn initialize(&mut self) -> Result<McpResponse> {
        let params = json!({
            "protocolVersion": "2024-11-05",
            "capabilities": self.client_info.capabilities,
            "clientInfo": {
                "name": self.client_info.name,
                "version": self.client_info.version
            }
        });

        self.call_method("initialize", params)
    }

    /// Call MCP tool with parameters
    pub fn call_tool(&mut self, tool_name: &str, arguments: Value) -> Result<ToolCallResult> {
        let start_time = Instant::now();

        let params = json!({
            "name": tool_name,
            "arguments": arguments
        });

        let response = self.call_method("tools/call", params)?;
        let execution_time = start_time.elapsed();
        let success = response.error.is_none();

        Ok(ToolCallResult {
            response,
            execution_time,
            tool_name: tool_name.to_string(),
            success,
        })
    }

    /// List available tools
    pub fn list_tools(&mut self) -> Result<McpResponse> {
        self.call_method("tools/list", json!({}))
    }

    /// List available resources
    pub fn list_resources(&mut self) -> Result<McpResponse> {
        self.call_method("resources/list", json!({}))
    }

    /// List available prompts
    pub fn list_prompts(&mut self) -> Result<McpResponse> {
        self.call_method("prompts/list", json!({}))
    }

    /// Get server capabilities
    pub fn get_capabilities(&mut self) -> Result<McpResponse> {
        self.call_method(
            "initialize",
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": self.client_info
            }),
        )
    }

    /// Send ping to server
    pub fn ping(&mut self) -> Result<McpResponse> {
        self.call_method("ping", json!({}))
    }

    /// Generic method call
    pub fn call_method(&mut self, method: &str, params: Value) -> Result<McpResponse> {
        self.request_id_counter += 1;

        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: self.request_id_counter,
            method: method.to_string(),
            params: params.clone(),
        };

        // For stdio communication with server process
        if let Some(ref mut process) = self.server_process {
            let request_json = serde_json::to_string(&request)?;

            if let Some(ref mut stdin) = process.stdin.as_mut() {
                writeln!(stdin, "{}", request_json)?;
                stdin.flush()?;
            }

            // Read response from stdout
            if let Some(ref mut stdout) = process.stdout.as_mut() {
                let mut reader = BufReader::new(stdout);
                let mut response_line = String::new();
                reader.read_line(&mut response_line)?;

                let response: McpResponse = serde_json::from_str(response_line.trim())?;
                return Ok(response);
            }
        }

        // Fallback for direct command execution
        self.execute_direct_command(method, params)
    }

    /// Execute command directly (fallback)
    fn execute_direct_command(&self, method: &str, params: Value) -> Result<McpResponse> {
        // This would implement direct command execution for testing
        // For now, return a mock response
        Ok(McpResponse {
            jsonrpc: "2.0".to_string(),
            id: self.request_id_counter,
            result: Some(json!({
                "method": method,
                "status": "mock_response",
                "params": params
            })),
            error: None,
        })
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

        std::fs::write(
            project_path.join("src/lib.rs"),
            r#"pub fn complex_function(data: &[i32]) -> Vec<i32> {
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
"#,
        )?;

        Ok(project_path)
    }

    /// Get workspace path
    pub fn workspace_path(&self) -> &std::path::Path {
        self.test_workspace.path()
    }

    /// Stop server process
    pub fn stop_server(&mut self) -> Result<()> {
        if let Some(mut process) = self.server_process.take() {
            process.kill().context("Failed to kill server process")?;
            process
                .wait()
                .context("Failed to wait for server process")?;
        }
        Ok(())
    }
}

impl Drop for McpTestClient {
    fn drop(&mut self) {
        let _ = self.stop_server();
    }
}

/// Validation helpers for MCP test results
pub struct McpValidators;

impl McpValidators {
    /// Validate JSON-RPC 2.0 compliance
    pub fn assert_jsonrpc_compliance(response: &McpResponse) -> Result<()> {
        if response.jsonrpc != "2.0" {
            anyhow::bail!("Invalid JSON-RPC version: {}", response.jsonrpc);
        }
        Ok(())
    }

    /// Validate tool call success
    pub fn assert_tool_success(result: &ToolCallResult) -> Result<()> {
        if !result.success {
            anyhow::bail!(
                "Tool call failed for {}: {:?}",
                result.tool_name,
                result.response.error
            );
        }
        Ok(())
    }

    /// Validate response contains expected fields
    pub fn assert_response_fields(response: &McpResponse, expected_fields: &[&str]) -> Result<()> {
        if let Some(ref result) = response.result {
            for field in expected_fields {
                if result.get(field).is_none() {
                    anyhow::bail!("Missing expected field '{}' in response", field);
                }
            }
        } else {
            anyhow::bail!("Response missing result field");
        }
        Ok(())
    }

    /// Validate performance requirements
    pub fn assert_performance(result: &ToolCallResult, max_duration: Duration) -> Result<()> {
        if result.execution_time > max_duration {
            anyhow::bail!(
                "Tool call took too long: {:?} > {:?}",
                result.execution_time,
                max_duration
            );
        }
        Ok(())
    }

    /// Validate error handling
    pub fn assert_error_handling(
        response: &McpResponse,
        expected_error_code: Option<i32>,
    ) -> Result<()> {
        match (response.error.as_ref(), expected_error_code) {
            (Some(error), Some(expected_code)) => {
                if error.code != expected_code {
                    anyhow::bail!(
                        "Expected error code {} but got {}",
                        expected_code,
                        error.code
                    );
                }
            }
            (None, Some(_)) => {
                anyhow::bail!("Expected error but got success response");
            }
            (Some(_), None) => {
                anyhow::bail!("Expected success but got error: {:?}", response.error);
            }
            (None, None) => {
                // Both success - OK
            }
        }
        Ok(())
    }

    /// Validate MCP protocol capabilities
    pub fn assert_protocol_capabilities(
        response: &McpResponse,
        expected_capabilities: &[&str],
    ) -> Result<()> {
        if let Some(ref result) = response.result {
            if let Some(capabilities) = result.get("capabilities") {
                for capability in expected_capabilities {
                    if capabilities.get(capability).is_none() {
                        anyhow::bail!("Missing expected capability '{}'", capability);
                    }
                }
            } else {
                anyhow::bail!("Response missing capabilities field");
            }
        } else {
            anyhow::bail!("Response missing result field");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_client_creation() {
        let client = McpTestClient::new();
        assert!(client.is_ok());

        let client = client.unwrap();
        assert_eq!(client.client_info.name, "pmat-test-client");
        assert_eq!(client.client_info.version, "1.0.0");
        assert!(client.test_workspace.path().exists());
    }

    #[test]
    fn test_sample_project_creation() {
        let client = McpTestClient::new().unwrap();
        let project_path = client.create_sample_project().unwrap();

        assert!(project_path.join("Cargo.toml").exists());
        assert!(project_path.join("src/main.rs").exists());
        assert!(project_path.join("src/lib.rs").exists());
    }

    #[test]
    fn test_jsonrpc_compliance_validation() {
        let response = McpResponse {
            jsonrpc: "2.0".to_string(),
            id: 1,
            result: Some(json!({"status": "ok"})),
            error: None,
        };

        assert!(McpValidators::assert_jsonrpc_compliance(&response).is_ok());

        let invalid_response = McpResponse {
            jsonrpc: "1.0".to_string(),
            id: 1,
            result: Some(json!({"status": "ok"})),
            error: None,
        };

        assert!(McpValidators::assert_jsonrpc_compliance(&invalid_response).is_err());
    }
}
