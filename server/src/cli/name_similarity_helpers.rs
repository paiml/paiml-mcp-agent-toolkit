//! Helper functions for name similarity analysis to reduce complexity

use std::path::PathBuf;
use anyhow::Result;
use serde_json::Value;

use super::{NameInfo, NameSimilarityResult, NameSimilarityOutputFormat, SearchScope};
use crate::services::file_discovery::{FileDiscoveryConfig, ProjectFileDiscovery};

/// Discover and filter source files based on configuration
pub fn discover_source_files(
    project_path: PathBuf,
    include: &Option<String>,
    exclude: &Option<String>,
) -> Result<Vec<(PathBuf, String)>> {
    let mut discovery_config = FileDiscoveryConfig::default();
    
    if let Some(exclude_pattern) = exclude {
        discovery_config.custom_ignore_patterns.push(exclude_pattern.clone());
    }
    
    let discovery = ProjectFileDiscovery::new(project_path).with_config(discovery_config);
    let discovered_files = discovery.discover_files()?;
    
    let mut analyzed_files = Vec::new();
    for file_path in discovered_files {
        if let Some(include_pattern) = include {
            if !file_path.to_string_lossy().contains(include_pattern) {
                continue;
            }
        }
        
        if let Ok(content) = std::fs::read_to_string(&file_path) {
            analyzed_files.push((file_path, content));
        }
    }
    
    Ok(analyzed_files)
}

/// Extract all identifiers from analyzed files
pub fn extract_all_identifiers(
    analyzed_files: &[(PathBuf, String)],
    scope: &SearchScope,
) -> Vec<NameInfo> {
    let mut all_names = Vec::new();
    for (file_path, content) in analyzed_files {
        let names = super::extract_identifiers(content, scope, file_path);
        all_names.extend(names);
    }
    all_names
}

/// Calculate similarity scores for all names
pub fn calculate_similarities(
    all_names: &[NameInfo],
    query: &str,
    threshold: f32,
    case_sensitive: bool,
    fuzzy: bool,
    phonetic: bool,
) -> Vec<NameSimilarityResult> {
    let mut similarities = Vec::new();
    let query_lower = if case_sensitive {
        query.to_string()
    } else {
        query.to_lowercase()
    };
    
    for name_info in all_names {
        let name_to_compare = if case_sensitive {
            name_info.name.clone()
        } else {
            name_info.name.to_lowercase()
        };
        
        let similarity_score = calculate_combined_similarity(
            &query_lower,
            &name_to_compare,
            fuzzy,
            phonetic,
        );
        
        if similarity_score >= threshold {
            similarities.push(NameSimilarityResult {
                name: name_info.name.clone(),
                similarity: similarity_score,
                file: name_info.file.clone(),
                line: name_info.line,
                name_type: name_info.name_type.clone(),
                context: name_info.context.clone(),
            });
        }
    }
    
    similarities.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());
    similarities
}

/// Calculate combined similarity score
fn calculate_combined_similarity(
    query: &str,
    name: &str,
    fuzzy: bool,
    phonetic: bool,
) -> f32 {
    let mut score = super::calculate_string_similarity(query, name);
    
    if fuzzy {
        let edit_distance = super::calculate_edit_distance(query, name);
        let max_len = query.len().max(name.len()) as f32;
        let fuzzy_score = if max_len > 0.0 {
            1.0 - (edit_distance as f32 / max_len)
        } else {
            1.0
        };
        score = score.max(fuzzy_score);
    }
    
    if phonetic {
        let query_soundex = super::calculate_soundex(query);
        let name_soundex = super::calculate_soundex(name);
        if query_soundex == name_soundex {
            score = score.max(0.8);
        }
    }
    
    score
}

/// Build results JSON with optional performance metrics
pub fn build_results_json(
    query: &str,
    all_names_len: usize,
    similarities: &[NameSimilarityResult],
    scope: &SearchScope,
    threshold: f32,
    phonetic: bool,
    fuzzy: bool,
    case_sensitive: bool,
    perf: bool,
    analysis_time: std::time::Duration,
    analyzed_files_len: usize,
) -> Value {
    let mut results = serde_json::json!({
        "query": query,
        "total_identifiers": all_names_len,
        "matches": similarities.len(),
        "results": similarities.iter().map(|s| serde_json::json!({
            "name": s.name,
            "similarity": s.similarity,
            "file": s.file.to_string_lossy(),
            "line": s.line,
            "type": s.name_type,
            "context": s.context
        })).collect::<Vec<_>>(),
        "parameters": {
            "scope": format!("{scope:?}"),
            "threshold": threshold,
            "phonetic": phonetic,
            "fuzzy": fuzzy,
            "case_sensitive": case_sensitive
        }
    });
    
    if perf {
        results["performance"] = serde_json::json!({
            "analysis_time_s": analysis_time.as_secs_f64(),
            "identifiers_per_second": all_names_len as f64 / analysis_time.as_secs_f64(),
            "files_analyzed": analyzed_files_len
        });
    }
    
    results
}

