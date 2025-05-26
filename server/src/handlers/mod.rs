pub mod initialize;
pub mod prompts;
pub mod resources;
pub mod tools;

use crate::models::mcp::{McpRequest, McpResponse};
use crate::TemplateServerTrait;
use std::sync::Arc;

pub async fn handle_request<T: TemplateServerTrait>(
    server: Arc<T>,
    request: McpRequest,
) -> McpResponse {
    match request.method.as_str() {
        "initialize" => initialize::handle_initialize(server, request).await,
        "tools/list" => initialize::handle_tools_list(server, request).await,
        "tools/call" => tools::handle_tool_call(server, request).await,
        "resources/list" => resources::handle_resource_list(server, request).await,
        "resources/read" => resources::handle_resource_read(server, request).await,
        "prompts/list" => prompts::handle_prompts_list(server, request).await,
        "prompts/get" => prompts::handle_prompt_get(server, request).await,
        _ => McpResponse::error(
            request.id,
            -32601,
            format!("Method not found: {}", request.method),
        ),
    }
}
