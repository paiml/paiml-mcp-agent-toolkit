#![cfg_attr(coverage_nightly, coverage(off))]
//! Utility command definitions (list, search, context, serve, diagnose)

use crate::stateless_server::StatelessTemplateServer;
use anyhow::Result;
use std::sync::Arc;

/// Command group for utility operations (list, search, context, serve)
pub struct UtilityCommandGroup;

impl Default for UtilityCommandGroup {
    fn default() -> Self {
        Self
    }
}

impl UtilityCommandGroup {
    /// Handle list command
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn handle_list(
        &self,
        server: Arc<StatelessTemplateServer>,
        toolchain: Option<String>,
        category: Option<String>,
        format: crate::cli::OutputFormat,
    ) -> Result<()> {
        crate::cli::handlers::utility_handlers::handle_list(server, toolchain, category, format)
            .await
    }

    /// Handle search command
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn handle_search(
        &self,
        server: Arc<StatelessTemplateServer>,
        query: String,
        toolchain: Option<String>,
        limit: usize,
    ) -> Result<()> {
        debug_assert!(limit > 0, "limit must be positive");
        crate::cli::handlers::utility_handlers::handle_search(server, query, toolchain, limit).await
    }

    /// Handle context command
    #[allow(clippy::too_many_arguments)]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub async fn handle_context(
        &self,
        toolchain: Option<String>,
        project_path: std::path::PathBuf,
        output: Option<std::path::PathBuf>,
        format: crate::cli::ContextFormat,
        include_large_files: bool,
        skip_expensive_metrics: bool,
        language: Option<String>,
        languages: Option<Vec<String>>,
    ) -> Result<()> {
        crate::cli::handlers::utility_handlers::handle_context(
            toolchain,
            project_path,
            output,
            format,
            include_large_files,
            skip_expensive_metrics,
            language,
            languages,
        )
        .await
    }

    /// Handle serve command
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn handle_serve(
        &self,
        host: String,
        port: u16,
        cors: bool,
        transport: crate::cli::commands::ServeTransport,
    ) -> Result<()> {
        crate::cli::handlers::utility_handlers::handle_serve(host, port, cors, transport).await
    }

    /// Handle diagnose command
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn handle_diagnose(&self, args: crate::cli::diagnose::DiagnoseArgs) -> Result<()> {
        crate::cli::handlers::utility_handlers::handle_diagnose(args).await
    }
}
