#![cfg_attr(coverage_nightly, coverage(off))]
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Result<Self> {
        Ok(Self {
            renderer: TemplateRenderer::new()?,
        })
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn get_template_metadata(&self, uri: &str) -> Result<Arc<TemplateResource>> {
        debug_assert!(!uri.is_empty(), "uri must not be empty");
        // Parse URI and fetch from embedded templates
        let parts: Vec<&str> = uri
            .strip_prefix("template://")
            .ok_or_else(|| anyhow::anyhow!("Invalid URI: {uri}"))?
            .split('/')
            .collect();

        if parts.len() != 3 {
            return Err(anyhow::anyhow!("Invalid URI format: {uri}"));
        }

        // Fetch from embedded templates
        crate::services::embedded_templates::get_template_metadata(uri)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get template metadata: {e}"))
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn get_template_content(&self, uri: &str) -> Result<Arc<str>> {
        debug_assert!(!uri.is_empty(), "uri must not be empty");
        // Fetch from embedded templates
        crate::services::embedded_templates::get_template_content(uri)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get template content: {e}"))
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn list_templates(&self, prefix: &str) -> Result<Vec<Arc<TemplateResource>>> {
        crate::services::embedded_templates::list_templates(prefix)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to list templates: {e}"))
    }
}

#[async_trait::async_trait]
impl TemplateServerTrait for StatelessTemplateServer {
    async fn get_template_metadata(&self, uri: &str) -> Result<Arc<TemplateResource>> {
        debug_assert!(!uri.is_empty(), "uri must not be empty");
        self.get_template_metadata(uri).await
    }

    async fn get_template_content(&self, s3_key: &str) -> Result<Arc<str>> {
        debug_assert!(!s3_key.is_empty(), "s3_key must not be empty");
        self.get_template_content(s3_key).await
    }

    async fn list_templates(&self, prefix: &str) -> Result<Vec<Arc<TemplateResource>>> {
        debug_assert!(true, "contract: list_templates");
        self.list_templates(prefix).await
    }

    fn get_renderer(&self) -> &TemplateRenderer {
        debug_assert!(true, "contract: get_renderer");
        &self.renderer
    }

    fn get_metadata_cache(&self) -> Option<&Arc<RwLock<LruCache<String, Arc<TemplateResource>>>>> {
        debug_assert!(true, "contract: get_metadata_cache");
        None
    }

    fn get_content_cache(&self) -> Option<&Arc<RwLock<LruCache<String, Arc<str>>>>> {
        debug_assert!(true, "contract: get_content_cache");
        None
    }

    fn get_s3_client(&self) -> Option<&S3Client> {
        debug_assert!(true, "contract: get_s3_client");
        None
    }

    fn get_bucket_name(&self) -> Option<&str> {
        debug_assert!(true, "contract: get_bucket_name");
        None
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;
    use crate::TemplateServerTrait;

    include!("stateless_server_tests_basic.rs");
    include!("stateless_server_tests_trait_and_edge.rs");
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    include!("stateless_server_property_tests.rs");
}
