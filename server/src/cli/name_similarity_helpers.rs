//! Helper functions for name similarity analysis to reduce complexity

use anyhow::Result;
use serde_json::Value;
use std::path::PathBuf;

use super::{NameInfo, NameSimilarityOutputFormat, NameSimilarityResult, SearchScope};
use crate::services::file_discovery::{FileDiscoveryConfig, ProjectFileDiscovery};

/// Configuration for JSON results building
pub struct JsonResultsConfig<'a> {
    pub query: &'a str,
    pub all_names_len: usize,
    pub similarities: &'a [NameSimilarityResult],
    pub scope: &'a SearchScope,
    pub threshold: f32,
    pub phonetic: bool,
    pub fuzzy: bool,
    pub case_sensitive: bool,
    pub perf: bool,
    pub analysis_time: std::time::Duration,
    pub analyzed_files_len: usize,
}

/// Configuration for output formatting
pub struct OutputConfig<'a> {
    pub format: NameSimilarityOutputFormat,
    pub query: &'a str,
    pub all_names_len: usize,
    pub similarities: &'a [NameSimilarityResult],
    pub final_results: &'a Value,
    pub perf: bool,
    pub analysis_time: std::time::Duration,
    pub analyzed_files_len: usize,
    pub output: Option<PathBuf>,
}

/// Discover and filter source files based on configuration
pub fn discover_source_files(
    project_path: PathBuf,
    include: &Option<String>,
    exclude: &Option<String>,
) -> Result<Vec<(PathBuf, String)>> {
    let mut discovery_config = FileDiscoveryConfig::default();

    if let Some(exclude_pattern) = exclude {
        discovery_config
            .custom_ignore_patterns
            .push(exclude_pattern.clone());
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
    _scope: &SearchScope,
) -> Vec<NameInfo> {
    let mut all_names = Vec::new();
    for (_file_path, content) in analyzed_files {
        let names = super::stubs::extract_identifiers(content);
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

        let similarity_score =
            calculate_combined_similarity(&query_lower, &name_to_compare, fuzzy, phonetic);

        if similarity_score >= threshold {
            similarities.push(NameSimilarityResult {
                name: name_info.name.clone(),
                kind: name_info.kind.clone(),
                file_path: name_info.file_path.clone(),
                line: name_info.line,
                similarity: similarity_score,
                phonetic_match: false,
                fuzzy_match: fuzzy,
            });
        }
    }

    similarities.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());
    similarities
}

/// Calculate combined similarity score
fn calculate_combined_similarity(query: &str, name: &str, fuzzy: bool, phonetic: bool) -> f32 {
    let mut score = super::stubs::calculate_string_similarity(query, name);

    if fuzzy {
        let edit_distance = super::stubs::calculate_edit_distance(query, name);
        let max_len = query.len().max(name.len()) as f32;
        let fuzzy_score = if max_len > 0.0 {
            1.0 - (edit_distance as f32 / max_len)
        } else {
            1.0
        };
        score = score.max(fuzzy_score);
    }

    if phonetic {
        let query_soundex = super::stubs::calculate_soundex(query);
        let name_soundex = super::stubs::calculate_soundex(name);
        if query_soundex == name_soundex {
            score = score.max(0.8);
        }
    }

    score
}

/// Build results JSON with optional performance metrics
pub fn build_results_json(config: JsonResultsConfig) -> Value {
    let mut results = serde_json::json!({
        "query": config.query,
        "total_identifiers": config.all_names_len,
        "matches": config.similarities.len(),
        "results": config.similarities.iter().map(|s| serde_json::json!({
            "name": s.name,
            "similarity": s.similarity,
            "file": s.file_path.to_string_lossy(),
            "line": s.line,
            "type": s.kind,
            "phonetic_match": s.phonetic_match,
            "fuzzy_match": s.fuzzy_match
        })).collect::<Vec<_>>(),
        "parameters": {
            "scope": format!("{:?}", config.scope),
            "threshold": config.threshold,
            "phonetic": config.phonetic,
            "fuzzy": config.fuzzy,
            "case_sensitive": config.case_sensitive
        }
    });

    if config.perf {
        results["performance"] = serde_json::json!({
            "analysis_time_s": config.analysis_time.as_secs_f64(),
            "identifiers_per_second": config.all_names_len as f64 / config.analysis_time.as_secs_f64(),
            "files_analyzed": config.analyzed_files_len
        });
    }

    results
}

