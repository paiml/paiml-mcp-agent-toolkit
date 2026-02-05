//! Query Handler - Semantic code search for agents (PMAT-470)
//!
//! Provides RAG-powered code search with quality annotations.
//! Designed as a grep replacement for AI agents.

use crate::cli::QueryOutputFormat;
use crate::services::agent_context::{
    enrich_results_with_churn, enrich_results_with_duplicates, enrich_results_with_entropy,
    enrich_results_with_faults, format_json, format_markdown, format_text, format_text_with_code,
    AgentContextIndex, QueryOptions, RankBy,
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
/// * `rank_by` - Ranking strategy (relevance, pagerank, centrality, indegree)
/// * `min_pagerank` - Minimum PageRank score filter
/// * `include_project` - Additional project paths to include in search
/// * `churn` - Enrich results with git churn data (commit count, volatility)
/// * `duplicates` - Enrich results with duplicate code detection
/// * `entropy` - Enrich results with entropy/pattern diversity metrics
/// * `faults` - Enrich results with batuta fault pattern annotations
/// * `definition_type` - Filter by definition type (fn, struct, enum, trait, type)
/// * `code` - Show source code inline (default: true, use --summary to disable)
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
    rank_by: Option<String>,
    min_pagerank: Option<f32>,
    include_project: Vec<PathBuf>,
    churn: bool,
    duplicates: bool,
    entropy: bool,
    faults: bool,
    definition_type: Option<String>,
    code: bool,
) -> anyhow::Result<()> {
    // Check for existing index
    let index_path = project_path.join(".pmat/context.idx");
    let workspace_idx = project_path.join(".pmat/workspace.idx");

    // Suppress status messages for JSON format (issue #145)
    let quiet = matches!(format, QueryOutputFormat::Json);

    // Auto-discover sibling projects with indexes (check early for workspace fast path)
    let mut siblings = AgentContextIndex::discover_sibling_indexes(&project_path);

    // Add explicitly included projects (--include-project option)
    for project in &include_project {
        let idx_path = project.join(".pmat/context.idx");
        if idx_path.exists() {
            let name = project
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| project.display().to_string());
            // Avoid duplicates
            if !siblings.iter().any(|(_, n)| n == &name) {
                siblings.push((idx_path, name));
            }
        } else if !quiet {
            eprintln!(
                "Warning: No index at {:?}, run 'pmat query --rebuild-index' in that project first",
                idx_path
            );
        }
    }

    // Fast path: if workspace cache is fresh, load directly without checking local index
    let index = if !siblings.is_empty()
        && !rebuild_index
        && is_workspace_cache_fresh(&workspace_idx, &siblings, &index_path)
    {
        if !quiet {
            eprintln!("Loading cached workspace index...");
        }
        match AgentContextIndex::load(&workspace_idx) {
            Ok(cached) => cached,
            Err(_) => {
                // Cache corrupted, fall back to normal path
                load_and_merge_index(
                    &project_path,
                    &index_path,
                    &workspace_idx,
                    &siblings,
                    rebuild_index,
                    quiet,
                )?
            }
        }
    } else {
        // Normal path: load local, incremental update, merge if needed
        load_and_merge_index(
            &project_path,
            &index_path,
            &workspace_idx,
            &siblings,
            rebuild_index,
            quiet,
        )?
    };

    if !quiet {
        let manifest = index.manifest();
        eprintln!(
            "Index: {} functions in {} files (avg TDG: {:.1})",
            manifest.function_count, manifest.file_count, manifest.avg_tdg_score
        );
    }

    // Parse rank_by option
    let rank_by_enum = match rank_by {
        Some(ref s) => s.parse::<RankBy>().unwrap_or_default(),
        None => RankBy::default(),
    };

    // Execute query (--code implies --include-source)
    let options = QueryOptions {
        limit,
        min_grade,
        max_complexity,
        max_loc: None,
        language,
        path_pattern,
        include_source: include_source || code,
        rank_by: rank_by_enum,
        min_pagerank,
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

    // Filter by definition type if requested
    if let Some(ref def_type) = definition_type {
        let def_type_lower = def_type.to_lowercase();
        let filter_type = match def_type_lower.as_str() {
            "fn" | "func" | "function" => "function".to_string(),
            "struct" | "structs" => "struct".to_string(),
            "enum" | "enums" => "enum".to_string(),
            "trait" | "traits" => "trait".to_string(),
            "type" | "types" | "typealias" => "typealias".to_string(),
            other => other.to_string(),
        };
        results.retain(|r| r.definition_type == filter_type);
    }

    // Enrich with git churn data if requested
    if churn && !results.is_empty() {
        if !quiet {
            eprintln!("Computing git churn metrics...");
        }
        if let Err(e) = enrich_results_with_churn(&mut results, &project_path, 90).await {
            if !quiet {
                eprintln!("Warning: Could not compute churn: {}", e);
            }
        }
    }

    // Enrich with duplicate detection if requested
    if duplicates && !results.is_empty() {
        if !quiet {
            eprintln!("Detecting code duplicates...");
        }
        if let Err(e) = enrich_results_with_duplicates(&mut results, &project_path).await {
            if !quiet {
                eprintln!("Warning: Could not detect duplicates: {}", e);
            }
        }
    }

    // Enrich with entropy/pattern diversity if requested
    if entropy && !results.is_empty() {
        if !quiet {
            eprintln!("Computing pattern diversity...");
        }
        if let Err(e) = enrich_results_with_entropy(&mut results, &project_path).await {
            if !quiet {
                eprintln!("Warning: Could not compute entropy: {}", e);
            }
        }
    }

    // Enrich with batuta fault pattern annotations if requested
    if faults && !results.is_empty() {
        if !quiet {
            eprintln!("Detecting fault patterns (batuta)...");
        }
        if let Err(e) = enrich_results_with_faults(&mut results, &project_path).await {
            if !quiet {
                eprintln!("Warning: Could not detect faults: {}", e);
            }
        }
    }

    if results.is_empty() {
        eprintln!("No matching functions found for: {}", query);
        return Ok(());
    }

    // Format and output results
    let output = match format {
        QueryOutputFormat::Text => {
            if code {
                format_text_with_code(&results)
            } else {
                format_text(&results)
            }
        }
        QueryOutputFormat::Json => format_json(&results).map_err(|e| anyhow::anyhow!("{}", e))?,
        QueryOutputFormat::Markdown => format_markdown(&results),
    };

    println!("{}", output);

    Ok(())
}

