#![cfg_attr(coverage_nightly, coverage(off))]
//! Output formatting functions for Big-O complexity analysis

use crate::cli::colors as c;
use crate::cli::BigOOutputFormat;
use crate::models::complexity_bound::BigOClass;
use crate::services::big_o_analyzer::{BigOAnalysisReport, BigOAnalyzer};
use anyhow::Result;
use std::path::PathBuf;
use tracing::info;

/// Format analysis output
pub(super) fn format_analysis_output(
    analyzer: &BigOAnalyzer,
    report: &BigOAnalysisReport,
    format: BigOOutputFormat,
    high_complexity_only: bool,
) -> Result<String> {
    match format {
        BigOOutputFormat::Json => analyzer.format_as_json_scoped(report, high_complexity_only),
        BigOOutputFormat::Markdown => {
            Ok(analyzer.format_as_markdown_scoped(report, high_complexity_only))
        }
        BigOOutputFormat::Summary => Ok(format_big_o_summary_scoped(report, high_complexity_only)),
        BigOOutputFormat::Detailed => {
            Ok(format_big_o_detailed_scoped(report, high_complexity_only))
        }
    }
}

/// Write analysis output to file or stdout
pub(super) async fn write_analysis_output(content: &str, output: Option<PathBuf>) -> Result<()> {
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, content).await?;
        info!("📄 Big-O analysis saved to: {}", output_path.display());
    } else {
        println!("{content}");
    }
    Ok(())
}

/// Format Big-O report as summary with top files
///
/// # Examples
///
/// ```no_run
/// use pmat::cli::handlers::big_o_handlers::format_big_o_summary;
/// use pmat::services::big_o_analyzer::{BigOAnalysisReport, FunctionComplexity};
/// use pmat::models::complexity_bound::{ComplexityBound, BigOClass};
/// use std::path::PathBuf;
///
/// let report = BigOAnalysisReport {
///     analyzed_functions: 100,
///     high_complexity_functions: vec![
///         FunctionComplexity {
///             function_name: "sort_data".to_string(),
///             file_path: PathBuf::from("src/utils.rs"),
///             line_number: 42,
///             time_complexity: ComplexityBound::quadratic().with_confidence(90),
///             space_complexity: ComplexityBound::linear().with_confidence(85),
///             confidence: 90,
///             notes: vec![],
///         },
///     ],
///     complexity_distribution: pmat::services::big_o_analyzer::ComplexityDistribution {
///         constant: 20,
///         logarithmic: 10,
///         linear: 50,
///         linearithmic: 5,
///         quadratic: 10,
///         cubic: 2,
///         exponential: 1,
///         unknown: 2,
///     },
///     pattern_matches: vec![],
///     recommendations: vec!["Consider optimizing quadratic algorithms".to_string()],
/// };
///
/// let output = format_big_o_summary(&report);
/// assert!(output.contains("Top Files by Complexity"));
/// assert!(output.contains("utils.rs"));
/// ```
#[must_use]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn format_big_o_summary(report: &BigOAnalysisReport) -> String {
    format_big_o_summary_scoped(report, false)
}

