/// Format provability results as summary
///
/// # Example
///
/// ```no_run
/// use pmat::cli::provability_helpers::format_provability_summary;
/// use pmat::services::lightweight_provability_analyzer::{FunctionId, ProofSummary};
/// use std::path::PathBuf;
///
/// let function_ids = vec![
///     FunctionId {
///         file_path: "src/main.rs".to_string(),
///         function_name: "high_score_func".to_string(),
///         line_number: 10,
///     },
///     FunctionId {
///         file_path: "src/lib.rs".to_string(),
///         function_name: "low_score_func".to_string(),
///         line_number: 20,
///     },
/// ];
///
/// let summaries = vec![
///     ProofSummary {
///         provability_score: 0.9,
///         analysis_time_us: 1000,
///         verified_properties: vec![],
///         version: 1,
///     },
///     ProofSummary {
///         provability_score: 0.3,
///         analysis_time_us: 500,
///         verified_properties: vec![],
///         version: 1,
///     },
/// ];
///
/// let output = format_provability_summary(&function_ids, &summaries, 5).unwrap();
///
/// assert!(output.contains("# Provability Analysis Summary"));
/// assert!(output.contains("Total functions analyzed: 2"));
/// assert!(output.contains("## Top Files by Provability"));
/// assert!(output.contains("1. `main.rs` - 90.0% avg score"));
/// ```
pub fn format_provability_summary(
    function_ids: &[FunctionId],
    summaries: &[ProofSummary],
    top_files: usize,
) -> Result<String> {
    let mut output = String::new();

    write_summary_header(&mut output, function_ids.len())?;
    write_scoring_model(&mut output)?;
    write_score_distribution(&mut output, summaries)?;
    write_property_coverage(&mut output, summaries)?;
    write_average_score(&mut output, summaries)?;
    write_lowest_scoring_functions(&mut output, function_ids, summaries, 10)?;
    write_top_files_section(&mut output, function_ids, summaries, top_files)?;

    Ok(output)
}

fn write_summary_header(output: &mut String, total_functions: usize) -> Result<()> {
    writeln!(output, "# Provability Analysis Summary\n")?;
    writeln!(output, "Total functions analyzed: {total_functions}")?;
    Ok(())
}

/// Explain the 4-factor scoring model so users understand what drives provability (#229).
fn write_scoring_model(output: &mut String) -> Result<()> {
    writeln!(output, "\n## Scoring Model (4 factors, equally weighted)\n")?;
    writeln!(output, "| Factor       | 100%         | 50%          | 0%               |")?;
    writeln!(output, "|--------------|--------------|--------------|------------------|")?;
    writeln!(output, "| Nullability  | NotNull      | MaybeNull    | Unknown/Null     |")?;
    writeln!(output, "| Bounds       | Both bounds  | One bound    | No bounds        |")?;
    writeln!(output, "| Aliasing     | NoAlias      | -            | MayAlias/Unknown |")?;
    writeln!(output, "| Purity       | Pure         | ReadOnly(70) | WriteGlobal      |")?;
    Ok(())
}

/// Show aggregate property verification coverage across all functions (#229).
fn write_property_coverage(output: &mut String, summaries: &[ProofSummary]) -> Result<()> {
    use crate::services::lightweight_provability_analyzer::PropertyType;

    if summaries.is_empty() {
        return Ok(());
    }

    let total = summaries.len();
    let mut counts: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    for name in &["NullSafety", "BoundsCheck", "NoAliasing", "PureFunction", "MemorySafety", "ThreadSafety"] {
        counts.insert(name, 0);
    }

    for s in summaries {
        for prop in &s.verified_properties {
            let key = match prop.property_type {
                PropertyType::NullSafety => "NullSafety",
                PropertyType::BoundsCheck => "BoundsCheck",
                PropertyType::NoAliasing => "NoAliasing",
                PropertyType::PureFunction => "PureFunction",
                PropertyType::MemorySafety => "MemorySafety",
                PropertyType::ThreadSafety => "ThreadSafety",
            };
            *counts.entry(key).or_default() += 1;
        }
    }

    writeln!(output, "\n## Verified Property Coverage\n")?;
    for (name, count) in &counts {
        let pct = (*count as f64 / total as f64) * 100.0;
        writeln!(output, "- {name}: {count}/{total} ({pct:.0}%)")?;
    }
    Ok(())
}

