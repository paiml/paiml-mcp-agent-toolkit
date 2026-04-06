// Shared helper functions for query result formatting: coverage metrics,
// truncation, rich metrics builders, call graph, fault lines, and source highlighting.
// Included into formatters.rs -- do NOT add `use` imports or `#!` inner attributes here.

// --- Coverage metric helpers (markdown) ---

fn format_coverage_metrics_md(r: &QueryResult, out: &mut String) {
    debug_assert!(true, "contract: format_coverage_metrics_md");
    match r.coverage_status.as_str() {
        "uncovered" => {
            out.push_str(&format!(" | 🛡️ **Uncovered (0/{} lines)**", r.lines_total));
        }
        "partial" => {
            out.push_str(&format!(
                " | 🛡️ **Coverage: {:.0}%** ({}/{})",
                r.line_coverage_pct, r.lines_covered, r.lines_total
            ));
            out.push_str(&format!(" | ⚠️ **{} missed lines**", r.missed_lines));
        }
        "full" => {
            out.push_str(&format!(
                " | 🛡️ **Fully covered** ({} lines)",
                r.lines_total
            ));
        }
        _ => {}
    }
    if r.impact_score > 1.0 {
        out.push_str(&format!(" | 📈 **Impact: {:.1}**", r.impact_score));
    }
    format_coverage_diff_md(r.coverage_diff, out);
}

fn format_coverage_diff_md(diff: f32, out: &mut String) {
    debug_assert!(true, "contract: format_coverage_diff_md");
    if diff > 0.0 {
        out.push_str(&format!(" | ✅ **+{:.1}% coverage**", diff));
    } else if diff < 0.0 {
        out.push_str(&format!(" | ❌ **{:.1}% coverage**", diff));
    }
}

// --- Coverage metric helpers (colorized text) ---

fn format_coverage_metrics_text(r: &QueryResult, out: &mut String) {
    debug_assert!(true, "contract: format_coverage_metrics_text");
    match r.coverage_status.as_str() {
        "uncovered" => {
            out.push_str(&format!(
                " | \x1b[1;31m🛡️ Uncovered (0/{})\x1b[0m",
                r.lines_total
            ));
        }
        "partial" => {
            let cov_color = if r.line_coverage_pct < 50.0 {
                "\x1b[1;31m"
            } else if r.line_coverage_pct < 80.0 {
                "\x1b[33m"
            } else {
                "\x1b[32m"
            };
            out.push_str(&format!(
                " | {}🛡️ Cov: {:.0}% ({}/{})\x1b[0m",
                cov_color, r.line_coverage_pct, r.lines_covered, r.lines_total
            ));
        }
        "full" => {
            out.push_str(&format!(
                " | \x1b[32m🛡️ Covered ({} lines)\x1b[0m",
                r.lines_total
            ));
        }
        _ => {}
    }
    if r.impact_score > 1.0 {
        out.push_str(&format!(
            " | \x1b[1;33m📈 Impact: {:.1}\x1b[0m",
            r.impact_score
        ));
    }
    format_coverage_diff_text(r.coverage_diff, out);
}

fn format_coverage_diff_text(diff: f32, out: &mut String) {
    debug_assert!(true, "contract: format_coverage_diff_text");
    if diff > 0.0 {
        out.push_str(&format!(" | \x1b[1;32m✅ +{:.1}% cov\x1b[0m", diff));
    } else if diff < 0.0 {
        out.push_str(&format!(" | \x1b[1;31m❌ {:.1}% cov\x1b[0m", diff));
    }
}

// --- Truncation ---

#[allow(clippy::incompatible_msrv)]
fn truncate_doc(doc: &str) -> String {
    debug_assert!(!doc.is_empty(), "doc must not be empty");
    let first_line = doc.lines().next().unwrap_or(doc);
    if first_line.len() > 100 {
        format!(
            "{}...",
            first_line
                .get(..first_line.floor_char_boundary(97))
                .unwrap_or(first_line)
        )
    } else {
        first_line.to_string()
    }
}

// --- Rich metrics builders (used by format_text_with_code) ---

fn build_rich_metrics(r: &QueryResult) -> Vec<String> {
    debug_assert!(true, "contract: build_rich_metrics");
    let mut metrics = Vec::new();
    metrics.push(format!("C:{}", r.complexity));
    metrics.push(format!("L:{}", r.loc));
    push_pagerank_metric(r, &mut metrics);
    push_indegree_metric(r, &mut metrics);
    push_churn_metric_rich(r, &mut metrics);
    push_entropy_metric(r, &mut metrics);
    if r.satd_count > 0 {
        metrics.push(format!("\x1b[1;33m⚠{}\x1b[0m", r.satd_count));
    }
    if r.clone_count > 0 {
        metrics.push(format!("\x1b[1;35m📋{}\x1b[0m", r.clone_count));
    }
    push_coverage_metric_rich(r, &mut metrics);
    push_fault_metric_rich(r, &mut metrics);
    metrics
}

