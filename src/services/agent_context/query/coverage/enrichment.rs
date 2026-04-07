#![cfg_attr(coverage_nightly, coverage(off))]

use super::super::types::QueryResult;
use std::collections::HashMap;

// ── Pure Enrichment Function ────────────────────────────────────────────────

/// Find coverage data for a file path, trying exact match then suffix matching.
///
/// Returns `None` if no coverage entry matches the given file path.
fn find_coverage_for_file<'a>(
    file_path: &str,
    file_coverage: &'a HashMap<String, HashMap<usize, u64>>,
) -> Option<&'a HashMap<usize, u64>> {
    // Try exact match first
    if let Some(hits) = file_coverage.get(file_path) {
        return Some(hits);
    }
    // Fallback: find coverage entry whose key ends with our file_path or vice versa
    file_coverage
        .iter()
        .find(|(k, _)| k.ends_with(file_path) || file_path.ends_with(k.as_str()))
        .map(|(_, v)| v)
}

/// Determine coverage status string from total and covered line counts.
///
/// Returns one of: `"no_data"`, `"uncovered"`, `"full"`, or `"partial"`.
fn determine_coverage_status(total: u32, covered: u32) -> &'static str {
    if total == 0 {
        "no_data"
    } else if covered == 0 {
        "uncovered"
    } else if covered == total {
        "full"
    } else {
        "partial"
    }
}

/// Annotate coverage gaps as fault patterns on a single result.
///
/// Missing coverage is a latent defect signal — uncovered code paths are
/// where bugs hide. By surfacing these as fault annotations, `--coverage`
/// composes naturally with `--faults` to show "fix the defect AND write
/// the test" opportunities in a single view.
fn annotate_coverage_faults(result: &mut QueryResult) {
    match result.coverage_status.as_str() {
        "uncovered" => {
            result.fault_annotations.push(format!(
                "NO_COVERAGE: 0/{} lines covered — untested code path",
                result.lines_total
            ));
        }
        "partial" if result.line_coverage_pct < 50.0 => {
            result.fault_annotations.push(format!(
                "LOW_COVERAGE: {:.0}% ({}/{} lines) — {} missed lines need tests",
                result.line_coverage_pct,
                result.lines_covered,
                result.lines_total,
                result.missed_lines
            ));
        }
        _ => {}
    }
    if result.impact_score > 5.0 {
        result.fault_annotations.push(format!(
            "COVERAGE_RISK: impact {:.1} — high-ROI test target (missed:{} x pagerank)",
            result.impact_score, result.missed_lines
        ));
    }
}

/// Enrich query results with coverage data (pure function).
///
/// For each result, intersects `start_line..=end_line` with the file's
/// line_hits map:
/// - Lines in line_hits with count > 0 -> covered
/// - Lines in line_hits with count == 0 -> uncovered
/// - Lines NOT in line_hits -> non-instrumented (excluded from total)
///
/// Sets `coverage_status` to:
/// - `"no_data"` -- file not in coverage map at all
/// - `"uncovered"` -- file instrumented, 0 lines covered
/// - `"partial"` -- some lines covered, some not
/// - `"full"` -- all instrumented lines covered
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn enrich_with_coverage(
    results: &mut [QueryResult],
    file_coverage: &HashMap<String, HashMap<usize, u64>>,
) {
    for result in results.iter_mut() {
        let line_hits = match find_coverage_for_file(&result.file_path, file_coverage) {
            Some(hits) => hits,
            None => {
                result.coverage_status = "no_data".to_string();
                continue;
            }
        };

        let mut covered = 0u32;
        let mut total = 0u32;

        for line in result.start_line..=result.end_line {
            if let Some(&count) = line_hits.get(&line) {
                total += 1;
                if count > 0 {
                    covered += 1;
                }
            }
        }

        result.lines_covered = covered;
        result.lines_total = total;
        result.missed_lines = total.saturating_sub(covered);
        result.line_coverage_pct = if total > 0 {
            (covered as f32 / total as f32) * 100.0
        } else {
            0.0
        };
        result.impact_score =
            compute_impact_score(result.missed_lines, result.pagerank, result.complexity);

        result.coverage_status = determine_coverage_status(total, covered).to_string();

        // Annotate coverage gaps as fault patterns (missing coverage = latent defect signal)
        annotate_coverage_faults(result);
    }
}

/// Compute coverage delta: enriches results with `coverage_diff` field showing
/// change from baseline. Positive = coverage improved, negative = regressed.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn enrich_with_coverage_diff(
    results: &mut [QueryResult],
    baseline_coverage: &HashMap<String, HashMap<usize, u64>>,
) {
    for result in results.iter_mut() {
        let baseline_hits = match baseline_coverage.get(&result.file_path) {
            Some(hits) => hits,
            None => continue, // no baseline data for this file
        };

        let mut base_covered = 0u32;
        let mut base_total = 0u32;

        for line in result.start_line..=result.end_line {
            if let Some(&count) = baseline_hits.get(&line) {
                base_total += 1;
                if count > 0 {
                    base_covered += 1;
                }
            }
        }

        let base_pct = if base_total > 0 {
            base_covered as f32 / base_total as f32 * 100.0
        } else {
            0.0
        };

        // Store delta as current - baseline
        result.coverage_diff = result.line_coverage_pct - base_pct;
    }
}

