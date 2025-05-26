use crate::models::mcp::{McpRequest, McpResponse, ResourceReadParams};
use crate::services::template_service;
use crate::TemplateServerTrait;
use serde_json::json;
use std::sync::Arc;
use tracing::error;

pub async fn handle_resource_list<T: TemplateServerTrait>(
    server: Arc<T>,
    request: McpRequest,
) -> McpResponse {
    match template_service::list_all_resources(server.as_ref()).await {
        Ok(resources) => {
            let resource_list: Vec<_> = resources
                .into_iter()
                .map(|r| {
                    json!({
                        "uri": r.uri,
                        "name": r.name,
                        "description": r.description,
                        "mimeType": "text/x-handlebars-template",
                    })
                })
                .collect();

            let result = json!({
                "resources": resource_list
            });
            McpResponse::success(request.id, result)
        }
        Err(e) => {
            error!("Resource listing failed: {}", e);
            McpResponse::error(
                request.id,
                -32000,
                format!("Failed to list resources: {}", e),
            )
        }
    }
}

pub async fn handle_resource_read<T: TemplateServerTrait>(
    server: Arc<T>,
    request: McpRequest,
) -> McpResponse {
    let params = match request.params {
        Some(p) => p,
        None => {
            return McpResponse::error(
                request.id,
                -32602,
                "Invalid params: missing resource read parameters".to_string(),
            );
        }
    };

    let read_params: ResourceReadParams = match serde_json::from_value(params) {
        Ok(p) => p,
        Err(e) => {
            return McpResponse::error(request.id, -32602, format!("Invalid params: {}", e));
        }
    };

    // Get the raw template content
    match template_service::get_template_content(server.as_ref(), &read_params.uri).await {
        Ok(content) => {
            let result = json!({
                "contents": [{
                    "uri": read_params.uri,
                    "mimeType": "text/x-handlebars-template",
                    "text": content
                }]
            });
            McpResponse::success(request.id, result)
        }
        Err(e) => {
            error!("Failed to read resource {}: {}", read_params.uri, e);
            McpResponse::error(
                request.id,
                -32000,
                format!("Failed to read resource: {}", e),
            )
        }
    }
}
