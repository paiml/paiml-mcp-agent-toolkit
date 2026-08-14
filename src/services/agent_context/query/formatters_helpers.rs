// Shared helper functions for query result formatting: coverage metrics,
// truncation, rich metrics builders, call graph, fault lines, and source highlighting.
// Included into formatters.rs -- do NOT add `use` imports or `#!` inner attributes here.

// --- Coverage metric helpers (markdown) ---

fn format_coverage_metrics_md(r: &QueryResult, out: &mut String) {
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
    if diff > 0.0 {
        out.push_str(&format!(" | ✅ **+{:.1}% coverage**", diff));
    } else if diff < 0.0 {
        out.push_str(&format!(" | ❌ **{:.1}% coverage**", diff));
    }
}

// --- Coverage metric helpers (colorized text) ---

/// Colour a partial-coverage percentage is rendered in: red < 50%, yellow
/// < 80%, green otherwise.
///
/// Pure, so colour SELECTION stays assertable after colour EMISSION became
/// conditional — the tests below used to assert `out.contains("\x1b[1;31m")`,
/// which pinned the unconditional-escape defect rather than the tier rule.
fn coverage_tier_color(pct: f32) -> Sgr {
    if pct < 50.0 {
        BOLD_RED
    } else if pct < 80.0 {
        YELLOW
    } else {
        GREEN
    }
}

fn format_coverage_metrics_text(r: &QueryResult, out: &mut String) {
    match r.coverage_status.as_str() {
        "uncovered" => {
            out.push_str(&format!(
                " | {BOLD_RED}🛡️ Uncovered (0/{}){RESET}",
                r.lines_total
            ));
        }
        "partial" => {
            let cov_color = coverage_tier_color(r.line_coverage_pct);
            out.push_str(&format!(
                " | {}🛡️ Cov: {:.0}% ({}/{}){RESET}",
                cov_color, r.line_coverage_pct, r.lines_covered, r.lines_total
            ));
        }
        "full" => {
            out.push_str(&format!(
                " | {GREEN}🛡️ Covered ({} lines){RESET}",
                r.lines_total
            ));
        }
        _ => {}
    }
    if r.impact_score > 1.0 {
        out.push_str(&format!(
            " | {BOLD_YELLOW}📈 Impact: {:.1}{RESET}",
            r.impact_score
        ));
    }
    format_coverage_diff_text(r.coverage_diff, out);
}

fn format_coverage_diff_text(diff: f32, out: &mut String) {
    if diff > 0.0 {
        out.push_str(&format!(" | {BOLD_GREEN}✅ +{:.1}% cov{RESET}", diff));
    } else if diff < 0.0 {
        out.push_str(&format!(" | {BOLD_RED}❌ {:.1}% cov{RESET}", diff));
    }
}

// --- Truncation ---

#[allow(clippy::incompatible_msrv)]
fn truncate_doc(doc: &str) -> String {
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
    let mut metrics = Vec::new();
    metrics.push(format!("C:{}", r.complexity));
    metrics.push(format!("L:{}", r.loc));
    push_pagerank_metric(r, &mut metrics);
    push_indegree_metric(r, &mut metrics);
    push_churn_metric_rich(r, &mut metrics);
    push_entropy_metric(r, &mut metrics);
    if r.satd_count > 0 {
        metrics.push(format!("{BOLD_YELLOW}⚠{}{RESET}", r.satd_count));
    }
    if r.clone_count > 0 {
        metrics.push(format!("{BOLD_MAGENTA}📋{}{RESET}", r.clone_count));
    }
    push_coverage_metric_rich(r, &mut metrics);
    push_fault_metric_rich(r, &mut metrics);
    metrics
}

fn push_pagerank_metric(r: &QueryResult, metrics: &mut Vec<String>) {
    if r.pagerank <= 0.0 {
        return;
    }
    let pr_scaled = r.pagerank * 10000.0;
    if pr_scaled >= 10.0 {
        metrics.push(format!("{BOLD_CYAN}★{:.0}{RESET}", pr_scaled));
    } else if pr_scaled >= 1.0 {
        metrics.push(format!("★{:.1}", pr_scaled));
    }
}

fn push_indegree_metric(r: &QueryResult, metrics: &mut Vec<String>) {
    if r.in_degree >= 5 {
        metrics.push(format!("{BOLD_GREEN}↓{}{RESET}", r.in_degree));
    } else if r.in_degree > 0 {
        metrics.push(format!("↓{}", r.in_degree));
    }
}

