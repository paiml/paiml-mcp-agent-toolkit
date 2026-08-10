//! Semantic and Embed Command Handlers for CommandDispatcher
//!
//! Extracted from command_dispatcher mod.rs for file health compliance (CB-040).
//! Contains semantic search and embedding command execution.
//!
//! NOTE: Uses pure Rust local embeddings via aprender/trueno-rag.
//! NO external API keys or internet connection required.
#![cfg_attr(coverage_nightly, coverage(off))]

use super::CommandDispatcher;
use crate::cli::commands::{EmbedCommands, SearchMode, SemanticCommands};
use crate::cli::semantic_commands::SemanticCli;
use crate::cli::OutputFormat;
use crate::services::configuration_service::ConfigurationService;
use std::path::Path;

/// Where a workspace's embeddings live when the config names no path.
///
/// This used to default to a single machine-global `~/.pmat/embeddings.db`,
/// shared by every project on the machine, while chunk paths are stored
/// workspace-relative (`./src/main.rs`). One project's leftover index was
/// therefore returned for every OTHER project, at paths that do not resolve
/// there: `pmat semantic search` in this repo returned five `./src/main.rs`
/// chunks from an unrelated crate, and an unrelated crate got the same five.
/// Keying the store to the workspace makes a relative chunk path mean something
/// again.
fn default_vector_db_path(workspace: &Path) -> String {
    workspace
        .join(".pmat")
        .join("embeddings.db")
        .to_string_lossy()
        .to_string()
}

impl CommandDispatcher {
    /// Execute embed commands for semantic search (PMAT-SEARCH-011)
    ///
    /// Uses local TF-IDF embeddings - no API keys required.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn execute_embed_command(embed_cmd: EmbedCommands) -> anyhow::Result<()> {
        // Load configuration with environment variable fallbacks
        let config_service = ConfigurationService::new(None);
        let semantic_config = config_service.get_semantic_config_with_env_fallback()?;

        // Issue #563: an explicit `pmat embed …` / `pmat semantic …` invocation
        // IS the opt-in. The local-embedding stack (aprender/trueno-rag) ships
        // in-binary and needs no API key or network, so these commands run out of
        // the box instead of bailing on a `semantic.enabled` flag that isn't
        // surfaced in --help. The toggle still gates passive/auto behavior
        // (auto-sync, MCP tool registration) elsewhere.

        // Get workspace path
        let workspace = semantic_config
            .workspace_path
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

        // Get database path — scoped to the workspace being searched.
        let db_path = semantic_config
            .vector_db_path
            .unwrap_or_else(|| default_vector_db_path(&workspace));

        // Initialize semantic CLI (no API key needed)
        let semantic_cli = SemanticCli::new(&db_path, &workspace)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;

        match embed_cmd {
            EmbedCommands::Sync {
                path,
                language,
                format,
            } => {
                let result = semantic_cli
                    .embed_sync(&path, language)
                    .await
                    .map_err(|e| anyhow::anyhow!(e))?;
                match format {
                    OutputFormat::Json => {
                        println!(
                            "{}",
                            serde_json::json!({"status": "success", "message": result})
                        );
                    }
                    _ => println!("{}", result),
                }
                Ok(())
            }
            EmbedCommands::Status { format } => {
                let result = semantic_cli
                    .embed_status()
                    .await
                    .map_err(|e| anyhow::anyhow!(e))?;
                match format {
                    OutputFormat::Json => {
                        println!(
                            "{}",
                            serde_json::json!({"status": "success", "message": result})
                        );
                    }
                    _ => println!("{}", result),
                }
                Ok(())
            }
            EmbedCommands::Clear { confirm } => {
                let result = semantic_cli
                    .embed_clear(confirm)
                    .await
                    .map_err(|e| anyhow::anyhow!(e))?;
                println!("{}", result);
                Ok(())
            }
        }
    }

    /// Execute semantic search commands (PMAT-SEARCH-011)
    ///
    /// Uses local TF-IDF embeddings - no API keys required.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn execute_semantic_command(semantic_cmd: SemanticCommands) -> anyhow::Result<()> {
        // Load configuration with environment variable fallbacks
        let config_service = ConfigurationService::new(None);
        let semantic_config = config_service.get_semantic_config_with_env_fallback()?;

        // Issue #563: an explicit `pmat embed …` / `pmat semantic …` invocation
        // IS the opt-in. The local-embedding stack (aprender/trueno-rag) ships
        // in-binary and needs no API key or network, so these commands run out of
        // the box instead of bailing on a `semantic.enabled` flag that isn't
        // surfaced in --help. The toggle still gates passive/auto behavior
        // (auto-sync, MCP tool registration) elsewhere.

        // Get workspace path
        let workspace = semantic_config
            .workspace_path
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

        // Get database path — scoped to the workspace being searched.
        let db_path = semantic_config
            .vector_db_path
            .unwrap_or_else(|| default_vector_db_path(&workspace));

        // Initialize semantic CLI (no API key needed)
        let semantic_cli = SemanticCli::new(&db_path, &workspace)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;

        match semantic_cmd {
            SemanticCommands::Search {
                query,
                search_mode,
                language,
                limit,
                format,
            } => {
                // Convert SearchMode to string
                let mode_str = match search_mode {
                    SearchMode::Keyword => "keyword",
                    SearchMode::Vector => "vector",
                    SearchMode::Hybrid => "hybrid",
                };

                let output = semantic_cli
                    .semantic_search_results(&query, mode_str, limit, language)
                    .await
                    .map_err(|e| anyhow::anyhow!(e))?;
                match format {
                    // JSON purity: stdout carries only the payload
                    OutputFormat::Json => {
                        println!("{}", serde_json::to_string_pretty(&output.to_json())?)
                    }
                    _ => println!("{}", output.render_text()),
                }
                Ok(())
            }
            SemanticCommands::Similar {
                file_path,
                limit,
                format,
            } => {
                let result = semantic_cli
                    .semantic_similar(&file_path, limit)
                    .await
                    .map_err(|e| anyhow::anyhow!(e))?;
                match format {
                    OutputFormat::Json => {
                        println!("{}", result); // Result is already JSON
                    }
                    _ => println!("{}", result),
                }
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod vector_store_scoping_tests {
    use super::*;

    /// Two workspaces must not share one embeddings store: with the old
    /// machine-global `~/.pmat/embeddings.db` default, `pmat semantic search`
    /// returned another project's chunks at `./src/main.rs` — a path that does
    /// not exist in the project being searched.
    #[test]
    fn test_default_store_is_per_workspace() {
        let a = default_vector_db_path(Path::new("/home/u/projects/alpha"));
        let b = default_vector_db_path(Path::new("/home/u/projects/beta"));

        assert_ne!(a, b, "two projects shared one embeddings store");
        assert!(a.starts_with("/home/u/projects/alpha/"), "{a}");
        assert!(b.starts_with("/home/u/projects/beta/"), "{b}");
        assert!(a.ends_with("embeddings.db"), "{a}");
    }

    /// The store lives beside the project's other pmat state.
    #[test]
    fn test_default_store_lives_under_dot_pmat() {
        let p = default_vector_db_path(Path::new("/w"));
        assert_eq!(p, "/w/.pmat/embeddings.db");
    }
}
