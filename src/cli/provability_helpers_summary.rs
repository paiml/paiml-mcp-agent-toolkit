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
/// assert!(output.contains("Provability Analysis Summary"));
/// assert!(output.contains("Total functions analyzed:"));
/// assert!(output.contains("Top Files by Provability"));
/// ```
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
    use crate::cli::colors as c;
    writeln!(output, "{}\n", c::header("Provability Analysis Summary"))?;
    writeln!(output, "Total functions analyzed: {}", c::number(&total_functions.to_string()))?;
    Ok(())
}

/// Explain the 4-factor scoring model so users understand what drives provability (#229).
fn write_scoring_model(output: &mut String) -> Result<()> {
    use crate::cli::colors as c;
    writeln!(output, "\n{}\n", c::subheader("Scoring Model (4 factors, equally weighted)"))?;
    writeln!(output, "  {}{:<14}{} {:<14} {:<14} 0%", c::seq(c::BOLD), "Factor", c::seq(c::RESET), "100%", "50%")?;
    writeln!(output, "  {}", c::separator())?;
    writeln!(output, "  {:<14} {}NotNull{}       {}MaybeNull{}    {}Unknown/Null{}", "Nullability", c::seq(c::GREEN), c::seq(c::RESET), c::seq(c::YELLOW), c::seq(c::RESET), c::seq(c::RED), c::seq(c::RESET))?;
    writeln!(output, "  {:<14} {}Both bounds{}   {}One bound{}    {}No bounds{}", "Bounds", c::seq(c::GREEN), c::seq(c::RESET), c::seq(c::YELLOW), c::seq(c::RESET), c::seq(c::RED), c::seq(c::RESET))?;
    writeln!(output, "  {:<14} {}NoAlias{}       {:<14} {}MayAlias/Unknown{}", "Aliasing", c::seq(c::GREEN), c::seq(c::RESET), "-", c::seq(c::RED), c::seq(c::RESET))?;
    writeln!(output, "  {:<14} {}Pure{}          {}ReadOnly(70){} {}WriteGlobal{}", "Purity", c::seq(c::GREEN), c::seq(c::RESET), c::seq(c::YELLOW), c::seq(c::RESET), c::seq(c::RED), c::seq(c::RESET))?;
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

    use crate::cli::colors as c;
    writeln!(output, "\n{}\n", c::subheader("Verified Property Coverage"))?;
    for (name, count) in &counts {
        let pct_val = (*count as f64 / total as f64) * 100.0;
        writeln!(output, "  {}{}{}: {}/{} ({})",
            c::seq(c::BOLD), name, c::seq(c::RESET),
            c::number(&count.to_string()),
            c::number(&total.to_string()),
            c::pct(pct_val, 80.0, 50.0),
        )?;
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

    use crate::cli::colors as c;
    writeln!(output, "\n{}\n", c::subheader("Lowest Scoring Functions"))?;
    for (idx, score_val) in indexed.iter().take(limit) {
        let func = &function_ids[*idx];
        let summary = &summaries[*idx];
        let filename = extract_filename(&func.file_path);
        let props: Vec<String> = summary
            .verified_properties
            .iter()
            .map(|p| format!("{:?}({:.0}%)", p.property_type, p.confidence * 100.0))
            .collect();
        let props_str = if props.is_empty() {
            format!("{}none verified{}", c::seq(c::DIM), c::seq(c::RESET))
        } else {
            props.join(", ")
        };
        writeln!(
            output,
            "  {} ({}:{}) \u{2014} {} \u{2014} verified: {props_str}",
            c::label(&func.function_name),
            c::path(filename),
            c::number(&func.line_number.to_string()),
            c::pct(score_val * 100.0, 80.0, 50.0),
        )?;
    }
    Ok(())
}

fn write_score_distribution(output: &mut String, summaries: &[ProofSummary]) -> Result<()> {
    use crate::cli::colors as c;
    let (high_count, medium_count, low_count) = categorize_scores(summaries);

    writeln!(output, "\n{}", c::subheader("Score Distribution:"))?;
    writeln!(output, "  {}High{} ({}\u{2265}80%{}): {} functions", c::seq(c::GREEN), c::seq(c::RESET), c::seq(c::GREEN), c::seq(c::RESET), c::number(&high_count.to_string()))?;
    writeln!(output, "  {}Medium{} ({}50-79%{}): {} functions", c::seq(c::YELLOW), c::seq(c::RESET), c::seq(c::YELLOW), c::seq(c::RESET), c::number(&medium_count.to_string()))?;
    writeln!(output, "  {}Low{} ({}<50%{}): {} functions", c::seq(c::RED), c::seq(c::RESET), c::seq(c::RED), c::seq(c::RESET), c::number(&low_count.to_string()))?;

    Ok(())
}

fn write_average_score(output: &mut String, summaries: &[ProofSummary]) -> Result<()> {
    use crate::cli::colors as c;
    let avg_score = calculate_average_score(summaries);
    writeln!(
        output,
        "\nAverage provability score: {}",
        c::pct(avg_score * 100.0, 80.0, 50.0)
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

    use crate::cli::colors as c;
    writeln!(output, "\n{}\n", c::subheader("Top Files by Provability"))?;
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

    // DETERMINISM (round-3 sweep): the average score is not a total order —
    // two files can score identically, and on a small tree they usually do —
    // and the ranking is built from a `HashMap`, so `.take(n)` printed
    // "1. b.rs / 2. a.rs" on one run and "1. a.rs / 2. b.rs" on the next for
    // byte-identical input. Path breaks the tie.
    file_avg_scores.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.cmp(b.0))
    });
    file_avg_scores
}