/// Load local index, do incremental update if needed, and merge siblings.
fn load_and_merge_index(
    project_path: &PathBuf,
    index_path: &PathBuf,
    workspace_idx: &std::path::Path,
    siblings: &[(PathBuf, String)],
    rebuild_index: bool,
    quiet: bool,
) -> anyhow::Result<AgentContextIndex> {
    let mut index = if index_path.exists() && !rebuild_index {
        if !quiet {
            eprintln!("Loading index from {:?}...", index_path);
        }
        match AgentContextIndex::load(index_path) {
            Ok(existing) => {
                // Try incremental update if checksums are available
                if !existing.manifest().file_checksums.is_empty() {
                    if !quiet {
                        eprintln!("Checking for incremental updates...");
                    }
                    match AgentContextIndex::build_incremental(project_path, &existing) {
                        Ok(updated) => {
                            // Only save if there were actual changes
                            if updated.manifest().last_incremental_changes > 0 {
                                let _ = updated.save(index_path);
                            }
                            updated
                        }
                        Err(_) => existing,
                    }
                } else {
                    existing
                }
            }
            Err(e) => {
                eprintln!("Failed to load index ({}), rebuilding...", e);
                build_and_save_index(project_path, index_path)?
            }
        }
    } else {
        if !quiet {
            eprintln!("Building index for {:?}...", project_path);
        }
        build_and_save_index(project_path, index_path)?
    };

    // Merge siblings if any
    if !siblings.is_empty() {
        merge_and_cache_workspace(&mut index, siblings, workspace_idx, quiet);
    }

    Ok(index)
}

/// Check if the cached workspace index is newer than all sibling indexes and local index.
fn is_workspace_cache_fresh(
    workspace_idx: &std::path::Path,
    siblings: &[(PathBuf, String)],
    local_idx: &std::path::Path,
) -> bool {
    // Use manifest.json mtime (not directory mtime) for consistent comparison
    let cache_manifest = workspace_idx.join("manifest.json");
    let cache_mtime = match std::fs::metadata(&cache_manifest).and_then(|m| m.modified()) {
        Ok(t) => t,
        Err(_) => return false, // No cache
    };

    // Check local index is not newer than cache
    let local_manifest = local_idx.join("manifest.json");
    if let Ok(local_mtime) = std::fs::metadata(&local_manifest).and_then(|m| m.modified()) {
        if local_mtime > cache_mtime {
            return false; // Local index updated since cache
        }
    }

    // Cache is fresh if it's newer than every sibling's index
    siblings.iter().all(|(idx_path, _)| {
        // Check manifest.json mtime (always written on save)
        let manifest = idx_path.join("manifest.json");
        match std::fs::metadata(&manifest).and_then(|m| m.modified()) {
            Ok(sibling_mtime) => cache_mtime > sibling_mtime,
            Err(_) => true, // Sibling gone, cache still valid for others
        }
    })
}

/// Merge siblings into index and save the combined result as workspace cache.
fn merge_and_cache_workspace(
    index: &mut AgentContextIndex,
    siblings: &[(PathBuf, String)],
    workspace_idx: &std::path::Path,
    quiet: bool,
) {
    if !quiet {
        eprintln!("Merging {} sibling project(s):", siblings.len());
    }
    index.merge_siblings(siblings);

    // Cache the merged index for next time
    match index.save(workspace_idx) {
        Ok(()) => {
            if !quiet {
                eprintln!("Workspace index cached.");
            }
        }
        Err(e) => {
            if !quiet {
                eprintln!("Failed to cache workspace index: {}", e);
            }
        }
    }
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
            None,  // rank_by
            None,  // min_pagerank
            vec![], // include_project
            false, // churn
            false, // duplicates
            false, // entropy
            false, // faults
            None,  // definition_type
            false, // code
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
            None,  // rank_by
            None,  // min_pagerank
            vec![], // include_project
            false, // churn
            false, // duplicates
            false, // entropy
            false, // faults
            None,  // definition_type
            false, // code
        )
        .await;

        assert!(result.is_ok());
    }
}
