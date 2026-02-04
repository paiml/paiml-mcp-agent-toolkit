//! Query Handler - Semantic code search for agents (PMAT-470)
//!
//! Provides RAG-powered code search with quality annotations.
//! Designed as a grep replacement for AI agents.

use crate::cli::QueryOutputFormat;
use crate::services::agent_context::{
    format_json, format_markdown, format_text, AgentContextIndex, QueryOptions,
};
use std::path::PathBuf;

/// Handle the `pmat query` command
///
/// # Arguments
/// * `query` - Natural language query
/// * `limit` - Maximum number of results
/// * `min_grade` - Minimum TDG grade filter
/// * `max_complexity` - Maximum complexity filter
/// * `language` - Language filter
/// * `path_pattern` - File path pattern filter
/// * `project_path` - Project root to search
/// * `format` - Output format
/// * `include_source` - Include full source code
/// * `rebuild_index` - Force rebuild index
pub async fn handle_query(
    query: String,
    limit: usize,
    min_grade: Option<String>,
    max_complexity: Option<u32>,
    language: Option<String>,
    path_pattern: Option<String>,
    project_path: PathBuf,
    format: QueryOutputFormat,
    include_source: bool,
    rebuild_index: bool,
    exclude_tests: bool,
) -> anyhow::Result<()> {
    // Check for existing index
    let index_path = project_path.join(".pmat/context.idx");

    // Suppress status messages for JSON format (issue #145)
    let quiet = matches!(format, QueryOutputFormat::Json);

    let index = if index_path.exists() && !rebuild_index {
        if !quiet {
            eprintln!("Loading index from {:?}...", index_path);
        }
        match AgentContextIndex::load(&index_path) {
            Ok(idx) => idx,
            Err(e) => {
                eprintln!("Failed to load index ({}), rebuilding...", e);
                build_and_save_index(&project_path, &index_path)?
            }
        }
    } else {
        if !quiet {
            eprintln!("Building index for {:?}...", project_path);
        }
        build_and_save_index(&project_path, &index_path)?
    };

    if !quiet {
        let manifest = index.manifest();
        eprintln!(
            "Index: {} functions in {} files (avg TDG: {:.1})",
            manifest.function_count, manifest.file_count, manifest.avg_tdg_score
        );
    }

    // Execute query
    let options = QueryOptions {
        limit,
        min_grade,
        max_complexity,
        max_loc: None,
        language,
        path_pattern,
        include_source,
    };

    let mut results = index
        .query(&query, options)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    // Filter out test functions if requested
    if exclude_tests {
        results.retain(|r| {
            !r.function_name.starts_with("test_")
                && !r.file_path.starts_with("tests/")
                && !r.file_path.contains("/tests/")
                && !r.file_path.contains("_tests.")
                && !r.file_path.contains("_test.")
        });
    }

    if results.is_empty() {
        eprintln!("No matching functions found for: {}", query);
        return Ok(());
    }

    // Format and output results
    let output = match format {
        QueryOutputFormat::Text => format_text(&results),
        QueryOutputFormat::Json => format_json(&results).map_err(|e| anyhow::anyhow!("{}", e))?,
        QueryOutputFormat::Markdown => format_markdown(&results),
    };

    println!("{}", output);

    Ok(())
}

/// Build index and save to disk
fn build_and_save_index(
    project_path: &PathBuf,
    index_path: &PathBuf,
) -> anyhow::Result<AgentContextIndex> {
    let index = AgentContextIndex::build(project_path)
        .map_err(|e| anyhow::anyhow!("Failed to build index: {}", e))?;

    // Create .pmat directory if needed
    if let Some(parent) = index_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Save index
    index
        .save(index_path)
        .map_err(|e| anyhow::anyhow!("Failed to save index: {}", e))?;

    eprintln!("Index saved to {:?}", index_path);

    Ok(index)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_handle_query_empty_project() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_path_buf();

        // Create empty project
        std::fs::create_dir_all(project_path.join("src")).unwrap();
        std::fs::write(project_path.join("src/main.rs"), "").unwrap();

        let result = handle_query(
            "test".to_string(),
            10,
            None,
            None,
            None,
            None,
            project_path,
            QueryOutputFormat::Text,
            false,
            false,
            false,
        )
        .await;

        // Should not error, just find nothing
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_query_with_functions() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_path_buf();

        // Create project with a function
        std::fs::create_dir_all(project_path.join("src")).unwrap();
        std::fs::write(
            project_path.join("src/main.rs"),
            r#"
/// Handle errors in the API layer
fn handle_api_error(err: String) -> String {
    format!("Error: {}", err)
}

fn main() {
    println!("Hello");
}
"#,
        )
        .unwrap();

        let result = handle_query(
            "error handling".to_string(),
            10,
            None,
            None,
            None,
            None,
            project_path,
            QueryOutputFormat::Json,
            false,
            true, // Force rebuild
            false,
        )
        .await;

        assert!(result.is_ok());
    }
}
