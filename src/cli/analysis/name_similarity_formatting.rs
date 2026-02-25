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
fn format_human_match_entry(output: &mut String, index: usize, m: &NameMatch) -> Result<()> {
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
        m.name, m.file, m.line, m.kind, m.similarity_score, m.edit_distance, m.phonetic_match
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
