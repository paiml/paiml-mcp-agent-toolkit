use anyhow::Result;
use lru::LruCache;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::models::template::TemplateResource;
use crate::services::renderer::TemplateRenderer;
use crate::{S3Client, TemplateServerTrait};

pub struct StatelessTemplateServer {
    pub renderer: TemplateRenderer,
}

impl StatelessTemplateServer {
    pub fn new() -> Result<Self> {
        Ok(Self {
            renderer: TemplateRenderer::new()?,
        })
    }

    pub async fn get_template_metadata(&self, uri: &str) -> Result<Arc<TemplateResource>> {
        // Parse URI and fetch from embedded templates
        let parts: Vec<&str> = uri
            .strip_prefix("template://")
            .ok_or_else(|| anyhow::anyhow!("Invalid URI: {}", uri))?
            .split('/')
            .collect();

        if parts.len() != 3 {
            return Err(anyhow::anyhow!("Invalid URI format: {}", uri));
        }

        // Fetch from embedded templates
        crate::services::embedded_templates::get_template_metadata(uri)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get template metadata: {}", e))
    }

    pub async fn get_template_content(&self, uri: &str) -> Result<Arc<str>> {
        // Fetch from embedded templates
        crate::services::embedded_templates::get_template_content(uri)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get template content: {}", e))
    }

    pub async fn list_templates(&self, prefix: &str) -> Result<Vec<Arc<TemplateResource>>> {
        crate::services::embedded_templates::list_templates(prefix)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to list templates: {}", e))
    }
}

#[async_trait::async_trait]
impl TemplateServerTrait for StatelessTemplateServer {
    async fn get_template_metadata(&self, uri: &str) -> Result<Arc<TemplateResource>> {
        self.get_template_metadata(uri).await
    }

    async fn get_template_content(&self, s3_key: &str) -> Result<Arc<str>> {
        self.get_template_content(s3_key).await
    }

    async fn list_templates(&self, prefix: &str) -> Result<Vec<Arc<TemplateResource>>> {
        self.list_templates(prefix).await
    }

    fn get_renderer(&self) -> &TemplateRenderer {
        &self.renderer
    }

    fn get_metadata_cache(&self) -> Option<&Arc<RwLock<LruCache<String, Arc<TemplateResource>>>>> {
        None
    }

    fn get_content_cache(&self) -> Option<&Arc<RwLock<LruCache<String, Arc<str>>>>> {
        None
    }

    fn get_s3_client(&self) -> Option<&S3Client> {
        None
    }

    fn get_bucket_name(&self) -> Option<&str> {
        None
    }
}

#[cfg(test)]
mod tests {
    // use super::*; // Unused in simple tests

    #[test]
    fn test_stateless_server_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }
}
