/// Build results JSON with optional performance metrics
#[must_use]
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
        println!("{output_content}");
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
    debug_assert!(!query.is_empty(), "query must not be empty");
    let mut output = String::new();
    output.push_str("Name Similarity Analysis\n");
    output.push_str("======================\n");
    output.push_str(&format!("Query: '{query}'\n"));
    output.push_str(&format!("Total identifiers: {all_names_len}\n"));
    output.push_str(&format!("Matches found: {}\n", similarities.len()));

    if !similarities.is_empty() {
        output.push_str("\nTop matches:\n");
        for (i, sim) in similarities.iter().take(10).enumerate() {
            output.push_str(&format!(
                "{}. {} (similarity: {:.3}) in {}:{}\n",
                i + 1,
                sim.name,
                sim.similarity,
                sim.file_path
                    .file_name()
                    .expect("internal error")
                    .to_string_lossy(),
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
        output.push_str(&format!("  Files analyzed: {analyzed_files_len}\n"));
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
    debug_assert!(!query.is_empty(), "query must not be empty");
    let mut output = String::new();
    output.push_str("# Name Similarity Analysis\n\n");
    output.push_str(&format!("**Query**: `{query}`\n\n"));
    output.push_str(&format!("**Total identifiers**: {all_names_len}\n\n"));
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
                sim.file_path
                    .file_name()
                    .expect("internal error")
                    .to_string_lossy(),
                sim.line
            ));
        }
    }

    output
}
