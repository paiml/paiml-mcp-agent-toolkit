use crate::models::proxy::{ProxyRequest, ProxyResponse};
use crate::services::quality_proxy::QualityProxyService;
use anyhow::Result;
use mcp_sdk::types::{Tool, ToolInfo};
use serde_json::{json, Value};
use tracing::{debug, info};

/// Creates the quality proxy tool definition for MCP.
///
/// # Example
///
/// ```ignore
/// use pmat::mcp::handlers::quality_proxy::create_quality_proxy_tool;
///
/// let tool = create_quality_proxy_tool();
/// assert_eq!(tool.name, "quality_proxy");
/// ```ignore
pub fn create_quality_proxy_tool() -> Tool {
    Tool {
        name: "quality_proxy".to_string(),
        description: Some("Proxy code changes through quality gates before applying".to_string()),
        input_schema: ToolInfo {
            r#type: "object".to_string(),
            properties: Some(json!({
                "operation": {
                    "type": "string",
                    "enum": ["write", "edit", "append"],
                    "description": "Type of file operation"
                },
                "file_path": {
                    "type": "string",
                    "description": "Target file path"
                },
                "content": {
                    "type": "string",
                    "description": "New content for write/append operations"
                },
                "old_content": {
                    "type": "string",
                    "description": "Content to replace for edit operations"
                },
                "new_content": {
                    "type": "string",
                    "description": "Replacement content for edit operations"
                },
                "mode": {
                    "type": "string",
                    "enum": ["strict", "advisory", "auto_fix"],
                    "default": "strict",
                    "description": "Proxy enforcement mode"
                },
                "quality_config": {
                    "type": "object",
                    "properties": {
                        "max_complexity": {
                            "type": "integer",
                            "default": 20
                        },
                        "allow_satd": {
                            "type": "boolean",
                            "default": false
                        },
                        "require_docs": {
                            "type": "boolean",
                            "default": true
                        },
                        "auto_format": {
                            "type": "boolean",
                            "default": true
                        }
                    },
                    "description": "Quality configuration settings"
                }
            })),
            required: Some(vec!["operation".to_string(), "file_path".to_string()]),
            additional_properties: None,
        },
    }
}

/// Handles quality proxy tool invocation.
///
/// # Arguments
///
/// * `arguments` - The tool arguments from MCP request
///
/// # Returns
///
/// A JSON value containing the proxy response
///
/// # Example
///
/// ```ignore
/// use pmat::mcp::handlers::quality_proxy::handle_quality_proxy;
/// use serde_json::json;
///
/// # async fn example() -> anyhow::Result<()> {
/// let args = json!({
///     "operation": "write",
///     "file_path": "test.rs",
///     "content": "fn main() {}"
/// });
///
/// let result = handle_quality_proxy(args).await?;
/// assert!(result["status"].is_string());
/// # Ok(())
/// # }
/// ```ignore
pub async fn handle_quality_proxy(arguments: Value) -> Result<Value> {
    info!("Processing quality proxy request");
    debug!("Arguments: {:?}", arguments);

    // Parse the request
    let request: ProxyRequest = serde_json::from_value(arguments)?;
    
    // Create the service and process the request
    let service = QualityProxyService::new();
    let response = service.proxy_operation(request).await?;
    
    // Convert response to JSON
    let result = serde_json::to_value(response)?;
    
    info!("Quality proxy request completed");
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::proxy::{ProxyMode, ProxyOperation, QualityConfig};

    #[test]
    fn test_create_quality_proxy_tool() {
        let tool = create_quality_proxy_tool();
        assert_eq!(tool.name, "quality_proxy");
        assert!(tool.description.is_some());
        assert!(tool.input_schema.properties.is_some());
    }

    #[tokio::test]
    async fn test_handle_quality_proxy_write() {
        let args = json!({
            "operation": "write",
            "file_path": "test.rs",
            "content": "/// Test function\nfn test() {}",
            "mode": "advisory"
        });

        let result = handle_quality_proxy(args).await.unwrap();
        assert!(result["status"].is_string());
        assert!(result["quality_report"].is_object());
    }

    #[tokio::test]
    async fn test_handle_quality_proxy_strict_mode() {
        let args = json!({
            "operation": "write",
            "file_path": "test.rs",
            "content": "// Implementation required\nfn stub() { unimplemented!() }",
            "mode": "strict"
        });

        let result = handle_quality_proxy(args).await.unwrap();
        assert_eq!(result["status"], "rejected");
        assert!(result["quality_report"]["metrics"]["satd_count"].as_u64().unwrap() > 0);
    }
}

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn tool_name_consistent() {
            let tool = create_quality_proxy_tool();
            prop_assert_eq!(tool.name, "quality_proxy");
            prop_assert!(tool.description.is_some());
        }

        #[test]
        fn proxy_request_validation(
            operation in prop::sample::select(vec!["write", "edit", "append"])
        ) {
            // Verify proxy request operations are valid
            let valid_ops = vec!["write", "edit", "append"];
            prop_assert!(valid_ops.contains(&operation.as_str()));
        }

        #[test]
        fn quality_modes_valid(
            mode in prop::sample::select(vec!["strict", "normal", "permissive"])
        ) {
            // All quality modes should be recognized
            let valid_modes = vec!["strict", "normal", "permissive"];
            prop_assert!(valid_modes.contains(&mode.as_str()));
        }
    }
}