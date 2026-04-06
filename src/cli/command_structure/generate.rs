#![cfg_attr(coverage_nightly, coverage(off))]
//! Generation-related command definitions (generate, scaffold, validate)

use crate::stateless_server::StatelessTemplateServer;
use anyhow::Result;
use std::sync::Arc;

/// Command group for generation operations (generate, scaffold, validate)
pub struct GenerateCommandGroup;

impl Default for GenerateCommandGroup {
    fn default() -> Self {
        Self
    }
}

impl GenerateCommandGroup {
    /// Handle generate command with modular implementation
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub async fn handle_generate(
        &self,
        server: Arc<StatelessTemplateServer>,
        category: String,
        template: String,
        params: Vec<(String, serde_json::Value)>,
        output: Option<std::path::PathBuf>,
        create_dirs: bool,
    ) -> Result<()> {
        // Delegate to generation handlers module
        crate::cli::handlers::generation_handlers::handle_generate(
            server,
            category,
            template,
            params,
            output,
            create_dirs,
        )
        .await
    }

    /// Handle scaffold command
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn handle_scaffold(
        &self,
        server: Arc<StatelessTemplateServer>,
        toolchain: String,
        templates: Vec<String>,
        params: Vec<(String, serde_json::Value)>,
        parallel: usize,
    ) -> Result<()> {
        crate::cli::handlers::generation_handlers::handle_scaffold(
            server, toolchain, templates, params, parallel,
        )
        .await
    }

    /// Handle validate command
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn handle_validate(
        &self,
        server: Arc<StatelessTemplateServer>,
        uri: String,
        params: Vec<(String, serde_json::Value)>,
    ) -> Result<()> {
        crate::cli::handlers::generation_handlers::handle_validate(server, uri, params).await
    }
}