/// Format a coverage summary line for display after enriched results.
///
/// Returns None if no results have coverage data.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn format_coverage_summary(results: &[QueryResult]) -> Option<String> {
    let with_data: Vec<_> = results
        .iter()
        .filter(|r| r.coverage_status != "no_data" && !r.coverage_status.is_empty())
        .collect();

    if with_data.is_empty() {
        return None;
    }

    let total_covered: u32 = with_data.iter().map(|r| r.lines_covered).sum();
    let total_lines: u32 = with_data.iter().map(|r| r.lines_total).sum();
    let total_pct = if total_lines > 0 {
        total_covered as f64 / total_lines as f64 * 100.0
    } else {
        0.0
    };

    let uncovered_count = with_data
        .iter()
        .filter(|r| r.coverage_status == "uncovered")
        .count();
    let partial_count = with_data
        .iter()
        .filter(|r| r.coverage_status == "partial")
        .count();

    let top_impact = with_data.iter().max_by(|a, b| {
        a.impact_score
            .partial_cmp(&b.impact_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut summary = format_coverage_header(total_covered, total_lines, total_pct, &with_data);
    append_coverage_counters(&mut summary, uncovered_count, partial_count);

    if let Some(top) = top_impact {
        if top.impact_score > 0.0 {
            summary.push_str(&format!(
                " | \x1b[1;33mTop impact: {} ({:.1})\x1b[0m",
                top.function_name, top.impact_score
            ));
        }
    }

    Some(summary)
}

/// Build the main coverage header line with color-coded percentage.
fn format_coverage_header(
    total_covered: u32,
    total_lines: u32,
    total_pct: f64,
    with_data: &[&QueryResult],
) -> String {
    let pct_color = if total_pct >= 80.0 {
        "\x1b[32m"
    } else if total_pct >= 50.0 {
        "\x1b[33m"
    } else {
        "\x1b[1;31m"
    };
    format!(
        "Coverage: {}/{} lines ({}{:.1}%\x1b[0m) across {} functions",
        total_covered,
        total_lines,
        pct_color,
        total_pct,
        with_data.len()
    )
}

/// Append uncovered/partial counters to summary string.
fn append_coverage_counters(summary: &mut String, uncovered_count: usize, partial_count: usize) {
    if uncovered_count > 0 {
        summary.push_str(&format!(
            " | \x1b[1;31m{} uncovered\x1b[0m",
            uncovered_count
        ));
    }
    if partial_count > 0 {
        summary.push_str(&format!(" | \x1b[33m{} partial\x1b[0m", partial_count));
    }
}

// ── Impact Score ────────────────────────────────────────────────────────────

/// ROI formula: missed_lines * pagerank_scaled * (1 / complexity_factor)
///
/// Higher impact = more uncovered lines in important, low-complexity code
/// (i.e., easy wins for coverage improvement).
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
pub fn compute_impact_score(missed_lines: u32, pagerank: f32, complexity: u32) -> f32 {
    debug_assert!(
        pagerank >= 0.0,
        "pagerank must be non-negative: {}",
        pagerank
    );
    if missed_lines == 0 {
        return 0.0;
    }
    let pr_factor = (pagerank * 10000.0).max(0.1);
    let complexity_factor = (complexity as f32).max(1.0);
    missed_lines as f32 * pr_factor / complexity_factor
}

#[cfg(kani)]
mod kani_proofs {
    use super::compute_impact_score;

    /// Prove: impact score is always non-negative for non-negative inputs.
    #[kani::proof]
    fn verify_impact_score_non_negative() {
        let missed: u32 = kani::any();
        let pagerank: f32 = kani::any();
        let complexity: u32 = kani::any();
        kani::assume(missed <= 10000);
        kani::assume(pagerank >= 0.0 && pagerank <= 1.0 && pagerank.is_finite());
        kani::assume(complexity <= 1000);
        let result = compute_impact_score(missed, pagerank, complexity);
        assert!(result >= 0.0, "impact score must be non-negative");
        assert!(result.is_finite(), "impact score must be finite");
    }

    /// Prove: zero missed lines always produces zero impact.
    #[kani::proof]
    fn verify_impact_score_zero_missed() {
        let pagerank: f32 = kani::any();
        let complexity: u32 = kani::any();
        kani::assume(pagerank >= 0.0 && pagerank.is_finite());
        kani::assume(complexity <= 1000);
        let result = compute_impact_score(0, pagerank, complexity);
        assert_eq!(result, 0.0, "zero missed lines must produce zero impact");
    }

    /// Prove: higher missed lines → higher or equal score (monotonicity).
    #[kani::proof]
    fn verify_impact_score_monotonic_missed() {
        let m1: u32 = kani::any();
        let m2: u32 = kani::any();
        let pagerank: f32 = kani::any();
        let complexity: u32 = kani::any();
        kani::assume(m1 <= m2 && m2 <= 1000);
        kani::assume(pagerank >= 0.0 && pagerank <= 1.0 && pagerank.is_finite());
        kani::assume(complexity > 0 && complexity <= 100);
        let s1 = compute_impact_score(m1, pagerank, complexity);
        let s2 = compute_impact_score(m2, pagerank, complexity);
        assert!(s2 >= s1, "more missed lines must not decrease score");
    }
}
