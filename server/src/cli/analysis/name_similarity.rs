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
    eprintln!("🔍 Searching for names similar to '{}'...", query);

    // Collect all names from the project
    let names = collect_names(&project_path, &include, &exclude, scope).await?;
    eprintln!("✅ Found {} names to analyze", names.len());

    // Find similar names
    let matches = find_similar_names(&query, names, threshold, phonetic, fuzzy, case_sensitive)?;

    // Take top K matches
    let mut top_matches = matches;
    top_matches.sort_by(|a, b| b.similarity_score.partial_cmp(&a.similarity_score).unwrap());
    top_matches.truncate(top_k);

    let result = NameSimilarityResult {
        query: query.clone(),
        total_candidates: top_matches.len(),
        matches: top_matches,
        search_scope: format!("{:?}", scope),
    };

    // Format output
    let content = format_output(result, format)?;

    // Write output
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &content).await?;
        eprintln!("✅ Results written to: {}", output_path.display());
    } else {
        println!("{}", content);
    }

    Ok(())
}

// Collect names from project based on scope
async fn collect_names(
    project_path: &Path,
    include: &Option<String>,
    exclude: &Option<String>,
    scope: crate::cli::SearchScope,
) -> Result<Vec<(String, String, usize, String)>> {
    let mut names = Vec::new();
    let files = collect_source_files(project_path, include, exclude).await?;

    for file in files {
        let content = tokio::fs::read_to_string(&file).await?;
        let file_str = file.to_string_lossy().to_string();

        // Extract names based on scope
        let file_names = extract_names(&content, &file_str, scope)?;
        names.extend(file_names);
    }

    Ok(names)
}

// Collect source files
async fn collect_source_files(
    project_path: &Path,
    include: &Option<String>,
    exclude: &Option<String>,
) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    collect_files_recursive(project_path, &mut files, include, exclude).await?;

    Ok(files)
}

// Recursively collect files
async fn collect_files_recursive(
    dir: &Path,
    files: &mut Vec<PathBuf>,
    include: &Option<String>,
    exclude: &Option<String>,
) -> Result<()> {
    let mut entries = tokio::fs::read_dir(dir).await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        let path_str = path.to_string_lossy();

        if let Some(excl) = exclude {
            if path_str.contains(excl) {
                continue;
            }
        }

        if path.is_dir() {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if !name.starts_with('.') && name != "node_modules" && name != "target" {
                Box::pin(collect_files_recursive(&path, files, include, exclude)).await?;
            }
        } else if is_code_file(&path) {
            if let Some(incl) = include {
                if path_str.contains(incl) {
                    files.push(path);
                }
            } else {
                files.push(path);
            }
        }
    }

    Ok(())
}

// Check if file is a code file
fn is_code_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|s| s.to_str()),
        Some("rs")
            | Some("js")
            | Some("ts")
            | Some("py")
            | Some("java")
            | Some("cpp")
            | Some("c")
            | Some("go")
    )
}

// Extract names from content based on scope
fn extract_names(
    content: &str,
    file: &str,
    scope: crate::cli::SearchScope,
) -> Result<Vec<(String, String, usize, String)>> {
    use regex::Regex;

    let mut names = Vec::new();

    let patterns = match scope {
        crate::cli::SearchScope::Functions => vec![
            (Regex::new(r"(?m)^(?:\w+\s+)*fn\s+(\w+)")?, "function"),
            (Regex::new(r"(?m)^(?:\w+\s+)*function\s+(\w+)")?, "function"),
            (Regex::new(r"(?m)^def\s+(\w+)")?, "function"),
        ],
        crate::cli::SearchScope::Types => vec![
            (Regex::new(r"(?m)^(?:\w+\s+)*struct\s+(\w+)")?, "struct"),
            (Regex::new(r"(?m)^(?:\w+\s+)*class\s+(\w+)")?, "class"),
            (Regex::new(r"(?m)^(?:\w+\s+)*enum\s+(\w+)")?, "enum"),
            (
                Regex::new(r"(?m)^(?:\w+\s+)*interface\s+(\w+)")?,
                "interface",
            ),
        ],
        crate::cli::SearchScope::Variables => vec![
            (
                Regex::new(r"(?m)^(?:\w+\s+)*let\s+(?:mut\s+)?(\w+)")?,
                "variable",
            ),
            (Regex::new(r"(?m)^(?:\w+\s+)*const\s+(\w+)")?, "constant"),
            (Regex::new(r"(?m)^(?:\w+\s+)*var\s+(\w+)")?, "variable"),
        ],
        crate::cli::SearchScope::All => vec![
            (Regex::new(r"(?m)^(?:\w+\s+)*fn\s+(\w+)")?, "function"),
            (Regex::new(r"(?m)^(?:\w+\s+)*struct\s+(\w+)")?, "struct"),
            (
                Regex::new(r"(?m)^(?:\w+\s+)*let\s+(?:mut\s+)?(\w+)")?,
                "variable",
            ),
            (Regex::new(r"(?m)^(?:\w+\s+)*const\s+(\w+)")?, "constant"),
        ],
    };

    for (line_no, line) in content.lines().enumerate() {
        for (pattern, kind) in &patterns {
            if let Some(captures) = pattern.captures(line) {
                if let Some(name_match) = captures.get(1) {
                    names.push((
                        name_match.as_str().to_string(),
                        file.to_string(),
                        line_no + 1,
                        kind.to_string(),
                    ));
                }
            }
        }
    }

    Ok(names)
}