/// Format and output results based on selected format
pub fn output_results(
    format: NameSimilarityOutputFormat,
    query: &str,
    all_names_len: usize,
    similarities: &[NameSimilarityResult],
    final_results: &Value,
    perf: bool,
    analysis_time: std::time::Duration,
    analyzed_files_len: usize,
    output: Option<PathBuf>,
) -> Result<()> {
    let output_content = match format {
        NameSimilarityOutputFormat::Summary => {
            format_summary_output(query, all_names_len, similarities, perf, analysis_time, analyzed_files_len)
        }
        NameSimilarityOutputFormat::Detailed => {
            format_detailed_output(similarities)
        }
        NameSimilarityOutputFormat::Json => {
            serde_json::to_string_pretty(final_results)?
        }
        NameSimilarityOutputFormat::Csv => {
            format_csv_output(similarities)
        }
        NameSimilarityOutputFormat::Markdown => {
            format_markdown_output(query, all_names_len, similarities)
        }
    };
    
    if let Some(output_path) = output {
        std::fs::write(output_path, output_content)?;
    } else {
        println!("{}", output_content);
    }
    
    Ok(())
}

fn format_summary_output(
    query: &str,
    all_names_len: usize,
    similarities: &[NameSimilarityResult],
    perf: bool,
    analysis_time: std::time::Duration,
    analyzed_files_len: usize,
) -> String {
    let mut output = String::new();
    output.push_str("Name Similarity Analysis\n");
    output.push_str("======================\n");
    output.push_str(&format!("Query: '{query}'\n"));
    output.push_str(&format!("Total identifiers: {}\n", all_names_len));
    output.push_str(&format!("Matches found: {}\n", similarities.len()));
    
    if !similarities.is_empty() {
        output.push_str("\nTop matches:\n");
        for (i, sim) in similarities.iter().take(10).enumerate() {
            output.push_str(&format!(
                "{}. {} (similarity: {:.3}) in {}:{}\n",
                i + 1,
                sim.name,
                sim.similarity,
                sim.file.file_name().unwrap().to_string_lossy(),
                sim.line
            ));
        }
    }
    
    if perf {
        output.push_str("\nPerformance:\n");
        output.push_str(&format!("  Analysis time: {:.2}s\n", analysis_time.as_secs_f64()));
        output.push_str(&format!("  Files analyzed: {}\n", analyzed_files_len));
    }
    
    output
}

fn format_detailed_output(similarities: &[NameSimilarityResult]) -> String {
    let mut output = String::new();
    output.push_str("Name Similarity Analysis Report\n");
    output.push_str("==============================\n");
    
    for sim in similarities {
        output.push_str(&format!("\nMatch: {}\n", sim.name));
        output.push_str(&format!("  Similarity: {:.3}\n", sim.similarity));
        output.push_str(&format!("  Type: {}\n", sim.name_type));
        output.push_str(&format!("  File: {}\n", sim.file.to_string_lossy()));
        output.push_str(&format!("  Line: {}\n", sim.line));
        if !sim.context.is_empty() {
            output.push_str(&format!("  Context: {}\n", sim.context));
        }
    }
    
    output
}

fn format_csv_output(similarities: &[NameSimilarityResult]) -> String {
    let mut output = String::new();
    output.push_str("name,similarity,type,file,line,context\n");
    for sim in similarities {
        output.push_str(&format!(
            "{},{:.3},{},{},{},\"{}\"\n",
            sim.name,
            sim.similarity,
            sim.name_type,
            sim.file.to_string_lossy(),
            sim.line,
            sim.context.replace('"', "\"\"")
        ));
    }
    output
}

fn format_markdown_output(
    query: &str,
    all_names_len: usize,
    similarities: &[NameSimilarityResult],
) -> String {
    let mut output = String::new();
    output.push_str("# Name Similarity Analysis\n\n");
    output.push_str(&format!("**Query**: `{query}`\n\n"));
    output.push_str(&format!("**Total identifiers**: {}\n\n", all_names_len));
    output.push_str(&format!("**Matches found**: {}\n\n", similarities.len()));
    
    if !similarities.is_empty() {
        output.push_str("## Results\n\n");
        output.push_str("| Rank | Name | Similarity | Type | File | Line |\n");
        output.push_str("| ---- | ---- | ---------- | ---- | ---- | ---- |\n");
        for (i, sim) in similarities.iter().enumerate() {
            output.push_str(&format!(
                "| {} | `{}` | {:.3} | {} | {} | {} |\n",
                i + 1,
                sim.name,
                sim.similarity,
                sim.name_type,
                sim.file.file_name().unwrap().to_string_lossy(),
                sim.line
            ));
        }
    }
    
    output
}