fn push_pagerank_metric(r: &QueryResult, metrics: &mut Vec<String>) {
    debug_assert!(true, "contract: push_pagerank_metric");
    if r.pagerank <= 0.0 {
        return;
    }
    let pr_scaled = r.pagerank * 10000.0;
    if pr_scaled >= 10.0 {
        metrics.push(format!("\x1b[1;36m★{:.0}\x1b[0m", pr_scaled));
    } else if pr_scaled >= 1.0 {
        metrics.push(format!("★{:.1}", pr_scaled));
    }
}

fn push_indegree_metric(r: &QueryResult, metrics: &mut Vec<String>) {
    debug_assert!(true, "contract: push_indegree_metric");
    if r.in_degree >= 5 {
        metrics.push(format!("\x1b[1;32m↓{}\x1b[0m", r.in_degree));
    } else if r.in_degree > 0 {
        metrics.push(format!("↓{}", r.in_degree));
    }
}

fn push_churn_metric_rich(r: &QueryResult, metrics: &mut Vec<String>) {
    debug_assert!(true, "contract: push_churn_metric_rich");
    if r.commit_count == 0 {
        return;
    }
    if r.churn_score > 0.7 {
        metrics.push(format!(
            "\x1b[1;31m🔥{}c {:.0}%\x1b[0m",
            r.commit_count,
            r.churn_score * 100.0
        ));
    } else if r.churn_score > 0.3 {
        metrics.push(format!("{}c {:.0}%", r.commit_count, r.churn_score * 100.0));
    } else {
        metrics.push(format!("{}c", r.commit_count));
    }
}

fn push_entropy_metric(r: &QueryResult, metrics: &mut Vec<String>) {
    debug_assert!(true, "contract: push_entropy_metric");
    if r.pattern_diversity <= 0.0 {
        return;
    }
    if r.pattern_diversity < 0.3 {
        metrics.push(format!(
            "\x1b[2m🔄{:.0}%\x1b[0m",
            r.pattern_diversity * 100.0
        ));
    } else if r.pattern_diversity > 0.8 {
        metrics.push(format!("H:{:.0}%", r.pattern_diversity * 100.0));
    }
}

fn push_coverage_metric_rich(r: &QueryResult, metrics: &mut Vec<String>) {
    debug_assert!(true, "contract: push_coverage_metric_rich");
    match r.coverage_status.as_str() {
        "uncovered" => {
            metrics.push(format!(
                "\x1b[1;31m\u{1f6e1}\u{fe0f}0/{}\x1b[0m",
                r.lines_total
            ));
        }
        "partial" => {
            let fmt = if r.line_coverage_pct < 50.0 {
                format!(
                    "\x1b[1;31m\u{1f6e1}\u{fe0f}{:.0}%\x1b[0m",
                    r.line_coverage_pct
                )
            } else if r.line_coverage_pct < 80.0 {
                format!("\u{1f6e1}\u{fe0f}{:.0}%", r.line_coverage_pct)
            } else {
                format!(
                    "\x1b[32m\u{1f6e1}\u{fe0f}{:.0}%\x1b[0m",
                    r.line_coverage_pct
                )
            };
            metrics.push(fmt);
        }
        "full" => {
            metrics.push("\x1b[32m\u{1f6e1}\u{fe0f}100%\x1b[0m".to_string());
        }
        _ => {}
    }
    if r.impact_score > 1.0 {
        metrics.push(format!("\x1b[1;33m\u{1f4c8}{:.1}\x1b[0m", r.impact_score));
    }
    if r.coverage_diff > 0.0 {
        metrics.push(format!("\x1b[1;32m+{:.1}%\x1b[0m", r.coverage_diff));
    } else if r.coverage_diff < 0.0 {
        metrics.push(format!("\x1b[1;31m{:.1}%\x1b[0m", r.coverage_diff));
    }
}

fn push_fault_metric_rich(r: &QueryResult, metrics: &mut Vec<String>) {
    debug_assert!(true, "contract: push_fault_metric_rich");
    if r.fault_annotations.is_empty() {
        return;
    }
    let first = r
        .fault_annotations
        .first()
        .map_or("", |s| s.split(':').next().unwrap_or(s));
    metrics.push(format!(
        "\x1b[1;91m🐛{}:{}\x1b[0m",
        r.fault_annotations.len(),
        first
    ));
}

// --- Call graph formatting ---

fn format_call_graph(r: &QueryResult) -> Option<String> {
    debug_assert!(true, "contract: format_call_graph");
    if r.calls.is_empty() && r.called_by.is_empty() {
        return None;
    }
    let mut parts = Vec::new();
    if !r.calls.is_empty() {
        let calls_str = if r.calls.len() <= 5 {
            r.calls.join(", ")
        } else {
            format!("{}, (+{} more)", r.calls[..5].join(", "), r.calls.len() - 5)
        };
        parts.push(format!("calls: {}", calls_str));
    }
    if !r.called_by.is_empty() {
        let called_str = if r.called_by.len() <= 3 {
            r.called_by.join(", ")
        } else {
            format!(
                "{}, (+{} more)",
                r.called_by[..3].join(", "),
                r.called_by.len() - 3
            )
        };
        parts.push(format!("← {}", called_str));
    }
    Some(parts.join(" │ "))
}