// Find similar names
fn find_similar_names(
    query: &str,
    candidates: Vec<(String, String, usize, String)>,
    threshold: f32,
    phonetic: bool,
    fuzzy: bool,
    case_sensitive: bool,
) -> Result<Vec<NameMatch>> {
    use crate::cli::analysis_utilities::{calculate_edit_distance, calculate_soundex};

    let mut matches = Vec::new();
    let query_lower = if case_sensitive {
        query.to_string()
    } else {
        query.to_lowercase()
    };
    let query_soundex = if phonetic {
        calculate_soundex(query)
    } else {
        String::new()
    };

    for (name, file, line, kind) in candidates {
        let name_compare = if case_sensitive {
            name.clone()
        } else {
            name.to_lowercase()
        };

        // Calculate similarity
        let edit_distance = calculate_edit_distance(&query_lower, &name_compare);
        let max_len = query.len().max(name.len());
        let similarity = if max_len > 0 {
            1.0 - (edit_distance as f32 / max_len as f32)
        } else {
            0.0
        };

        // Check phonetic match
        let phonetic_match = if phonetic {
            calculate_soundex(&name) == query_soundex
        } else {
            false
        };

        // Apply fuzzy matching boost
        let final_score = if fuzzy && name_compare.contains(&query_lower) {
            (similarity + 0.3).min(1.0)
        } else {
            similarity
        };

        // Check threshold
        if final_score >= threshold || phonetic_match {
            matches.push(NameMatch {
                name,
                file,
                line,
                kind,
                similarity_score: final_score,
                edit_distance,
                phonetic_match,
            });
        }
    }

    Ok(matches)
}

// Format output
pub fn format_output(
    result: NameSimilarityResult,
    format: crate::cli::NameSimilarityOutputFormat,
) -> Result<String> {
    match format {
        crate::cli::NameSimilarityOutputFormat::Json => format_json_output(&result),
        crate::cli::NameSimilarityOutputFormat::Human
        | crate::cli::NameSimilarityOutputFormat::Summary
        | crate::cli::NameSimilarityOutputFormat::Detailed => format_human_output(&result),
        crate::cli::NameSimilarityOutputFormat::Csv => format_csv_output(&result),
        crate::cli::NameSimilarityOutputFormat::Markdown => format_markdown_output(&result),
    }
}

/// Format output as JSON (cognitive complexity ≤2)
fn format_json_output(result: &NameSimilarityResult) -> Result<String> {
    Ok(serde_json::to_string_pretty(result)?)
}

/// Format output for human reading (cognitive complexity ≤8)
fn format_human_output(result: &NameSimilarityResult) -> Result<String> {
    use std::fmt::Write;
    
    let mut output = String::new();
    writeln!(&mut output, "# Name Similarity Analysis\n")?;
    writeln!(&mut output, "Query: '{}'\n", result.query)?;
    writeln!(&mut output, "Found {} matches:\n", result.matches.len())?;

    for (i, m) in result.matches.iter().enumerate() {
        format_human_match_entry(&mut output, i, m)?;
    }

    Ok(output)
}