/// `format_big_o_summary`, with the `--high-complexity-only` scope applied.
///
/// With the flag on the distribution lists only the O(n²)-or-worse rows, and
/// says which scope it is in. The flag used to be applied by `retain`ing a list
/// that had already been built with the same predicate, so it could not change
/// a byte of any format.
#[must_use]
pub(crate) fn format_big_o_summary_scoped(
    report: &BigOAnalysisReport,
    high_complexity_only: bool,
) -> String {
    let mut output = String::with_capacity(1024);

    output.push_str(&format!(
        "{}\n",
        c::header("Big-O Complexity Analysis Summary")
    ));
    output.push('\n');

    output.push_str(&format!(
        "  {}: {}\n",
        c::label("Total Functions Analyzed"),
        c::number(&report.analyzed_functions.to_string()),
    ));
    let high_color = if report.high_complexity_functions.is_empty() {
        c::GREEN
    } else {
        c::YELLOW
    };
    // Same disclosure as the json/markdown renderers: the LISTED count is
    // capped by `--top-files`; the FOUND count (from the unfiltered
    // distribution) is not. Printing only the capped number made the default
    // run report 24 where the project had 106.
    let dist_found = report.complexity_distribution.quadratic
        + report.complexity_distribution.cubic
        + report.complexity_distribution.exponential;
    let listed = report.high_complexity_functions.len();
    let found = dist_found.max(listed);
    let high_suffix = if listed < found {
        format!(" (of {found} found; list truncated by --top-files)")
    } else {
        String::new()
    };
    // COLOUR: these used to interpolate the raw `c::GREEN`/`c::RESET` consts,
    // which are unconditional — so `--color never` and a redirected stdout
    // still wrote `^[[32m0^[[0m` here (GH #684 class). `c::colored` consults
    // `colors_enabled()` and yields the bare payload when colour is off.
    output.push_str(&format!(
        "  {}: {}{}\n\n",
        c::label("High Complexity Functions"),
        c::colored(high_color, &listed.to_string()),
        high_suffix,
    ));

    if high_complexity_only {
        output.push_str(&format!(
            "{}\n",
            c::subheader("Complexity Distribution (--high-complexity-only):")
        ));
    } else {
        output.push_str(&format!("{}\n", c::subheader("Complexity Distribution:")));
    }
    // The counts and the is-high predicate come from
    // `BigOAnalyzer::distribution_rows`, so the terminal, markdown and JSON
    // renderers cannot disagree about which rows the flag keeps. Only the
    // display label and colour are local (the terminal spells O(n²), markdown
    // O(n^2), JSON keys it "O(n^2)").
    const ROWS: [(&str, c::Sgr, usize); 8] = [
        ("O(1)", c::GREEN, 7),
        ("O(log n)", c::GREEN, 3),
        ("O(n)", c::YELLOW, 7),
        ("O(n log n)", c::YELLOW, 1),
        ("O(n²)", c::RED, 6),
        ("O(n³)", c::RED, 6),
        ("O(2^n)", c::BOLD_RED, 5),
        ("Unknown", c::DIM, 4),
    ];
    for ((_, count, is_high), (label, colour, pad)) in BigOAnalyzer::distribution_rows(report)
        .into_iter()
        .zip(ROWS)
    {
        if !BigOAnalyzer::distribution_row_kept(is_high, high_complexity_only) {
            continue;
        }
        output.push_str(&format!(
            "  {}{:pad$}: {} functions\n",
            c::colored(colour, label),
            "",
            c::number(&format!("{count:>4}")),
        ));
    }

    if !report.recommendations.is_empty() {
        output.push_str(&format!("\n{}\n", c::subheader("Recommendations:")));
        for rec in &report.recommendations {
            output.push_str(&format!("  {} {rec}\n", c::warn("")));
        }
    }

    // Show top files by complexity
    if !report.high_complexity_functions.is_empty() {
        output.push_str(&format!("\n{}\n", c::subheader("Top Files by Complexity:")));

        // Group functions by file
        use std::collections::HashMap;
        let mut file_scores: HashMap<&std::path::Path, f64> = HashMap::new();
        let mut file_function_counts: HashMap<&std::path::Path, usize> = HashMap::new();

        for func in &report.high_complexity_functions {
            let score = match func.time_complexity.class {
                BigOClass::Constant => 1.0,
                BigOClass::Logarithmic => 2.0,
                BigOClass::Linear => 3.0,
                BigOClass::Linearithmic => 4.0,
                BigOClass::Quadratic => 5.0,
                BigOClass::Cubic => 6.0,
                BigOClass::Exponential => 7.0,
                BigOClass::Factorial => 8.0,
                BigOClass::Unknown => 3.0,
            };
            *file_scores.entry(&func.file_path).or_insert(0.0) += score;
            *file_function_counts.entry(&func.file_path).or_insert(0) += 1;
        }

        // Sort files by total complexity score. DETERMINISM: the score is not a
        // total order (most files tie), the source is a `HashMap`, and
        // `sort_by` is stable — so which tied file appeared in the top 10 came
        // out of the process's hash seed. Path breaks the tie.
        let mut sorted_files: Vec<_> = file_scores.into_iter().collect();
        sorted_files.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(b.0))
        });

        // Display top 10 files
        for (i, (file_path, score)) in sorted_files.iter().take(10).enumerate() {
            let filename = file_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(file_path.to_str().unwrap_or("unknown"));
            let function_count = file_function_counts.get(file_path).unwrap_or(&0);
            let score_color = if *score > 20.0 {
                c::RED
            } else if *score > 10.0 {
                c::YELLOW
            } else {
                c::GREEN
            };
            output.push_str(&format!(
                "  {}. {} - score: {}, {} functions\n",
                c::number(&(i + 1).to_string()),
                c::path(filename),
                c::colored(score_color, &format!("{score:.1}")),
                c::number(&function_count.to_string()),
            ));
        }
    }

    output
}

/// Format Big-O report with detailed information
pub(super) fn format_big_o_detailed(report: &BigOAnalysisReport) -> String {
    format_big_o_detailed_scoped(report, false)
}