/// Show the lowest-scoring functions with their verified properties (#229).
fn write_lowest_scoring_functions(
    output: &mut String,
    function_ids: &[FunctionId],
    summaries: &[ProofSummary],
    limit: usize,
) -> Result<()> {
    if function_ids.is_empty() {
        return Ok(());
    }

    let mut indexed: Vec<(usize, f64)> = summaries
        .iter()
        .enumerate()
        .map(|(i, s)| (i, s.provability_score))
        .collect();
    indexed.sort_by(|a, b| a.1.total_cmp(&b.1));

    writeln!(output, "\n## Lowest Scoring Functions\n")?;
    for (idx, score) in indexed.iter().take(limit) {
        let func = &function_ids[*idx];
        let summary = &summaries[*idx];
        let filename = extract_filename(&func.file_path);
        let props: Vec<String> = summary
            .verified_properties
            .iter()
            .map(|p| format!("{:?}({:.0}%)", p.property_type, p.confidence * 100.0))
            .collect();
        let props_str = if props.is_empty() {
            "none verified".to_string()
        } else {
            props.join(", ")
        };
        writeln!(
            output,
            "- `{}` ({filename}:{}) \u{2014} {:.0}% \u{2014} verified: {props_str}",
            func.function_name,
            func.line_number,
            score * 100.0,
        )?;
    }
    Ok(())
}

fn write_score_distribution(output: &mut String, summaries: &[ProofSummary]) -> Result<()> {
    let (high_count, medium_count, low_count) = categorize_scores(summaries);

    writeln!(output, "\n## Score Distribution:")?;
    writeln!(output, "- High (\u{2265}80%): {high_count} functions")?;
    writeln!(output, "- Medium (50-79%): {medium_count} functions")?;
    writeln!(output, "- Low (<50%): {low_count} functions")?;

    Ok(())
}

fn write_average_score(output: &mut String, summaries: &[ProofSummary]) -> Result<()> {
    let avg_score = calculate_average_score(summaries);
    writeln!(
        output,
        "\nAverage provability score: {:.1}%",
        avg_score * 100.0
    )?;
    Ok(())
}

fn write_top_files_section(
    output: &mut String,
    function_ids: &[FunctionId],
    summaries: &[ProofSummary],
    top_files: usize,
) -> Result<()> {
    if function_ids.is_empty() {
        return Ok(());
    }

    writeln!(output, "\n## Top Files by Provability\n")?;
    let file_avg_scores = calculate_file_averages(function_ids, summaries);
    write_top_files_list(output, &file_avg_scores, top_files)?;

    Ok(())
}

fn calculate_file_averages<'a>(
    function_ids: &'a [FunctionId],
    summaries: &'a [ProofSummary],
) -> Vec<(&'a str, f64, usize)> {
    let mut file_scores: HashMap<&str, Vec<f64>> = HashMap::new();

    for (func_id, summary) in function_ids.iter().zip(summaries.iter()) {
        file_scores
            .entry(&func_id.file_path)
            .or_default()
            .push(summary.provability_score);
    }

    let mut file_avg_scores: Vec<_> = file_scores
        .iter()
        .map(|(file_path, scores)| {
            let avg_score = scores.iter().sum::<f64>() / scores.len() as f64;
            (*file_path, avg_score, scores.len())
        })
        .collect();

    file_avg_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    file_avg_scores
}

fn write_top_files_list(
    output: &mut String,
    file_avg_scores: &[(&str, f64, usize)],
    top_files: usize,
) -> Result<()> {
    let files_to_show = if top_files == 0 { 10 } else { top_files };

    for (i, (file_path, avg_score, function_count)) in
        file_avg_scores.iter().take(files_to_show).enumerate()
    {
        let filename = extract_filename(file_path);
        writeln!(
            output,
            "{}. `{}` - {:.1}% avg score ({} functions)",
            i + 1,
            filename,
            avg_score * 100.0,
            function_count
        )?;
    }

    Ok(())
}