/// Format a single match entry for human output (cognitive complexity ≤6)
fn format_human_match_entry(
    output: &mut String,
    index: usize,
    m: &NameMatch,
) -> Result<()> {
    use std::fmt::Write;
    
    writeln!(
        output,
        "{}. {} (score: {:.2})",
        index + 1,
        m.name,
        m.similarity_score
    )?;
    writeln!(output, "   File: {}:{}", m.file, m.line)?;
    writeln!(output, "   Type: {}", m.kind)?;
    writeln!(output, "   Edit distance: {}", m.edit_distance)?;
    if m.phonetic_match {
        writeln!(output, "   ✓ Phonetic match")?;
    }
    writeln!(output)?;
    
    Ok(())
}

/// Format output as CSV (cognitive complexity ≤5)
fn format_csv_output(result: &NameSimilarityResult) -> Result<String> {
    use std::fmt::Write;
    
    let mut output = String::new();
    writeln!(
        &mut output,
        "name,file,line,kind,similarity_score,edit_distance,phonetic_match"
    )?;
    
    for m in &result.matches {
        format_csv_match_entry(&mut output, m)?;
    }
    
    Ok(output)
}

/// Format a single match entry for CSV output (cognitive complexity ≤2)
fn format_csv_match_entry(output: &mut String, m: &NameMatch) -> Result<()> {
    use std::fmt::Write;
    
    writeln!(
        output,
        "{},{},{},{},{:.3},{},{}",
        m.name,
        m.file,
        m.line,
        m.kind,
        m.similarity_score,
        m.edit_distance,
        m.phonetic_match
    )?;
    
    Ok(())
}

/// Format output as Markdown (cognitive complexity ≤7)
fn format_markdown_output(result: &NameSimilarityResult) -> Result<String> {
    use std::fmt::Write;
    
    let mut output = String::new();
    writeln!(&mut output, "# Name Similarity Report\n")?;
    writeln!(&mut output, "**Query:** `{}`\n", result.query)?;
    writeln!(&mut output, "**Total matches:** {}\n", result.matches.len())?;
    
    format_markdown_table_header(&mut output)?;
    
    for m in result.matches.iter().take(20) {
        format_markdown_match_entry(&mut output, m)?;
    }
    
    Ok(output)
}

/// Format Markdown table header (cognitive complexity ≤2)
fn format_markdown_table_header(output: &mut String) -> Result<()> {
    use std::fmt::Write;
    
    writeln!(output, "## Matches\n")?;
    writeln!(
        output,
        "| Name | File | Line | Type | Score | Edit Distance | Phonetic |"
    )?;
    writeln!(
        output,
        "|------|------|------|------|-------|---------------|----------|"
    )?;
    
    Ok(())
}

/// Format a single match entry for Markdown output (cognitive complexity ≤3)
fn format_markdown_match_entry(output: &mut String, m: &NameMatch) -> Result<()> {
    use std::fmt::Write;
    
    writeln!(
        output,
        "| {} | {} | {} | {} | {:.2} | {} | {} |",
        m.name,
        m.file,
        m.line,
        m.kind,
        m.similarity_score,
        m.edit_distance,
        if m.phonetic_match { "✓" } else { "✗" }
    )?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_code_file() {
        assert!(is_code_file(Path::new("test.rs")));
        assert!(is_code_file(Path::new("test.js")));
        assert!(!is_code_file(Path::new("test.txt")));
    }

    #[test]
    fn test_extract_names() {
        let content = "fn test_function() {}\nstruct TestStruct {}";
        let names = extract_names(content, "test.rs", crate::cli::SearchScope::All).unwrap();
        assert_eq!(names.len(), 2);
        assert_eq!(names[0].0, "test_function");
        assert_eq!(names[1].0, "TestStruct");
    }

    #[test]
    fn test_find_similar_names() {
        let candidates = vec![
            (
                "test_function".to_string(),
                "test.rs".to_string(),
                1,
                "function".to_string(),
            ),
            (
                "test_func".to_string(),
                "test.rs".to_string(),
                2,
                "function".to_string(),
            ),
            (
                "unrelated".to_string(),
                "test.rs".to_string(),
                3,
                "function".to_string(),
            ),
        ];

        let matches = find_similar_names("test_fun", candidates, 0.5, false, false, false).unwrap();

        assert!(matches.len() >= 2);
        assert!(matches.iter().any(|m| m.name == "test_function"));
        assert!(matches.iter().any(|m| m.name == "test_func"));
    }
}