// --- Fault line formatting ---

fn format_fault_lines(faults: &[String], output: &mut String) {
    debug_assert!(!faults.is_empty(), "faults must not be empty");
    for fault in faults {
        if fault.contains("Boundary") || fault.contains("condition") {
            output.push_str(&format!("\x1b[1;33m⚠️  {}\x1b[0m\n", fault));
        } else if fault.contains("Arithmetic") {
            output.push_str(&format!("\x1b[1;31m⚠️  {}\x1b[0m\n", fault));
        } else {
            output.push_str(&format!("\x1b[1;35m⚠️  {}\x1b[0m\n", fault));
        }
    }
}

// --- Match highlighting and source rendering ---

/// Highlight matching text in a single line for grep-like output.
/// For literal mode (`is_regex=false`), does case-insensitive substring matching.
/// For regex mode (`is_regex=true`), uses regex pattern matching.
fn highlight_matches_in_line(line: &str, pattern: &str, is_regex: bool) -> String {
    debug_assert!(!line.is_empty(), "line must not be empty");
    const HL_START: &str = "\x1b[1;43m"; // Bold + yellow background
    const HL_END: &str = "\x1b[0m";

    if is_regex {
        if let Ok(re) = regex::Regex::new(pattern) {
            let mut result = String::new();
            let mut last = 0;
            for m in re.find_iter(line) {
                result.push_str(line.get(last..m.start()).unwrap_or_default());
                result.push_str(HL_START);
                result.push_str(m.as_str());
                result.push_str(HL_END);
                last = m.end();
            }
            result.push_str(line.get(last..).unwrap_or_default());
            result
        } else {
            line.to_string()
        }
    } else {
        // Case-insensitive literal replacement preserving original case
        let lower_line = line.to_lowercase();
        let lower_pattern = pattern.to_lowercase();
        if lower_pattern.is_empty() {
            return line.to_string();
        }
        let mut result = String::new();
        let mut pos = 0;
        while let Some(idx) = lower_line
            .get(pos..)
            .unwrap_or_default()
            .find(&lower_pattern)
        {
            let abs_idx = pos + idx;
            result.push_str(line.get(pos..abs_idx).unwrap_or_default());
            result.push_str(HL_START);
            result.push_str(
                line.get(abs_idx..abs_idx + pattern.len())
                    .unwrap_or_default(),
            );
            result.push_str(HL_END);
            pos = abs_idx + pattern.len();
        }
        result.push_str(line.get(pos..).unwrap_or_default());
        result
    }
}

fn highlight_source(
    source: &str,
    file_path: &str,
    output: &mut String,
    start_line: usize,
    highlight: Option<(&str, bool)>,
) {
    debug_assert!(!source.is_empty(), "source must not be empty");
    debug_assert!(!file_path.is_empty(), "file_path must not be empty");
    if let Some((pattern, is_regex)) = highlight {
        // Match highlighting mode: line numbers + yellow highlight on matches
        for (i, line) in source.lines().enumerate() {
            let line_num = start_line + i;
            let highlighted = highlight_matches_in_line(line, pattern, is_regex);
            output.push_str(&format!(
                "\x1b[2m{:>4}\x1b[0m\u{2502} {}\n",
                line_num, highlighted
            ));
        }
    } else {
        #[cfg(feature = "syntax-highlighting")]
        {
            // Syntect syntax highlighting mode
            use syntect::easy::HighlightLines;
            use syntect::highlighting::ThemeSet;
            use syntect::parsing::SyntaxSet;
            use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

            let ps = SyntaxSet::load_defaults_newlines();
            let ts = ThemeSet::load_defaults();
            let theme = &ts.themes["base16-ocean.dark"];

            let ext = file_path.rsplit('.').next().unwrap_or("rs");
            let syntax = ps
                .find_syntax_by_extension(ext)
                .unwrap_or_else(|| ps.find_syntax_plain_text());
            let mut h = HighlightLines::new(syntax, theme);

            for line in LinesWithEndings::from(source) {
                match h.highlight_line(line, &ps) {
                    Ok(ranges) => output.push_str(&as_24_bit_terminal_escaped(&ranges[..], false)),
                    Err(_) => output.push_str(line),
                }
            }
            if !source.ends_with('\n') {
                output.push('\n');
            }
            output.push_str("\x1b[0m");
        }
        #[cfg(not(feature = "syntax-highlighting"))]
        {
            // Plain text fallback when syntect is not available
            let _ = file_path; // Used only by syntax-highlighting feature
            for (i, line) in source.lines().enumerate() {
                let line_num = start_line + i;
                output.push_str(&format!("\x1b[2m{:>4}\x1b[0m\u{2502} {}\n", line_num, line));
            }
        }
    }
}
