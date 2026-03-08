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

impl CommandDispatcher {
    /// Execute embed commands for semantic search (PMAT-SEARCH-011)
    ///
    /// Uses local TF-IDF embeddings - no API keys required.
    pub async fn execute_embed_command(embed_cmd: EmbedCommands) -> anyhow::Result<()> {
        // Load configuration with environment variable fallbacks
        let config_service = ConfigurationService::new(None);
        let semantic_config = config_service.get_semantic_config_with_env_fallback()?;

        // Check if semantic search is enabled
        if !semantic_config.enabled {
            anyhow::bail!(
                "Semantic search is not enabled.\n\
                 To enable, set semantic.enabled = true in config file.\n\
                 No API keys required - uses local embeddings."
            );
        }

        // Get database path
        let db_path = semantic_config.vector_db_path.unwrap_or_else(|| {
            dirs::home_dir()
                .map(|h| {
                    h.join(".pmat")
                        .join("embeddings.db")
                        .to_string_lossy()
                        .to_string()
                })
                .unwrap_or_else(|| "embeddings.db".to_string())
        });

        // Get workspace path
        let workspace = semantic_config
            .workspace_path
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

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
    pub async fn execute_semantic_command(semantic_cmd: SemanticCommands) -> anyhow::Result<()> {
        // Load configuration with environment variable fallbacks
        let config_service = ConfigurationService::new(None);
        let semantic_config = config_service.get_semantic_config_with_env_fallback()?;

        // Check if semantic search is enabled
        if !semantic_config.enabled {
            anyhow::bail!(
                "Semantic search is not enabled.\n\
                 To enable, set semantic.enabled = true in config file.\n\
                 No API keys required - uses local embeddings."
            );
        }

        // Get database path
        let db_path = semantic_config.vector_db_path.unwrap_or_else(|| {
            dirs::home_dir()
                .map(|h| {
                    h.join(".pmat")
                        .join("embeddings.db")
                        .to_string_lossy()
                        .to_string()
                })
                .unwrap_or_else(|| "embeddings.db".to_string())
        });

        // Get workspace path
        let workspace = semantic_config
            .workspace_path
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

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

                let result = semantic_cli
                    .semantic_search(&query, mode_str, limit, language)
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