/// Format and output results based on selected format
pub fn output_results(config: OutputConfig) -> Result<()> {
    let output_content = match config.format {
        NameSimilarityOutputFormat::Summary => format_summary_output(
            config.query,
            config.all_names_len,
            config.similarities,
            config.perf,
            config.analysis_time,
            config.analyzed_files_len,
        ),
        NameSimilarityOutputFormat::Detailed => format_detailed_output(config.similarities),
        NameSimilarityOutputFormat::Human => format_summary_output(
            config.query,
            config.all_names_len,
            config.similarities,
            config.perf,
            config.analysis_time,
            config.analyzed_files_len,
        ),
        NameSimilarityOutputFormat::Json => serde_json::to_string_pretty(config.final_results)?,
        NameSimilarityOutputFormat::Csv => format_csv_output(config.similarities),
        NameSimilarityOutputFormat::Markdown => {
            format_markdown_output(config.query, config.all_names_len, config.similarities)
        }
    };

    if let Some(output_path) = config.output {
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
                sim.file_path.file_name().unwrap().to_string_lossy(),
                sim.line
            ));
        }
    }

    if perf {
        output.push_str("\nPerformance:\n");
        output.push_str(&format!(
            "  Analysis time: {:.2}s\n",
            analysis_time.as_secs_f64()
        ));
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
        output.push_str(&format!("  Type: {}\n", sim.kind));
        output.push_str(&format!("  File: {}\n", sim.file_path.to_string_lossy()));
        output.push_str(&format!("  Line: {}\n", sim.line));
        output.push_str(&format!("  Phonetic Match: {}\n", sim.phonetic_match));
        output.push_str(&format!("  Fuzzy Match: {}\n", sim.fuzzy_match));
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
            sim.kind,
            sim.file_path.to_string_lossy(),
            sim.line,
            if sim.phonetic_match { "true" } else { "false" }
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
                sim.kind,
                sim.file_path.file_name().unwrap().to_string_lossy(),
                sim.line
            ));
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::time::Duration;
    use tempfile::TempDir;

    fn create_test_name_info(name: &str, line: u32) -> NameInfo {
        NameInfo {
            name: name.to_string(),
            kind: "function".to_string(),
            file_path: PathBuf::from("test.rs"),
            line: line as usize,
        }
    }

    fn create_test_similarity_result(name: &str, similarity: f32) -> NameSimilarityResult {
        NameSimilarityResult {
            name: name.to_string(),
            kind: "function".to_string(),
            file_path: PathBuf::from("test.rs"),
            line: 1,
            similarity,
            phonetic_match: false,
            fuzzy_match: false,
        }
    }

    #[test]
    fn test_discover_source_files_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let result = discover_source_files(temp_dir.path().to_path_buf(), &None, &None);

        assert!(result.is_ok());
        let files = result.unwrap();
        assert!(files.is_empty());
    }

    #[test]
    fn test_discover_source_files_with_files() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        std::fs::write(&test_file, "fn main() {}").unwrap();

        let result = discover_source_files(temp_dir.path().to_path_buf(), &None, &None);

        assert!(result.is_ok());
        let files = result.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].1, "fn main() {}");
    }

    #[test]
    fn test_discover_source_files_with_include_filter() {
        let temp_dir = TempDir::new().unwrap();
        let rust_file = temp_dir.path().join("test.rs");
        let js_file = temp_dir.path().join("test.js");
        std::fs::write(&rust_file, "fn main() {}").unwrap();
        std::fs::write(&js_file, "function main() {}").unwrap();

        let result = discover_source_files(
            temp_dir.path().to_path_buf(),
            &Some(".rs".to_string()),
            &None,
        );

        assert!(result.is_ok());
        let files = result.unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0].0.to_string_lossy().contains(".rs"));
    }

    #[test]
    fn test_extract_all_identifiers() {
        let analyzed_files = vec![
            (PathBuf::from("test1.rs"), "fn foo() {}".to_string()),
            (PathBuf::from("test2.rs"), "fn bar() {}".to_string()),
        ];
        let scope = SearchScope::Functions;

        let result = extract_all_identifiers(&analyzed_files, &scope);

        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|n| n.name == "foo"));
        assert!(result.iter().any(|n| n.name == "bar"));
    }

    #[test]
    fn test_extract_all_identifiers_empty() {
        let analyzed_files = vec![];
        let scope = SearchScope::Functions;

        let result = extract_all_identifiers(&analyzed_files, &scope);

        assert!(result.is_empty());
    }

    #[test]
    fn test_calculate_similarities_exact_match() {
        let all_names = vec![
            create_test_name_info("test_function", 1),
            create_test_name_info("other_function", 2),
        ];

        let result = calculate_similarities(&all_names, "test_function", 0.9, true, false, false);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "test_function");
        assert!(result[0].similarity > 0.9);
    }

    #[test]
    fn test_calculate_similarities_threshold_filter() {
        let all_names = vec![
            create_test_name_info("test_function", 1),
            create_test_name_info("completely_different", 2),
        ];

        let result = calculate_similarities(&all_names, "test_function", 0.9, true, false, false);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "test_function");
    }

    #[test]
    fn test_calculate_similarities_case_insensitive() {
        let all_names = vec![create_test_name_info("TEST_FUNCTION", 1)];

        let result = calculate_similarities(&all_names, "test_function", 0.5, false, false, false);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "TEST_FUNCTION");
        assert!(result[0].similarity > 0.9);
    }

    #[test]
    fn test_calculate_similarities_sorted_by_score() {
        let all_names = vec![
            create_test_name_info("test", 1),
            create_test_name_info("test_function", 2),
            create_test_name_info("testing", 3),
        ];

        let result = calculate_similarities(&all_names, "test", 0.3, true, false, false);

        assert!(result.len() >= 2);
        // Results should be sorted by similarity score (descending)
        for i in 1..result.len() {
            assert!(result[i - 1].similarity >= result[i].similarity);
        }
    }

    #[test]
    fn test_calculate_combined_similarity_basic() {
        let score = calculate_combined_similarity("test", "test", false, false);
        assert!(score > 0.9);

        let score = calculate_combined_similarity("test", "completely_different", false, false);
        assert!(score < 0.5);
    }

    #[test]
    fn test_calculate_combined_similarity_with_fuzzy() {
        let score = calculate_combined_similarity("test", "tset", true, false);
        assert!(score >= 0.5); // Should account for edit distance (exactly 0.5 for 2 swaps in 4 chars)
    }

    #[test]
    fn test_build_results_json_basic() {
        let similarities = vec![create_test_similarity_result("test_func", 0.95)];
        let config = JsonResultsConfig {
            query: "test",
            all_names_len: 10,
            similarities: &similarities,
            scope: &SearchScope::Functions,
            threshold: 0.8,
            phonetic: false,
            fuzzy: false,
            case_sensitive: true,
            perf: false,
            analysis_time: Duration::from_millis(100),
            analyzed_files_len: 5,
        };

        let result = build_results_json(config);

        assert_eq!(result["query"], "test");
        assert_eq!(result["total_identifiers"], 10);
        assert_eq!(result["matches"], 1);
        assert!(result["results"].is_array());
        assert_eq!(result["results"][0]["name"], "test_func");
    }

    #[test]
    fn test_build_results_json_with_performance() {
        let similarities = vec![create_test_similarity_result("test_func", 0.95)];
        let config = JsonResultsConfig {
            query: "test",
            all_names_len: 100,
            similarities: &similarities,
            scope: &SearchScope::Functions,
            threshold: 0.8,
            phonetic: false,
            fuzzy: false,
            case_sensitive: true,
            perf: true,
            analysis_time: Duration::from_secs(2),
            analyzed_files_len: 20,
        };

        let result = build_results_json(config);

        assert!(result["performance"].is_object());
        assert_eq!(result["performance"]["analysis_time_s"], 2.0);
        assert_eq!(result["performance"]["identifiers_per_second"], 50.0);
        assert_eq!(result["performance"]["files_analyzed"], 20);
    }

    #[test]
    fn test_output_results_json_format() {
        let similarities = vec![create_test_similarity_result("test_func", 0.95)];
        let results = json!({"test": "data"});
        let config = OutputConfig {
            format: NameSimilarityOutputFormat::Json,
            query: "test",
            all_names_len: 10,
            similarities: &similarities,
            final_results: &results,
            perf: false,
            analysis_time: Duration::from_millis(100),
            analyzed_files_len: 5,
            output: None,
        };

        let result = output_results(config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_output_results_summary_format() {
        let similarities = vec![create_test_similarity_result("test_func", 0.95)];
        let results = json!({"test": "data"});
        let config = OutputConfig {
            format: NameSimilarityOutputFormat::Summary,
            query: "test",
            all_names_len: 10,
            similarities: &similarities,
            final_results: &results,
            perf: true,
            analysis_time: Duration::from_millis(100),
            analyzed_files_len: 5,
            output: None,
        };

        let result = output_results(config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_format_summary_output() {
        let similarities = vec![
            create_test_similarity_result("test_func", 0.95),
            create_test_similarity_result("test_var", 0.85),
        ];

        let output =
            format_summary_output("test", 100, &similarities, true, Duration::from_secs(1), 10);

        assert!(output.contains("Name Similarity Analysis"));
        assert!(output.contains("Query: 'test'"));
        assert!(output.contains("Total identifiers: 100"));
        assert!(output.contains("Matches found: 2"));
        assert!(output.contains("test_func"));
        assert!(output.contains("Performance:"));
        assert!(output.contains("Analysis time: 1.00s"));
        assert!(output.contains("Files analyzed: 10"));
    }

    #[test]
    fn test_format_detailed_output() {
        let similarities = vec![create_test_similarity_result("test_func", 0.95)];

        let output = format_detailed_output(&similarities);

        assert!(output.contains("Name Similarity Analysis Report"));
        assert!(output.contains("Match: test_func"));
        assert!(output.contains("Similarity: 0.950"));
        assert!(output.contains("Type: function"));
        assert!(output.contains("File: test.rs"));
        assert!(output.contains("Line: 1"));
    }

    #[test]
    fn test_format_csv_output() {
        let similarities = vec![
            create_test_similarity_result("test_func", 0.95),
            create_test_similarity_result("test_var", 0.85),
        ];

        let output = format_csv_output(&similarities);

        assert!(output.contains("name,similarity,type,file,line,context"));
        assert!(output.contains("test_func,0.950"));
        assert!(output.contains("test_var,0.850"));
    }

    #[test]
    fn test_format_markdown_output() {
        let similarities = vec![create_test_similarity_result("test_func", 0.95)];

        let output = format_markdown_output("test", 100, &similarities);

        assert!(output.contains("# Name Similarity Analysis"));
        assert!(output.contains("**Query**: `test`"));
        assert!(output.contains("**Total identifiers**: 100"));
        assert!(output.contains("**Matches found**: 1"));
        assert!(output.contains("| Rank | Name | Similarity"));
        assert!(output.contains("| 1 | `test_func` | 0.950"));
    }

    #[test]
    fn test_format_markdown_output_empty() {
        let similarities = vec![];

        let output = format_markdown_output("test", 100, &similarities);

        assert!(output.contains("# Name Similarity Analysis"));
        assert!(output.contains("**Matches found**: 0"));
        assert!(!output.contains("## Results"));
    }
}