fn write_top_files_list(
    output: &mut String,
    file_avg_scores: &[(&str, f64, usize)],
    top_files: usize,
) -> Result<()> {
    use crate::cli::colors as c;
    let files_to_show = if top_files == 0 { 10 } else { top_files };

    for (i, (file_path, avg_score, function_count)) in
        file_avg_scores.iter().take(files_to_show).enumerate()
    {
        let filename = extract_filename(file_path);
        writeln!(
            output,
            "  {}. {} - {} avg score ({} functions)",
            c::number(&(i + 1).to_string()),
            c::path(filename),
            c::pct(avg_score * 100.0, 80.0, 50.0),
            c::number(&function_count.to_string()),
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod plain_output_tests {
    //! `--color never`, `NO_COLOR=1` and a redirected stdout all left 17
    //! escape-bearing lines in `analyze provability`'s summary: the renderer
    //! interpolated the raw `pub const` sequences, which are `const` and so
    //! cannot consult `colors_enabled()`. Only `--color always` differed —
    //! i.e. the flag parsed and only its "always" branch did anything.
    use super::*;
    use crate::services::lightweight_provability_analyzer::{FunctionId, ProofSummary};

    fn fixture() -> (Vec<FunctionId>, Vec<ProofSummary>) {
        let ids = vec![
            FunctionId {
                file_path: "src/main.rs".to_string(),
                function_name: "high".to_string(),
                line_number: 10,
            },
            FunctionId {
                file_path: "src/lib.rs".to_string(),
                function_name: "low".to_string(),
                line_number: 20,
            },
        ];
        let summaries = vec![
            ProofSummary {
                provability_score: 0.9,
                analysis_time_us: 1000,
                verified_properties: vec![],
                version: 1,
            },
            ProofSummary {
                provability_score: 0.3,
                analysis_time_us: 500,
                verified_properties: vec![],
                version: 1,
            },
        ];
        (ids, summaries)
    }

    #[test]
    fn summary_is_plain_text_when_colour_is_disabled() {
        assert!(
            !crate::cli::colors::colors_enabled(),
            "cargo test captures stdout, so colour must resolve to off here"
        );

        let (ids, summaries) = fixture();
        let rendered = format_provability_summary(&ids, &summaries, 5).expect("render");

        assert!(
            !rendered.contains('\u{1b}'),
            "no ANSI escape may reach a redirected stdout: {:?}",
            rendered
                .lines()
                .filter(|l| l.contains('\u{1b}'))
                .collect::<Vec<_>>()
        );
        // The payload must survive the de-colouring.
        assert!(rendered.contains("Provability Analysis Summary"));
        assert!(rendered.contains("Scoring Model"));
        assert!(rendered.contains("Score Distribution"));
        assert!(rendered.contains("Top Files by Provability"));
    }
}
