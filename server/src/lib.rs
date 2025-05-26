pub mod cli;
pub mod handlers;
pub mod models;
pub mod services;
pub mod stateless_server;
pub mod utils;

use anyhow::Result;
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

use crate::models::template::TemplateResource;
use crate::services::renderer::TemplateRenderer;

// Type aliases to reduce complexity
type MetadataCache = Arc<RwLock<LruCache<String, Arc<TemplateResource>>>>;
type ContentCache = Arc<RwLock<LruCache<String, Arc<str>>>>;

// Dummy type for S3Client to satisfy trait requirements without AWS SDK
pub struct S3Client;

#[async_trait::async_trait]
pub trait TemplateServerTrait: Send + Sync {
    async fn get_template_metadata(&self, uri: &str) -> Result<Arc<TemplateResource>>;
    async fn get_template_content(&self, s3_key: &str) -> Result<Arc<str>>;
    async fn list_templates(&self, prefix: &str) -> Result<Vec<Arc<TemplateResource>>>;
    fn get_renderer(&self) -> &TemplateRenderer;
    fn get_metadata_cache(&self) -> Option<&MetadataCache>;
    fn get_content_cache(&self) -> Option<&ContentCache>;
    fn get_s3_client(&self) -> Option<&S3Client>;
    fn get_bucket_name(&self) -> Option<&str>;
}

pub struct TemplateServer {
    pub s3_client: S3Client,
    pub bucket_name: String,
    pub metadata_cache: MetadataCache,
    pub content_cache: ContentCache,
    pub renderer: TemplateRenderer,
}

impl TemplateServer {
    pub async fn new() -> Result<Self> {
        // Dummy implementation for Lambda compatibility
        // The stateless server should be used instead
        let cache_size = 1024;

        Ok(Self {
            s3_client: S3Client,
            bucket_name: "dummy".to_string(),
            metadata_cache: Arc::new(RwLock::new(LruCache::new(
                NonZeroUsize::new(cache_size / 2).unwrap(),
            ))),
            content_cache: Arc::new(RwLock::new(LruCache::new(
                NonZeroUsize::new(cache_size).unwrap(),
            ))),
            renderer: TemplateRenderer::new()?,
        })
    }

    pub async fn warm_cache(&self) -> Result<()> {
        let common_templates = vec![
            "template://makefile/rust/cli-binary",
            "template://makefile/deno/cli-application",
            "template://makefile/python-uv/cli-application",
            "template://readme/rust/cli-application",
            "template://gitignore/rust/embedded-target",
        ];

        info!(
            "Warming cache with {} common templates",
            common_templates.len()
        );

        for template_uri in common_templates {
            match self.get_template_metadata(template_uri).await {
                Ok(resource) => {
                    let _ = self.get_template_content(&resource.s3_object_key).await;
                }
                Err(e) => {
                    info!("Failed to warm cache for {}: {}", template_uri, e);
                }
            }
        }

        Ok(())
    }

    pub async fn get_template_metadata(&self, _uri: &str) -> Result<Arc<TemplateResource>> {
        // Dummy implementation - use StatelessTemplateServer instead
        Err(anyhow::anyhow!(
            "TemplateServer with S3 is deprecated. Use StatelessTemplateServer instead."
        ))
    }

    pub async fn get_template_content(&self, _s3_key: &str) -> Result<Arc<str>> {
        // Dummy implementation - use StatelessTemplateServer instead
        Err(anyhow::anyhow!(
            "TemplateServer with S3 is deprecated. Use StatelessTemplateServer instead."
        ))
    }
}

#[async_trait::async_trait]
impl TemplateServerTrait for TemplateServer {
    async fn get_template_metadata(&self, uri: &str) -> Result<Arc<TemplateResource>> {
        self.get_template_metadata(uri).await
    }

    async fn get_template_content(&self, s3_key: &str) -> Result<Arc<str>> {
        self.get_template_content(s3_key).await
    }

    async fn list_templates(&self, _prefix: &str) -> Result<Vec<Arc<TemplateResource>>> {
        // Dummy implementation - use StatelessTemplateServer instead
        Err(anyhow::anyhow!(
            "TemplateServer with S3 is deprecated. Use StatelessTemplateServer instead."
        ))
    }

    fn get_renderer(&self) -> &TemplateRenderer {
        &self.renderer
    }

    fn get_metadata_cache(&self) -> Option<&MetadataCache> {
        Some(&self.metadata_cache)
    }

    fn get_content_cache(&self) -> Option<&ContentCache> {
        Some(&self.content_cache)
    }

    fn get_s3_client(&self) -> Option<&S3Client> {
        Some(&self.s3_client)
    }

    fn get_bucket_name(&self) -> Option<&str> {
        Some(&self.bucket_name)
    }
}

// Public exports for CLI consumption
pub use models::error::TemplateError;
pub use models::template::{ParameterSpec, ParameterType};
pub use services::template_service::{
    generate_template, list_templates, scaffold_project, search_templates, validate_template,
};

// MCP server runner function
pub async fn run_mcp_server<T: TemplateServerTrait + 'static>(server: Arc<T>) -> Result<()> {
    use crate::models::mcp::{McpRequest, McpResponse};
    use std::io::{self, BufRead, Write};
    use tracing::{error, info};

    info!("MCP server ready, waiting for requests on stdin...");

    // Read JSON-RPC requests from stdin
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = line?;

        // Skip empty lines
        if line.trim().is_empty() {
            continue;
        }

        // Parse the JSON-RPC request
        match serde_json::from_str::<McpRequest>(&line) {
            Ok(request) => {
                info!(
                    "Received request: method={}, id={:?}",
                    request.method, request.id
                );

                // Handle the request using the existing handler
                let response = handlers::handle_request(Arc::clone(&server), request).await;

                // Write response to stdout
                let response_json = serde_json::to_string(&response)?;
                writeln!(stdout, "{}", response_json)?;
                stdout.flush()?;
            }
            Err(e) => {
                error!("Failed to parse JSON-RPC request: {}", e);

                // Send error response
                let error_response = McpResponse::error(
                    serde_json::Value::Null,
                    -32700,
                    format!("Parse error: {}", e),
                );

                let response_json = serde_json::to_string(&error_response)?;
                writeln!(stdout, "{}", response_json)?;
                stdout.flush()?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    #[path = "../tests/template_rendering.rs"]
    mod template_rendering;

    #[path = "../tests/claude_code_e2e.rs"]
    mod claude_code_e2e;

    #[path = "../tests/mcp_protocol.rs"]
    mod mcp_protocol;

    #[path = "../tests/template_resources.rs"]
    mod template_resources;

    #[path = "../tests/prompts.rs"]
    mod prompts;

    #[path = "../tests/error.rs"]
    mod error;

    #[path = "../tests/cache.rs"]
    mod cache;

    #[path = "../tests/tools.rs"]
    mod tools;

    #[path = "../tests/resources.rs"]
    mod resources;

    #[path = "../tests/lib.rs"]
    mod lib_tests;

    #[path = "../tests/build_naming_validation.rs"]
    mod build_naming_validation;
}