/// `format_big_o_detailed`, with the `--high-complexity-only` scope applied.
pub(super) fn format_big_o_detailed_scoped(
    report: &BigOAnalysisReport,
    high_complexity_only: bool,
) -> String {
    let mut output = format_big_o_summary_scoped(report, high_complexity_only);

    if !report.high_complexity_functions.is_empty() {
        output.push_str(&format!("\n{}\n", c::header("High Complexity Functions:")));

        for func in &report.high_complexity_functions {
            output.push_str(&format!(
                "\n{} ({}:{})\n",
                c::label(&func.function_name),
                c::path(&func.file_path.display().to_string()),
                c::colored(c::DIM, &func.line_number.to_string()),
            ));
            output.push_str(&format!(
                "  {}: {} ({})\n",
                c::label("Time Complexity"),
                func.time_complexity.notation(),
                c::pct(func.time_complexity.confidence as f64, 80.0, 50.0),
            ));
            output.push_str(&format!(
                "  {}: {} ({})\n",
                c::label("Space Complexity"),
                func.space_complexity.notation(),
                c::pct(func.space_complexity.confidence as f64, 80.0, 50.0),
            ));

            if !func.notes.is_empty() {
                output.push_str(&format!("  {}:\n", c::label("Notes")));
                for note in &func.notes {
                    output.push_str(&format!("    {} {note}\n", c::colored(c::DIM, "─")));
                }
            }
        }
    }

    if !report.pattern_matches.is_empty() {
        output.push_str(&format!("\n{}\n", c::header("Pattern Matches:")));

        for pattern in &report.pattern_matches {
            output.push_str(&format!(
                "  {} : {} occurrences\n",
                c::label(&pattern.pattern_name),
                c::number(&pattern.occurrences.to_string()),
            ));
        }
    }

    output
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod colour_gating_tests {
    //! `analyze big-o --color never` (and a plain redirected stdout) still wrote
    //! `^[[32m0^[[0m` for the high-complexity count and `^[[32mO(1)^[[0m` for
    //! every distribution row, because those rows interpolated the raw
    //! `c::GREEN` / `c::RESET` consts instead of a helper that consults
    //! `colors_enabled()`.
    use super::{format_big_o_detailed, format_big_o_summary};
    use crate::models::complexity_bound::ComplexityBound;
    use crate::services::big_o_analyzer::{
        BigOAnalysisReport, ComplexityDistribution, FunctionComplexity,
    };
    use std::path::PathBuf;

    fn report() -> BigOAnalysisReport {
        BigOAnalysisReport {
            analyzed_functions: 100,
            high_complexity_functions: vec![FunctionComplexity {
                function_name: "sort_data".to_string(),
                file_path: PathBuf::from("src/utils.rs"),
                line_number: 42,
                time_complexity: ComplexityBound::quadratic().with_confidence(90),
                space_complexity: ComplexityBound::linear().with_confidence(85),
                confidence: 90,
                notes: vec!["nested loop".to_string()],
            }],
            complexity_distribution: ComplexityDistribution {
                constant: 20,
                logarithmic: 10,
                linear: 50,
                linearithmic: 5,
                quadratic: 10,
                cubic: 2,
                exponential: 1,
                unknown: 2,
            },
            pattern_matches: vec![],
            recommendations: vec!["Consider optimizing quadratic algorithms".to_string()],
        }
    }

    #[test]
    fn summary_emits_no_ansi_when_colour_is_disabled() {
        assert!(
            !crate::cli::colors::colors_enabled(),
            "cargo test captures stdout, so colour must resolve to off here"
        );
        let out = format_big_o_summary(&report());
        assert!(
            !out.contains('\x1b'),
            "big-o summary must be plain with colour off, got {out:?}"
        );
    }

    #[test]
    fn detailed_emits_no_ansi_when_colour_is_disabled() {
        let out = format_big_o_detailed(&report());
        assert!(
            !out.contains('\x1b'),
            "big-o detailed must be plain with colour off, got {out:?}"
        );
    }

    #[test]
    fn summary_keeps_its_payload_text() {
        // The escapes must go, not the numbers they wrapped.
        let out = format_big_o_summary(&report());
        assert!(out.contains("O(1)"), "{out}");
        assert!(out.contains("O(log n)"), "{out}");
        assert!(out.contains("O(2^n)"), "{out}");
        assert!(out.contains("Unknown"), "{out}");
        assert!(out.contains("High Complexity Functions"), "{out}");
        assert!(out.contains("utils.rs"), "{out}");
    }
}
