//! Name similarity analysis - finds similar names in code

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Name match.
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
/// Result of name similarity operation.
pub struct NameSimilarityResult {
    pub query: String,
    pub matches: Vec<NameMatch>,
    /// How many names the query was compared against — the denominator, not
    /// the number of matches.
    pub total_candidates: usize,
    pub search_scope: String,
}

#[allow(clippy::too_many_arguments)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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
    crate::status_eprintln!("🔍 Searching for names similar to '{query}'...");

    // Collect all names from the project
    let names = collect_names(&project_path, &include, &exclude, scope).await?;
    crate::status_eprintln!("✅ Found {} names to analyze", names.len());

    // #1015. An empty directory yielded zero candidate names and this printed
    //
    //     Name Similarity Analysis
    //       Query: zzz
    //       Found: 0 matches
    //
    // with exit 0 — BYTE-IDENTICAL to the answer a 4,000-name codebase gives
    // for a query that genuinely matches nothing, because the report shows the
    // numerator and never the denominator. "0 of 0" and "0 of 4000" are
    // different facts; the first one is not a search result.
    crate::cli::ensure_files_were_analyzed("names", "name-similarity", &project_path, names.len())?;
    let total_candidates = names.len();

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
        // The names the query was compared AGAINST. This was
        // `top_matches.len()` — the matches, i.e. the same number as
        // `matches.len()` — so the one field that could tell a reader
        // "0 matches out of 4,000 candidates" from "0 matches out of nothing"
        // carried the numerator twice and the denominator never.
        total_candidates,
        matches: top_matches,
        search_scope: format!("{scope:?}"),
    };

    // Format output
    let content = format_output(result, format)?;

    // Write output
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &content).await?;
        crate::status_eprintln!("✅ Results written to: {}", output_path.display());
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
