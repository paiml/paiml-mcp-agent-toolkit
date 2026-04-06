//! Name similarity analysis - finds similar names in code

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NameMatch {
    pub name: String,
    pub file: String,
    pub line: usize,
    pub kind: String,
    pub similarity_score: f32,
    pub edit_distance: usize,
    pub phonetic_match: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct NameSimilarityResult {
    pub query: String,
    pub matches: Vec<NameMatch>,
    pub total_candidates: usize,
    pub search_scope: String,
}

#[allow(clippy::too_many_arguments)]
pub async fn handle_analyze_name_similarity(
    project_path: PathBuf,
    query: String,
    top_k: usize,
    phonetic: bool,
    scope: crate::cli::SearchScope,
    threshold: f32,
    format: crate::cli::NameSimilarityOutputFormat,
    include: Option<String>,
    exclude: Option<String>,
    output: Option<PathBuf>,
    _perf: bool,
    fuzzy: bool,
    case_sensitive: bool,
) -> Result<()> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    eprintln!("🔍 Searching for names similar to '{query}'...");

    // Collect all names from the project
    let names = collect_names(&project_path, &include, &exclude, scope).await?;
    eprintln!("✅ Found {} names to analyze", names.len());

    // Find similar names
    let matches = find_similar_names(&query, names, threshold, phonetic, fuzzy, case_sensitive)?;

    // Take top K matches
    let mut top_matches = matches;
    top_matches.sort_by(|a, b| {
        b.similarity_score
            .partial_cmp(&a.similarity_score)
            .expect("internal error")
    });
    top_matches.truncate(top_k);

    let result = NameSimilarityResult {
        query: query.clone(),
        total_candidates: top_matches.len(),
        matches: top_matches,
        search_scope: format!("{scope:?}"),
    };

    // Format output
    let content = format_output(result, format)?;

    // Write output
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &content).await?;
        eprintln!("✅ Results written to: {}", output_path.display());
    } else {
        println!("{content}");
    }

    Ok(())
}

// --- Submodule includes ---
include!("name_similarity_file_collection.rs");
include!("name_similarity_scoring.rs");
include!("name_similarity_formatting.rs");
include!("name_similarity_tests.rs");