fn push_churn_metric_rich(r: &QueryResult, metrics: &mut Vec<String>) {
    if r.commit_count == 0 {
        return;
    }
    if r.churn_score > 0.7 {
        metrics.push(format!(
            "{BOLD_RED}🔥{}c {:.0}%{RESET}",
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
    if r.pattern_diversity <= 0.0 {
        return;
    }
    if r.pattern_diversity < 0.3 {
        metrics.push(format!("{DIM}🔄{:.0}%{RESET}", r.pattern_diversity * 100.0));
    } else if r.pattern_diversity > 0.8 {
        metrics.push(format!("H:{:.0}%", r.pattern_diversity * 100.0));
    }
}

fn push_coverage_metric_rich(r: &QueryResult, metrics: &mut Vec<String>) {
    match r.coverage_status.as_str() {
        "uncovered" => {
            metrics.push(format!(
                "{BOLD_RED}\u{1f6e1}\u{fe0f}0/{}{RESET}",
                r.lines_total
            ));
        }
        "partial" => {
            let fmt = if r.line_coverage_pct < 50.0 {
                format!(
                    "{BOLD_RED}\u{1f6e1}\u{fe0f}{:.0}%{RESET}",
                    r.line_coverage_pct
                )
            } else if r.line_coverage_pct < 80.0 {
                format!("\u{1f6e1}\u{fe0f}{:.0}%", r.line_coverage_pct)
            } else {
                format!("{GREEN}\u{1f6e1}\u{fe0f}{:.0}%{RESET}", r.line_coverage_pct)
            };
            metrics.push(fmt);
        }
        "full" => {
            metrics.push(format!("{GREEN}\u{1f6e1}\u{fe0f}100%{RESET}"));
        }
        _ => {}
    }
    if r.impact_score > 1.0 {
        metrics.push(format!(
            "{BOLD_YELLOW}\u{1f4c8}{:.1}{RESET}",
            r.impact_score
        ));
    }
    if r.coverage_diff > 0.0 {
        metrics.push(format!("{BOLD_GREEN}+{:.1}%{RESET}", r.coverage_diff));
    } else if r.coverage_diff < 0.0 {
        metrics.push(format!("{BOLD_RED}{:.1}%{RESET}", r.coverage_diff));
    }
}

fn push_fault_metric_rich(r: &QueryResult, metrics: &mut Vec<String>) {
    if r.fault_annotations.is_empty() {
        return;
    }
    let first = r
        .fault_annotations
        .first()
        .map_or("", |s| s.split(':').next().unwrap_or(s));
    metrics.push(format!(
        "{BRIGHT_RED_BOLD}🐛{}:{}{RESET}",
        r.fault_annotations.len(),
        first
    ));
}

// --- Call graph formatting ---

fn format_call_graph(r: &QueryResult) -> Option<String> {
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
    for fault in faults {
        if fault.contains("Boundary") || fault.contains("condition") {
            output.push_str(&format!("{BOLD_YELLOW}⚠️  {}{RESET}\n", fault));
        } else if fault.contains("Arithmetic") {
            output.push_str(&format!("{BOLD_RED}⚠️  {}{RESET}\n", fault));
        } else {
            output.push_str(&format!("{BOLD_MAGENTA}⚠️  {}{RESET}\n", fault));
        }
    }
}

// --- Match highlighting and source rendering ---

/// Highlight matching text in a single line for grep-like output.
/// For literal mode (`is_regex=false`), does case-insensitive substring matching.
/// For regex mode (`is_regex=true`), uses regex pattern matching.
fn highlight_matches_in_line(line: &str, pattern: &str, is_regex: bool) -> String {
    // Bold + yellow background, gated on `--color`/`NO_COLOR` like everything
    // else this module prints: these used to be `const &str` raw escapes, which
    // is exactly how `--color never` leaked out of `pmat query`.
    let hl_start = BG_YELLOW_BOLD.to_string();
    let hl_end = RESET.to_string();

    if is_regex {
        if let Ok(re) = regex::Regex::new(pattern) {
            let mut result = String::new();
            let mut last = 0;
            for m in re.find_iter(line) {
                result.push_str(line.get(last..m.start()).unwrap_or_default());
                result.push_str(&hl_start);
                result.push_str(m.as_str());
                result.push_str(&hl_end);
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
            result.push_str(&hl_start);
            result.push_str(
                line.get(abs_idx..abs_idx + pattern.len())
                    .unwrap_or_default(),
            );
            result.push_str(&hl_end);
            pos = abs_idx + pattern.len();
        }
        result.push_str(line.get(pos..).unwrap_or_default());
        result
    }
}

/// Numbered, uncoloured source lines — the rendering used when syntect is not
/// compiled in AND when it is but colour is off.
fn plain_source_lines(source: &str, start_line: usize, output: &mut String) {
    for (i, line) in source.lines().enumerate() {
        let line_num = start_line + i;
        output.push_str(&format!("{DIM}{:>4}{RESET}\u{2502} {}\n", line_num, line));
    }
}

fn highlight_source(
    source: &str,
    file_path: &str,
    output: &mut String,
    start_line: usize,
    highlight: Option<(&str, bool)>,
) {
    if let Some((pattern, is_regex)) = highlight {
        // Match highlighting mode: line numbers + yellow highlight on matches
        for (i, line) in source.lines().enumerate() {
            let line_num = start_line + i;
            let highlighted = highlight_matches_in_line(line, pattern, is_regex);
            output.push_str(&format!(
                "{DIM}{:>4}{RESET}\u{2502} {}\n",
                line_num, highlighted
            ));
        }
    } else {
        // Syntect writes its own 24-bit escapes, which no `--color` rule can
        // reach from the outside, so the decision is made here: with colour off
        // the plain fallback runs instead.
        #[cfg(feature = "syntax-highlighting")]
        if crate::cli::colors::colors_enabled() {
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
            output.push_str(&RESET.to_string());
        } else {
            plain_source_lines(source, start_line, output);
        }
        #[cfg(not(feature = "syntax-highlighting"))]
        {
            // Plain text fallback when syntect is not available
            let _ = file_path; // Used only by syntax-highlighting feature
            plain_source_lines(source, start_line, output);
        }
    }
}
