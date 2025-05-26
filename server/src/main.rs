use lambda_http::{run, service_fn, Body, Error, Request, RequestExt, Response};
use mcp_template_server::{
    handlers,
    models::mcp::{McpRequest, McpResponse},
    TemplateServer,
};
use once_cell::sync::OnceCell;
use std::sync::Arc;
use tracing::info;

static TEMPLATE_SERVER: OnceCell<Arc<TemplateServer>> = OnceCell::new();

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("mcp_template_server=debug".parse().unwrap()),
        )
        .json()
        .init();

    info!("Starting MCP Template Server Lambda (HTTP)");

    // Initialize the template server
    let server = Arc::new(
        TemplateServer::new()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to initialize template server: {}", e))?,
    );

    // Pre-warm cache during initialization
    server
        .warm_cache()
        .await
        .map_err(|e| anyhow::anyhow!("Cache warming failed: {}", e))?;

    // Store the server in the global static
    TEMPLATE_SERVER.set(server).ok();

    run(service_fn(handler)).await
}

async fn handler(event: Request) -> Result<Response<Body>, Error> {
    let start_time = std::time::Instant::now();

    // Get request ID from Lambda context
    let request_id = event.lambda_context().request_id.clone();

    // Parse the request body as MCP request
    let body = event.body();
    let mcp_request: McpRequest = match serde_json::from_slice(body) {
        Ok(req) => req,
        Err(e) => {
            let error_response = McpResponse::error(
                serde_json::Value::Null,
                -32700,
                format!("Parse error: {}", e),
            );
            return Ok(Response::builder()
                .status(400)
                .header("content-type", "application/json")
                .body(serde_json::to_string(&error_response)?.into())
                .expect("failed to build response"));
        }
    };

    let server = Arc::clone(
        TEMPLATE_SERVER
            .get()
            .expect("Template server not initialized"),
    );

    let mcp_response = handlers::handle_request(server, mcp_request).await;

    let duration = start_time.elapsed();
    info!(
        request_id = %request_id,
        method = %mcp_response.id,
        duration_ms = duration.as_millis(),
        "Request processed"
    );

    // Build HTTP response
    let response = Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .header("x-request-id", request_id)
        .body(serde_json::to_string(&mcp_response)?.into())
        .expect("failed to build response");

    Ok(response)
}
